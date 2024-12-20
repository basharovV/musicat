<script lang="ts">
    import { liveQuery } from "dexie";
    import md5 from "md5";
    import { onMount } from "svelte";
    import { fly } from "svelte/transition";
    import type { Album } from "../../App";
    import { db } from "../../data/db";
    import {
        compressionSelected,
        currentSong,
        isPlaying,
        playlist,
        query,
        rightClickedAlbum,
        rightClickedTrack,
        rightClickedTracks,
        uiPreferences,
        uiView
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import AlbumItem from "../albums/AlbumItem.svelte";
    import AlbumMenu from "../albums/AlbumMenu.svelte";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import { path } from "@tauri-apps/api";

    let isLoading = true;
    let isVisible = false;
    let isInit = true;

    $: albums = liveQuery(async () => {
        let albums = await db.albums.toArray();

        if ($compressionSelected === "lossless") {
            albums = albums.filter(({title, lossless}) => title.length && lossless);
        } else if ($compressionSelected === "lossy") {
            albums = albums.filter(({title, lossless}) => title.length && !lossless);
        } else {
            albums = albums.filter(({title}) => title.length);
        }

        if ($uiPreferences.albumsViewSortBy === 'title') {
            albums.sort((a, b) => {
                if (a.title < b.title) return -1;
                if (a.title > b.title) return 1;
                return 0;
            });
        } else if ($uiPreferences.albumsViewSortBy === 'artist') {
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

    $: queriedAlbums =
        $albums && $query.query.length
            ? $albums.filter(
                  (a) =>
                      a.artist
                          .toLowerCase()
                          .includes($query.query.toLowerCase()) ||
                      a.title.includes($query.query.toLowerCase())
              )
            : [];

    let isCurrentAlbumInView = false;
    let currentAlbum: Album;
    let currentAlbumElement: HTMLDivElement;

    async function showCurrentlyPlayingAlbum() {
        if (!$currentSong) return;
        
        if (await updatePlayingAlbum()) {
            currentAlbumElement?.scrollIntoView({
                block: "center",
                behavior: "instant"
            });

            isVisible = true;
            isInit = false;
        }
    }
    
    async function updatePlayingAlbum() {
        // Strip the song from album path
        const albumPath = await path.dirname($currentSong.path)
        // Find the album currently playing
        currentAlbum = await db.albums.get(
            md5(`${albumPath} - ${$currentSong.album}`.toLowerCase())
        );
        if (!currentAlbum) return false;
        
        currentAlbumElement = document.querySelector(
            `[data-album='${currentAlbum.id}']`
        );
        
        return true;
    }

    $: if (isInit && $playlist && $currentSong) {
        showCurrentlyPlayingAlbum();
    } else {
        isVisible = true;
        isInit = false;
    }
    
    $: if (container && $currentSong?.album.toLowerCase() !== currentAlbum?.title.toLowerCase()) {
        if (updatePlayingAlbum()) {
            onScroll();
        }
    }

    $: minWidth = $uiPreferences.albumsViewGridSize;
    $: showSingles = $uiPreferences.albumsViewShowSingles;
    $: showInfo = $uiPreferences.albumsViewShowInfo;

    let showAlbumMenu = false;
    let pos;
    let highlightedAlbum;

    async function onRightClick(e, album, idx) {
        highlightedAlbum = album.id;
        $rightClickedAlbum = album;
        const tracks = (await db.songs.bulkGet(album.tracksIds)).sort((a, b) => a.trackNumber - b.trackNumber);
        $rightClickedTrack = null;
        $rightClickedTracks = tracks;
        showAlbumMenu = true;
        pos = { x: e.clientX, y: e.clientY };
    }

    let container: HTMLDivElement;

    function onScroll() {
        const containerRect = container.getBoundingClientRect();

        if (currentAlbumElement && containerRect) {
            isCurrentAlbumInView =
                currentAlbumElement.offsetTop > container.scrollTop &&
                currentAlbumElement.offsetTop <
                    container.scrollTop + containerRect.height;
        }
    }

    function scrollToCurrentAlbum() {
        currentAlbumElement?.scrollIntoView({
            block: "center",
            behavior: "smooth"
        });
    }
    onMount(() => {
        isInit = false;
    });
</script>

<AlbumMenu
    bind:showMenu={showAlbumMenu}
    bind:pos
    onClose={() => {
        highlightedAlbum = null;
    }}
/>

<div class="albums-container">
    <div class="grid-container" on:scroll={onScroll} bind:this={container}>
        {#if isLoading}
            <!-- <div
                class="loading"
                out:fade={{ duration: 90, easing: cubicInOut }}
            >
                <p>💿 one sec...</p>
            </div> -->
        {:else}
            {#if $query.query?.length && queriedAlbums?.length}
                <div
                    class="grid"
                    class:show={$query.query?.length}
                    class:visible={isVisible}
                    style="grid-template-columns: repeat(auto-fit, minmax({minWidth}px, 0.1fr));width: 100%;"
                >
                    {#each queriedAlbums as album, idx (album.id)}
                        <div
                            on:contextmenu|preventDefault={(e) =>
                                onRightClick(e, album, idx)}
                            data-album={album.id}
                        >
                            <AlbumItem
                                {album}
                                highlighted={highlightedAlbum === album.id}
                                {showInfo}
                            />
                        </div>
                    {/each}
                </div>
            {/if}
            <div
                class="grid"
                class:show={$albums && $query.query?.length === 0}
                class:visible={isVisible}
                style="grid-template-columns: repeat(auto-fit, minmax({minWidth}px, 0.1fr));width: 100%;"
            >
                {#if $albums}
                    {#each $albums as album, idx (album.id)}
                        {#if (showSingles && album.tracksIds.length > 0) || (!showSingles && album.tracksIds.length > 1)}
                            <div
                                on:contextmenu|preventDefault={(e) =>
                                    onRightClick(e, album, idx)}
                                data-album={album.id}
                            >
                                <AlbumItem
                                    {album}
                                    highlighted={highlightedAlbum === album.id}
                                    {showInfo}
                                />
                            </div>
                        {/if}
                    {/each}
                {/if}
            </div>
        {/if}

        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
    </div>
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
        /* border: 0.7px solid #ffffff0b; */
        border-top: 0.7px solid #ffffff19;
        border-bottom: 0.7px solid #ffffff2a;
    }
    .grid-container {
        overflow-x: hidden;
        overflow-y: auto;
        display: grid;
        height: 100%;
        width: 100%;
        position: relative;
        grid-template-rows: auto 1fr;
        grid-template-columns: 1fr;
        border-bottom-left-radius: 5px;
        border-bottom-right-radius: 5px;
        border-left: 0.7px solid #ffffff2a;
        border-bottom: 0.7px solid #ffffff2a;
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
