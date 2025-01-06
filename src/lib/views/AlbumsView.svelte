<script lang="ts">
    import { liveQuery } from "dexie";
    import md5 from "md5";
    import { onMount } from "svelte";
    import { fly } from "svelte/transition";
    import type { Album } from "../../App";
    import { db } from "../../data/db";
    import {
        compressionSelected,
        current,
        isPlaying,
        query,
        queue,
        rightClickedAlbum,
        rightClickedTrack,
        rightClickedTracks,
        uiPreferences,
        uiView,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import AlbumItem from "../albums/AlbumItem.svelte";
    import AlbumMenu from "../albums/AlbumMenu.svelte";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import { path } from "@tauri-apps/api";
    import VirtualList from "svelte-tiny-virtual-list";
    import { debounce } from "lodash-es";

    const PADDING = 14;

    let activeAlbums: Album[] = [];
    let columnWidth = 0;
    let container: HTMLDivElement;
    let currentAlbum: Album;
    let currentAlbumOffset = 0;
    let highlightedAlbum;
    let isCurrentAlbumInView = false;
    let isInit = true;
    let isLoading = true;
    let itemSizes = [];
    let lastCount = 0;
    let lastOffset = 0;
    let position;
    let rowCount = 0;
    let showAlbumMenu = false;
    let virtualList;

    $: albums = liveQuery(async () => {
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
        rowCount = Math.ceil(activeAlbums.length / columnCount) + 2;

        onResize();

        const remainder = activeAlbums.length % columnCount;
        lastCount = remainder === 0 ? columnCount : remainder;
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

        const index = activeAlbums.findIndex(
            (album) => album.id === currentAlbum.id,
        );

        currentAlbumOffset = Math.ceil(index / columnCount) * 225 + PADDING;

        return true;
    }

    function onAfterScroll(e) {
        if (currentAlbum && container) {
            const { offset } = e.detail;

            updateInView(offset);
        }
    }

    let height = 0;
    function onResize() {
        if (!container) {
            return;
        }
        height = container?.clientHeight;
        const contentWidth = getContentWidth(container) - PADDING - PADDING;
        const count = Math.floor(contentWidth / minWidth);
        const remaining = contentWidth - count * minWidth;
        const perColumn = Math.floor(remaining / count);
        const max = Math.floor(contentWidth * 0.1);

        if (minWidth + perColumn > max) {
            columnWidth = Math.max(max, minWidth + Math.floor(perColumn / 2));
        } else {
            columnWidth = minWidth + perColumn;
        }
        columnCount = count;

        const size = Math.floor(columnWidth + (showInfo ? 55 : 0));

        itemSizes = Array(rowCount).fill(size);
        itemSizes[0] = PADDING;
        itemSizes[itemSizes.length - 1] = PADDING;
    }

    async function onRightClick(e, album) {
        highlightedAlbum = album.id;
        $rightClickedAlbum = album;
        const tracks = (await db.songs.bulkGet(album.tracksIds)).sort(
            (a, b) => a.trackNumber - b.trackNumber,
        );
        $rightClickedTrack = null;
        $rightClickedTracks = tracks;
        showAlbumMenu = true;
        position = { x: e.clientX, y: e.clientY };
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
    });
</script>

<AlbumMenu
    bind:showMenu={showAlbumMenu}
    bind:position
    onClose={() => {
        highlightedAlbum = null;
    }}
/>

<svelte:window on:resize={debounce(onResize, 30)} />
<div class="albums-container">
    {#if isLoading}
        <!-- <div
            class="loading"
            out:fade={{ duration: 90, easing: cubicInOut }}
        >
            <p>💿 one sec...</p>
        </div> -->
    {:else}
        <div class="grid-container" bind:this={container}>
            <VirtualList
                bind:this={virtualList}
                width="100%"
                height={height || "100%"}
                itemCount={rowCount}
                itemSize={itemSizes}
                scrollToAlignment="center"
                scrollToBehaviour="smooth"
                on:afterScroll={debounce(onAfterScroll, 20)}
            >
                <div
                    slot="item"
                    let:index
                    let:style
                    {style}
                    class="grid-row"
                    class:last-row={index + 2 === rowCount}
                >
                    {#if index === 0 || index + 1 === rowCount}
                        <div></div>
                    {:else}
                        {#each Array(columnCount) as _, col (col)}
                            {@const albumIdx = (index - 1) * columnCount + col}
                            {@const album =
                                albumIdx < activeAlbums.length &&
                                activeAlbums[albumIdx]}
                            {#if album}
                                <div
                                    on:contextmenu|preventDefault={(e) =>
                                        onRightClick(e, album)}
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
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
            in:fly={{ duration: 150, y: 30 }}
            out:fly={{ duration: 150, y: 30 }}
            class="scroll-now-playing"
            on:click={scrollToCurrentAlbum}
        >
            <div class="eq">
                <span class="eq1" />
                <span class="eq2" />
                <span class="eq3" />
            </div>
            <p>{$LL.albums.scrollToNowPlaying()}</p>
        </div>
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
        margin: 5px 5px 0 0;
        row-gap: 5px;
        border-radius: 5px;
        box-sizing: border-box;
        overflow: hidden;
        border-top: 0.7px solid var(--panel-secondary-border-accent);
        border-bottom: 0.7px solid var(--panel-secondary-border-main);
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
        border-left: 0.7px solid var(--panel-secondary-border-main);
        border-bottom: 0.7px solid var(--panel-secondary-border-main);
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
        /* background-color: rgb(34, 33, 33); */
        /* background-image: url("images/textures/soft-wallpaper.png"); */
        /* background-repeat: repeat; */
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

    .scroll-now-playing {
        position: absolute;
        bottom: 0.5em;
        grid-column: 1 / 4;
        left: 0;
        right: 0;
        padding: 0.5em 1em;
        border-radius: 10px;
        background-color: #1b1b1c;
        border: 1px solid rgb(58, 56, 56);
        box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.834);
        color: white;
        margin: auto;
        width: fit-content;
        z-index: 11;
        display: inline-flex;
        align-items: center;
        gap: 10px;
        cursor: default;
        user-select: none;

        @media only screen and (max-width: 522px) {
            display: none;
        }
        &:hover {
            background-color: #1f1f21;
            border: 1px solid rgb(101, 98, 98);
            box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.934);
        }
        &:active {
            background-color: #2a2a2d;
            border: 2px solid rgb(101, 98, 98);
            box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.934);
        }

        .eq {
            width: 15px;
            padding: 0.5em;
            position: relative;

            span {
                display: inline-block;
                width: 3px;
                background-color: #ddd;
                position: absolute;
                bottom: 0;
            }

            .eq1 {
                height: 13px;
                left: 0;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0s;
            }

            .eq2 {
                height: 15px;
                left: 6px;
                animation-name: talleq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.17s;
            }

            .eq3 {
                height: 13px;
                left: 12px;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.34s;
            }
        }
        p {
            margin: 0;
        }
    }

    @keyframes shorteq {
        0% {
            height: 10px;
        }
        50% {
            height: 5px;
        }
        100% {
            height: 10px;
        }
    }
    @keyframes talleq {
        0% {
            height: 15px;
        }
        50% {
            height: 8px;
        }
        100% {
            height: 15px;
        }
    }
</style>
