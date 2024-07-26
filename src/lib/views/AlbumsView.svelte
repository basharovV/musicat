<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { liveQuery } from "dexie";
    import md5 from "md5";
    import * as musicMetadata from "music-metadata-browser";
    import { onMount } from "svelte";
    import { fly } from "svelte/transition";
    import type { Album, Song } from "../../App";
    import { lookForArt } from "../../data/LibraryImporter";
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
        uiView
    } from "../../data/store";
    import AlbumItem from "../albums/AlbumItem.svelte";
    import AlbumMenu from "../albums/AlbumMenu.svelte";
    import Dropdown from "../ui/Dropdown.svelte";
    import ShadowGradient from "../ui/ShadowGradient.svelte";

    let isLoading = true;
    let isVisible = false;
    let isInit = true;

    let cachedAlbums: Album[] = [];

    $: albums = liveQuery(async () => {
        let resultsArray: Album[] = [];
        resultsArray = await db.albums.orderBy(orderBy.value).toArray();

        isLoading = false;
        cachedAlbums = resultsArray;
        return resultsArray.filter((a) => {
            const hasTitle = a.title.length;
            let compressionFilterMatch = true;
            if ($compressionSelected === "lossless") {
                compressionFilterMatch = a.lossless;
            } else if ($compressionSelected === "lossy") {
                compressionFilterMatch = a.lossless === false;
            }
            return hasTitle && compressionFilterMatch;
        });
    });

    $: queriedAlbums =
        $albums && $query.query.length
            ? $albums.filter(
                  (a) =>
                      a.artist
                          .toLowerCase()
                          .includes($query.query.toLowerCase()) ||
                      a.title.toLowerCase().includes($query.query.toLowerCase())
              )
            : [];

    $: {
        if ($albums && $albums?.length) {
            loadData();
        }
    }

    let isCurrentAlbumInView = false;
    let currentAlbum: Album;
    let currentAlbumElement: HTMLDivElement;

    async function showCurrentlyPlayingAlbum() {
        if (!$currentSong) return;

        // Find the album currently playing
        currentAlbum = await db.albums.get(
            md5(`${$currentSong.artist} - ${$currentSong.album}`.toLowerCase())
        );
        if (!currentAlbum) return;
        // $albumPlaylist = tracks;
        // $playlistIsAlbum = true;

        // Scroll to album
        currentAlbumElement = document.querySelector(
            `[data-album='${currentAlbum.id}']`
        );

        currentAlbumElement?.scrollIntoView({
            block: "center",
            behavior: "instant"
        });

        isVisible = true;
        isInit = false;
    }

    $: if (isInit && $playlist && $currentSong) {
        showCurrentlyPlayingAlbum();
    } else {
        isVisible = true;
        isInit = false;
    }

    let showSingles = false;
    let showInfo = true;

    async function loadData() {
        // await getAlbumTrack($albums);
    }

    let albumsData: {
        [key: string]: {
            album: Album;
            tracks: Song[];
            artworkFormat: string;
            artworkSrc: string;
        };
    } = null;

    async function addArtwork(albumId: string, track) {
        if (track) {
            const result = await getArtwork(track);
            albumsData[albumId].artworkFormat = result.artworkFormat;
            albumsData[albumId].artworkSrc = result.artworkSrc;
        }
    }

    async function getAlbumTrack(albums: Album[]) {
        console.log("albumdata", albums);
        await Promise.all(
            albums.map(
                (album) =>
                    new Promise<void>(async (resolve) => {
                        if (albumsData === null) albumsData = {};
                        if (albumsData[album.id] === undefined) {
                            albumsData[album.id] = {
                                album,
                                tracks: [],
                                artworkFormat: null,
                                artworkSrc: null
                            };
                        }
                        const firstTrack = await db.songs.get(
                            album.tracksIds[0]
                        );

                        addArtwork(album.id, firstTrack);
                        resolve();
                    })
            )
        );
    }

    async function getArtwork(firstTrack: Song) {
        let artworkFormat;
        let artworkBuffer;
        let artworkSrc;
        const metadata = await musicMetadata.fetchFromUrl(
            convertFileSrc(firstTrack.path)
        );
        if (metadata.common.picture?.length) {
            artworkFormat = metadata.common.picture[0].format;
            artworkBuffer = metadata.common.picture[0].data;
            artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                "base64"
            )}`;
        } else {
            artworkSrc = null;
            const artwork = await lookForArt(firstTrack.path, firstTrack.file);
            if (artwork) {
                artworkSrc = artwork.artworkSrc;
                artworkFormat = artwork.artworkFormat;
            }
        }
        return {
            artworkFormat,
            artworkSrc
        };
    }

    let minWidth = 200;

    $: count = $query.query?.length ? queriedAlbums?.length : $albums?.length;

    $: displayTracks = minWidth > 300;

    let showAlbumMenu = false;
    let pos;
    let highlightedAlbum;

    async function onRightClick(e, album, idx) {
        highlightedAlbum = album.id;
        $rightClickedAlbum = album;
        const tracks = await db.songs.bulkGet(album.tracksIds);
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

    const fields = [
        {
            value: "title",
            label: "Title"
        },
        {
            value: "artist",
            label: "Artist"
        },
        {
            value: "year",
            label: "Year"
        }
    ];

    let orderBy = fields[0];

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
        <div class="header">
            <h1>Albums</h1>
            <!-- {#if count}<p>{count} {count === 1 ? "album" : "albums"}</p>{/if} -->
            <div class="options">
                <div class="order-by">
                    <p>order by</p>
                    <Dropdown options={fields} bind:selected={orderBy} />
                </div>
                <label
                    >show singles
                    <input type="checkbox" bind:checked={showSingles} /></label
                >
                <label
                    >show info
                    <input type="checkbox" bind:checked={showInfo} /></label
                >
                <label
                    >grid size
                    <input
                        type="range"
                        min={100}
                        max={400}
                        bind:value={minWidth}
                    /></label
                >
            </div>
        </div>

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
                        {#if (showSingles && album.trackCount > 0) || (!showSingles && album.trackCount > 1)}
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
            <p>Scroll to Now playing</p>
        </div>
    {/if}

    <!-- <ShadowGradient type="top" /> -->
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

    .header {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: flex-end;
        padding: 1em;
        gap: 20px;
        grid-column: 1 /3;
        grid-row: 1;

        h1 {
            margin: 0;
            font-family: "Snake";
            flex-grow: 1;
            text-align: left;
            margin-left: 10px;
            padding-left: 5px;
            font-size: 4em;
            opacity: 0.3;
        }

        .options {
            display: flex;
            margin-right: 5px;
            gap: 20px;

            .order-by {
                display: flex;
                gap: 3px;
                color: var(--text-secondary);
            }
        }
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

    label {
        display: flex;
        flex-direction: row-reverse;
        gap: 4px;
        align-items: center;
        color: var(--text-secondary);
    }
    input[type="checkbox"] {
        padding: 0;
        margin: 0;
    }
    input[type="range"] {
        appearance: none;
        outline: none;
        border: none;
        box-shadow: none;
        max-width: 100px;
        &::-webkit-slider-thumb {
            appearance: none;
            background-color: rgb(132, 175, 166);
            border-radius: 2px;
            color: red;
            width: 10px;
            height: 10px;
            top: -2.5px;
            left: 0;
            position: relative;
        }
        &::-webkit-slider-runnable-track {
            background-color: var(--icon-secondary);
            appearance: none;
            border-radius: 10px;
            outline: none;
            height: 4px;
        }
        ::-moz-range-track {
            background: #ade8ff;
            height: 4px;
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
