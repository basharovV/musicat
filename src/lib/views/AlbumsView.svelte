<script lang="ts">
    import { liveQuery } from "dexie";
    import md5 from "md5";
    import { onMount } from "svelte";
    import type { Album, AlbumsSortBy, Song } from "../../App";
    import { db, getAlbumTracks } from "../../data/db";
    import {
        compressionSelected,
        current,
        isPlaying,
        query,
        queue,
        uiPreferences,
        uiView,
        userSettings,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import AlbumDetails from "../albums/AlbumDetails.svelte";
    import AlbumItem from "../albums/AlbumItem.svelte";
    import AlbumMenu from "../albums/AlbumMenu.svelte";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import { path } from "@tauri-apps/api";
    import VirtualList from "svelte-tiny-virtual-list";
    import { debounce } from "lodash-es";
    import { getAlbumDetailsHeight } from "../albums/util";
    import ScrollTo from "../ui/ScrollTo.svelte";
    import { fade, fly, scale } from "svelte/transition";
    import {
        cubicIn,
        cubicInOut,
        cubicOut,
        quadInOut,
        quartIn,
        quartInOut,
        quintIn,
        sineInOut,
    } from "svelte/easing";
    import {
        beetsAlbumSearch,
        beetsAlbumsOnly,
        beetsSearch,
        createBeetsSearch,
    } from "../../data/beets";
    import { derived } from "svelte/store";
    import { invoke } from "@tauri-apps/api/core";

    const PADDING = 14;

    let activeAlbums: Album[] = [];
    let albumMenu: AlbumMenu;
    let columnWidth = 0;
    let container: HTMLDivElement;
    let gridContainer: HTMLDivElement;
    let currentAlbum: Album;
    let currentAlbumOffset = 0;
    let detailsAlbum: Album = null;
    let detailsPosition = { x: 0, y: 0, width: 0, height: 0 };
    let maskPosition = { x: 0, y: 0, width: 0, height: 0 };
    let detailsAlbumHeight = 0;
    let detailsAlbumIndex = -1;
    let detailsAlbumRow = -1;
    let detailsAlbumTracks: Song[] = null;
    let highlightedAlbum;
    let isCurrentAlbumInView = false;
    let isInit = true;
    let isLoading = true;
    let itemSizes = [];
    let lastOffset = 0;
    let rowCount = 0;
    let rowHeight = 0;
    let virtualList;

    $: albums = $userSettings.beetsDbLocation
        ? beetsAlbumsOnly
        : liveQuery(async () => {
              let albums = await db.albums.toArray();

              if ($compressionSelected === "lossless") {
                  albums = albums.filter(
                      ({ title, lossless }) => title.length && lossless,
                  );
              } else if ($compressionSelected === "lossy") {
                  albums = albums.filter(
                      ({ title, lossless }) => title.length && !lossless,
                  );
              } else {
                  albums = albums.filter(({ title }) => title.length);
              }

              if ($uiPreferences.albumsViewSortBy === "title") {
                  albums.sort((a, b) => {
                      if (a.title < b.title) return -1;
                      if (a.title > b.title) return 1;
                      return 0;
                  });
              } else if ($uiPreferences.albumsViewSortBy === "artist") {
                  albums.sort((a, b) => {
                      if (a.artist < b.artist) return -1;
                      if (a.artist > b.artist) return 1;
                      if (a.title < b.title) return -1;
                      if (a.title > b.title) return 1;
                      return 0;
                  });
              } else {
                  albums.sort((a, b) => {
                      if (a.year < b.year) return -1;
                      if (a.year > b.year) return 1;
                      if (a.title < b.title) return -1;
                      if (a.title > b.title) return 1;
                      return 0;
                  });
              }

              isLoading = false;

              return albums;
          });

    $: if (isInit && $queue && $current?.song) {
        console.log("Scrolling to current album");
        showCurrentlyPlayingAlbum();
    } else {
        isInit = false;
    }

    $: if (
        container &&
        $current.song?.album.toLowerCase() !== currentAlbum?.title.toLowerCase()
    ) {
        if (updatePlayingAlbum()) {
            updateInView();
        }
    }

    $: columnCount = 0;
    $: minWidth = $uiPreferences.albumsViewGridSize;
    $: queriedAlbums = [];
    $: showInfo = $uiPreferences.albumsViewShowInfo;
    $: showSingles = $uiPreferences.albumsViewShowSingles;

    $: if ($albums && columnCount) {
        activeAlbums = $albums;

        if (!showSingles) {
            activeAlbums = activeAlbums.filter((a) => a.tracksIds.length > 1);
            console.log("Filtered", activeAlbums.length);
        }

        onResize();
    }

    async function onQueryChanged(query: string, sortBy: AlbumsSortBy) {
        if ($userSettings.beetsDbLocation) {
            await beetsAlbumSearch.updateSearch({
                query,
                sortBy: sortBy,
                descending: false,
            });
        } else {
            queriedAlbums =
                $albums?.filter(
                    (a) =>
                        a.artist.toLowerCase().includes(query.toLowerCase()) ||
                        a.title.includes(query.toLowerCase()),
                ) || [];

            activeAlbums = queriedAlbums;
        }

        if (columnCount) {
            rowCount = Math.ceil(activeAlbums.length / columnCount) + 2;
            // Add this line to keep sizes in sync with the new count
            itemSizes = Array(rowCount).fill(rowHeight);
        }

        if (previousQuery !== query) {
            virtualList.scrollToIndex = 0;
        }
        previousQuery = query || "";

        isLoading = false;
    }

    let previousQuery = $query || "";

    $: {
        onQueryChanged($query || "", $uiPreferences.albumsViewSortBy);
    }

    function getContentWidth(element) {
        const widthWithPaddings = element.clientWidth;
        const elementComputedStyle = window.getComputedStyle(element, null);

        return (
            widthWithPaddings -
            parseFloat(elementComputedStyle.paddingLeft) -
            parseFloat(elementComputedStyle.paddingRight)
        );
    }

    async function showCurrentlyPlayingAlbum() {
        if (!$current?.song) return;

        if (await updatePlayingAlbum()) {
            scrollToCurrentAlbum();

            isInit = false;
        }
    }

    async function updatePlayingAlbum() {
        if ($userSettings.beetsDbLocation) {
            const result: Album[] = await invoke("get_albums_by_id", {
                albumIds: [$current?.song?.albumId],
            });
            if (result.length === 0) return false;
            currentAlbum = result[0];
        } else {
            // Strip the song from album path
            const albumPath = await path.dirname($current?.song?.path);
            // Find the album currently playing
            currentAlbum = await db.albums.get(
                md5(`${albumPath} - ${$current?.song?.album}`.toLowerCase()),
            );
        }

        if (!currentAlbum) return false;

        updatePlayingAlbumOffset();

        return true;
    }

    function updatePlayingAlbumOffset() {
        const { id } = currentAlbum;
        const index = activeAlbums.findIndex((album) => album.id === id);

        currentAlbumOffset = Math.round(
            (Math.floor(index / columnCount) + 0.5) * rowHeight,
        );
    }

    function onAfterScroll(e) {
        if (currentAlbum && container) {
            const { offset } = e.detail;
            updateInView(offset);
        }
    }

    let height = 0;
    let width = 0;
    let detailsContainer;

    function onResize() {
        if (!container) {
            return;
        }

        height = container?.clientHeight;
        width = container?.clientWidth;

        let contentWidth = getContentWidth(container) - PADDING - PADDING;
        let count = Math.floor(contentWidth / minWidth);

        if (count === 0) {
            count = 1;
            contentWidth = width - PADDING - PADDING;
        }
        if (count === 1) {
            if (contentWidth / minWidth >= 1.5) {
                count = 2;
                contentWidth = Math.floor(contentWidth / 2);
            }
        }

        const remaining = contentWidth - count * minWidth;
        const perColumn = Math.floor(remaining / count);
        const max = Math.floor(contentWidth * 0.1);

        if (minWidth + perColumn > max) {
            columnWidth = Math.max(max, minWidth + Math.floor(perColumn / 2));
        } else {
            columnWidth = minWidth + perColumn;
        }

        columnCount = count;
        rowCount = Math.ceil(activeAlbums.length / columnCount) + 2;
        rowHeight = Math.floor(columnWidth + (showInfo ? 55 : 0));
        const newSizes = Array(rowCount).fill(rowHeight);
        if (newSizes.length > 0) {
            newSizes[0] = PADDING;
            newSizes[newSizes.length - 1] = PADDING;
        }
        itemSizes = newSizes;

        console.log("itemSizes", itemSizes, "rowCount", rowCount);
        if (currentAlbum) {
            updatePlayingAlbumOffset();
            updateInView();
        }

        if (detailsAlbum) {
            // Update position
            const albumElement: HTMLDivElement = document.querySelector(
                `[data-album="${detailsAlbum.id}"]`,
            );
            const isInView = (element) => {
                const bounding = element.getBoundingClientRect();

                return (
                    bounding.top >= 0 &&
                    bounding.left >= 0 &&
                    bounding.bottom <=
                        (window.innerHeight ||
                            document.documentElement.clientHeight) &&
                    bounding.right <=
                        (window.innerWidth ||
                            document.documentElement.clientWidth)
                );
            };

            if (!albumElement || !isInView(albumElement)) {
                // Can't find album element or it's not in view - gone out of view
                unselectAlbum();
            } else {
                detailsPosition = albumElement.getBoundingClientRect();
                maskPosition = {
                    x: albumElement.offsetLeft,
                    y: detailsPosition.y - 35,
                    width: detailsPosition.width,
                    height: detailsPosition.height,
                };
            }
        }

        if (virtualList?.scrollToIndex) virtualList.scrollToIndex = null;
    }

    async function onRightClick(e, album) {
        highlightedAlbum = album.id;

        const songs = await getAlbumTracks(album);
        albumMenu?.open(album, songs, { x: e.clientX, y: e.clientY });
    }

    async function onLeftClick(e, album, index) {
        if (detailsAlbum == album) {
            unselectAlbum(index);
        } else {
            detailsAlbum = album;
            const rect = e.currentTarget.getBoundingClientRect();
            console.log("RECT", rect);
            const x = rect.left;
            const y = rect.top;
            detailsPosition = { x, y, width: rect.width, height: rect.height };
            maskPosition = {
                x: e.currentTarget.offsetLeft,
                y: y - 35,
                width: rect.width,
                height: rect.height,
            };
            console.log(
                "maskPosition",
                e.currentTarget.clientTop,
                e.currentTarget.offsetTop,
                e.currentTarget.scrollTop,
            );
            detailsAlbumTracks = await getAlbumTracks(album);
            detailsAlbumIndex = index;
        }
    }

    function unselectAlbum(index?: number) {
        detailsAlbum = null;
        detailsAlbumTracks = null;
    }

    function scrollToCurrentAlbum() {
        virtualList.scrollToIndex = null;

        const index = activeAlbums.findIndex(
            (album) => album.id === currentAlbum.id,
        );

        setTimeout(() => {
            virtualList.scrollToIndex = Math.ceil(index / columnCount);
        }, 5);
    }

    function updateInView(offset = lastOffset) {
        const { clientHeight } = container;
        lastOffset = offset;
        isCurrentAlbumInView =
            offset < currentAlbumOffset &&
            currentAlbumOffset < offset + clientHeight;
    }

    const resizeObserver = new ResizeObserver(debounce(onResize, 5));
    onMount(() => {
        isInit = false;

        albums.subscribe(() => {
            onResize();
        });

        resizeObserver.observe(container);

        // This callback cleans up the observer
        return () => {
            resizeObserver.unobserve(container);
        };
    });
</script>

<AlbumMenu
    bind:this={albumMenu}
    onClose={() => {
        highlightedAlbum = null;
    }}
/>

<div class="albums-container" bind:this={container}>
    {#if isLoading}
        <!-- <div
            class="loading"
            out:fade={{ duration: 90, easing: cubicInOut }}
        >
            <p>💿 one sec...</p>
        </div> -->
    {:else}
        <div class="grid-container" class:details-open={detailsAlbum}>
            {#if detailsAlbum}
                <div
                    class="details-bg"
                    transition:fade={{ duration: 200 }}
                    style="
                        --x: {maskPosition.x}px;
                        --y: {maskPosition.y}px;
                        --w: {maskPosition.width}px;
                        --h: {maskPosition.height}px;
                    "
                />
                <div
                    class="details"
                    in:fade={{
                        duration: 150,
                        easing: cubicOut,
                    }}
                    out:fade={{
                        duration: 150,
                        easing: cubicInOut,
                    }}
                >
                    <AlbumDetails
                        album={detailsAlbum}
                        anchorTopLeft={detailsPosition}
                        tracks={detailsAlbumTracks}
                        onUnselect={() => unselectAlbum(detailsAlbumIndex)}
                        containerWidth={width}
                    />
                </div>
            {/if}
            <div class="albums-list" bind:this={gridContainer}>
                <VirtualList
                    bind:this={virtualList}
                    width={"100%"}
                    height={height || "100%"}
                    itemCount={rowCount}
                    itemSize={itemSizes}
                    scrollToAlignment="start"
                    scrollToBehaviour="smooth"
                    on:afterScroll={debounce(onAfterScroll, 20)}
                >
                    <div
                        slot="item"
                        let:index
                        let:style
                        {style}
                        class="grid-row"
                    >
                        {#if index === 0 || index + 1 === rowCount}
                            <div></div>
                        {:else}
                            {#each Array(columnCount) as _, col (col)}
                                {@const albumIdx =
                                    (index - 1) * columnCount + col}

                                {@const album =
                                    albumIdx < activeAlbums.length &&
                                    activeAlbums[albumIdx]}
                                {#if album}
                                    <div
                                        on:contextmenu|preventDefault={(e) =>
                                            onRightClick(e, album)}
                                        on:click|preventDefault={(e) =>
                                            onLeftClick(e, album, albumIdx)}
                                        data-album={album.id}
                                        style="width: {columnWidth}px;"
                                        class:details-album={detailsAlbum ===
                                            album}
                                    >
                                        <AlbumItem
                                            {album}
                                            highlighted={highlightedAlbum ===
                                                album.id}
                                            {showInfo}
                                        />
                                    </div>
                                {:else}
                                    <div style="width: {columnWidth}px" />
                                {/if}
                            {/each}
                        {/if}
                    </div>
                </VirtualList>
            </div>
        </div>
    {/if}
    {#if $uiView === "albums" && $isPlaying && currentAlbum && !isCurrentAlbumInView}
        <ScrollTo equalizer={true} on:click={scrollToCurrentAlbum}>
            {$LL.albums.scrollToNowPlaying()}
        </ScrollTo>
    {/if}

    <ShadowGradient type="top" />
    <ShadowGradient type="bottom" />
</div>

<style lang="scss">
    .albums-container {
        position: relative;
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 1fr;
        row-gap: 5px;
        border-radius: 5px;
        box-sizing: border-box;
        overflow: hidden;
    }
    .grid-container {
        display: grid;
        height: 100%;
        width: 100%;
        position: relative;
        grid-template-rows: 1fr;
        grid-template-columns: 1fr;
        border-bottom-left-radius: 5px;
        border-bottom-right-radius: 5px;
        background-color: var(--panel-background);

        > .details-bg {
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            z-index: 1;
            background-color: var(--popup-backdrop);
            backdrop-filter: blur(1px);

            /* Two layers:
       1. full white (visible)
       2. black (hole) */
            mask-image: linear-gradient(white 0%, black 100%),
                radial-gradient(
                    white 0%,
                    white 20%,
                    white 35%,
                    rgba(0, 0, 0, 0.2) 48%,
                    transparent 65%
                );

            mask-size:
                100% 100%,
                calc(var(--w) * 2) calc(var(--h) * 2);
            mask-position:
                0 0,
                calc(var(--x) - var(--w) / 2) calc(var(--y) - var(--h) / 1.7);
            mask-composite: exclude;
            mask-repeat: no-repeat;
        }
        > .details {
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            z-index: 2;
            grid-row: 1;
        }

        .albums-list {
            grid-row: 2;
            min-height: 0; /* THIS IS IMPORTANT in CSS grid */
            height: 100%;

            .details-album {
                z-index: 10000;
            }
        }
    }

    .grid-row {
        display: flex;
        justify-content: space-evenly;
        padding: 0 1em;
        overflow-x: clip;
        > div {
            padding: 5px;
        }
    }

    .loading {
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        position: absolute;
        margin: auto;
        display: flex;
        align-items: center;
        justify-content: center;
        p {
            opacity: 0.6;
        }
    }
</style>
