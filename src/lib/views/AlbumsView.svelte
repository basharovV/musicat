<script lang="ts">
    import { liveQuery } from "dexie";
    import md5 from "md5";
    import { onMount } from "svelte";
    import type { Album, Song } from "../../App";
    import { db } from "../../data/db";
    import {
        compressionSelected,
        current,
        importStatus,
        isPlaying,
        query,
        queue,
        selectedSmartQuery,
        uiPreferences,
        uiView,
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
    import SmartQuery from "../smart-query/Query";
    import BuiltInQueries from "../../data/SmartQueries";
    import ImportPlaceholder from "../library/ImportPlaceholder.svelte";

    const PADDING = 14;

    let activeAlbums: Album[] = [];
    let albumMenu: AlbumMenu;
    let columnWidth = 0;
    let container: HTMLDivElement;
    let currentAlbum: Album;
    let currentAlbumOffset = 0;
    let detailsAlbum: Album = null;
    let detailsAlbumHeight = 0;
    let detailsAlbumIndex = -1;
    let detailsAlbumRow = -1;
    let detailsAlbumTracks: Song[] = null;
    let highlightedAlbum;
    let isCurrentAlbumInView = false;
    let isInit = true;
    let itemSizes = [];
    let lastOffset = 0;
    let rowCount = 0;
    let rowHeight = 0;
    let virtualList;

    $: albums = liveQuery(async () => {
        let albums: Album[];

        if ($uiView === "smart-query:icon") {
            let songs: Song[];

            if ($selectedSmartQuery.startsWith("~usq:")) {
                // Run the query from the user-built blocks
                const query = await SmartQuery.loadWithUQI($selectedSmartQuery);
                songs = await query.run();
            } else {
                // Run the query from built-in functions
                songs = await BuiltInQueries[$selectedSmartQuery].run();
            }

            const byAlbumIds = {};

            for (const song of songs) {
                if (song.albumId) {
                    if (byAlbumIds[song.albumId]) {
                        byAlbumIds[song.albumId].push(song);
                    } else {
                        byAlbumIds[song.albumId] = [song];
                    }
                }
            }

            albums = await db.albums.bulkGet(Object.keys(byAlbumIds));

            albums = albums.map((album) => ({
                ...album,
                tracksIds: byAlbumIds[album.id].map(({ id }) => id),
            }));
        } else {
            albums = await db.albums.toArray();

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

        return albums;
    });

    $: if (isInit && $queue && $current?.song) {
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
        if ($query.query.length) {
            queriedAlbums = $albums.filter(
                (a) =>
                    a.artist
                        .toLowerCase()
                        .includes($query.query.toLowerCase()) ||
                    a.title.includes($query.query.toLowerCase()),
            );

            activeAlbums = queriedAlbums;
        } else {
            activeAlbums = $albums;
        }

        if (!showSingles) {
            activeAlbums = activeAlbums.filter((a) => a.tracksIds.length > 1);
        }

        onResize();
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
        // Strip the song from album path
        const albumPath = await path.dirname($current?.song?.path);
        // Find the album currently playing
        currentAlbum = await db.albums.get(
            md5(`${albumPath} - ${$current?.song?.album}`.toLowerCase()),
        );
        if (!currentAlbum) return false;

        updatePlayingAlbumOffset();

        return !!virtualList;
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

    function onResize() {
        if (!container) {
            return;
        }

        if (columnCount) {
            rowCount = Math.ceil(activeAlbums.length / columnCount) + 2;

            if (detailsAlbum) {
                const index = activeAlbums.findIndex(
                    (album) => album === detailsAlbum,
                );

                if (index === -1) {
                    unselectAlbum();
                } else {
                    rowCount += 1;
                    detailsAlbumIndex = index;
                }
            }
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

        rowHeight = Math.floor(columnWidth + (showInfo ? 55 : 0));

        itemSizes = Array(rowCount).fill(rowHeight);
        itemSizes[0] = PADDING;
        itemSizes[itemSizes.length - 1] = PADDING;

        if (detailsAlbum) {
            const row = Math.floor(detailsAlbumIndex / count) + 2;

            itemSizes[row] = detailsAlbumHeight;
            detailsAlbumRow = row;
        }

        if (currentAlbum) {
            updatePlayingAlbumOffset();
            updateInView();
        }

        if (virtualList?.scrollToIndex) virtualList.scrollToIndex = null;
    }

    async function onRightClick(e, album) {
        if (e.button !== 2) {
            return;
        }

        highlightedAlbum = album.id;

        const songs = (await db.songs.bulkGet(album.tracksIds))
            // make sure that the song exist
            .filter((song) => song)
            // sort by track number
            .sort((a, b) => a.trackNumber - b.trackNumber);

        albumMenu.open(album, songs, { x: e.clientX, y: e.clientY });
    }

    async function onLeftClick(e, album, index) {
        if (detailsAlbum == album) {
            unselectAlbum(index);
        } else {
            const oldRow = detailsAlbumRow;

            detailsAlbum = album;
            detailsAlbumTracks = await db.songs.bulkGet(album.tracksIds);
            detailsAlbumIndex = index;
            detailsAlbumHeight = await getAlbumDetailsHeight(
                detailsAlbumTracks.length,
            );
            detailsAlbumRow = Math.floor(index / columnCount) + 2;

            if (oldRow >= 0) {
                var sizes = [...itemSizes];
                sizes.splice(oldRow, 1);
                sizes.splice(detailsAlbumRow, 0, detailsAlbumHeight);
                itemSizes = sizes;
            } else {
                itemSizes.splice(detailsAlbumRow, 0, detailsAlbumHeight);
                rowCount += 1;
            }
        }
    }

    function unselectAlbum(index?: number) {
        detailsAlbum = null;
        detailsAlbumTracks = null;
        detailsAlbumHeight = 0;
        detailsAlbumIndex = -1;
        detailsAlbumRow = -1;

        if (typeof index === "number") {
            itemSizes.splice(Math.floor(index / columnCount) + 2, 1);
            rowCount -= 1;
        }
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

    onMount(() => {
        isInit = false;

        albums.subscribe(() => {
            onResize();
        });

        const resizeObserver = new ResizeObserver(debounce(onResize, 5));

        resizeObserver.observe(container);

        // This callback cleans up the observer
        return () => resizeObserver.unobserve(container);
    });
</script>

<AlbumMenu
    bind:this={albumMenu}
    onClose={() => {
        highlightedAlbum = null;
    }}
/>

<div class="albums-container" bind:this={container}>
    {#if !$albums}
        <!-- <div
            class="loading"
            out:fade={{ duration: 90, easing: cubicInOut }}
        >
            <p>💿 one sec...</p>
        </div> -->
    {:else if ($importStatus.isImporting && $importStatus.backgroundImport === false) || ($albums.length === 0 && $query.query.length === 0 && !/^(smart-query|favourites|to-delete)/.test($uiView))}
        <ImportPlaceholder />
    {:else}
        <div class="grid-container">
            <VirtualList
                bind:this={virtualList}
                width={width || "100%"}
                height={height || "100%"}
                itemCount={rowCount}
                itemSize={itemSizes}
                scrollToAlignment="start"
                scrollToBehaviour="smooth"
                on:afterScroll={debounce(onAfterScroll, 20)}
            >
                <div slot="item" let:index let:style {style} class="grid-row">
                    {#if index === 0 || index + 1 === rowCount}
                        <div></div>
                    {:else if detailsAlbumRow === index}
                        <AlbumDetails
                            album={detailsAlbum}
                            tracks={detailsAlbumTracks}
                            onUnselect={() => unselectAlbum(detailsAlbumIndex)}
                        />
                    {:else}
                        {#each Array(columnCount) as _, col (col)}
                            {@const albumIdx =
                                detailsAlbumRow === -1 ||
                                index < detailsAlbumRow
                                    ? (index - 1) * columnCount + col
                                    : (index - 2) * columnCount + col}
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
                                    style="width: {columnWidth}px"
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
    }

    .grid {
        grid-column: 1;
        grid-row: 2;
        width: 100%;
        display: none;
        height: fit-content;
        padding: 1em;
        gap: 10px;
        min-width: 0; // hack to make the grid respect wrap
        visibility: hidden;
        opacity: 0;
        transition: opacity 0.15s cubic-bezier(0.455, 0.03, 0.515, 0.955);
        > div {
            position: relative;
            width: 100%;
        }

        &.show {
            display: grid;
        }
        &.visible {
            visibility: visible;
            opacity: 1;
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
