<script lang="ts">
    import { resolve } from "@tauri-apps/api/path";
    import { liveQuery, type IndexableTypeArray } from "dexie";
    import "iconify-icon";
    import type { Album, Song } from "../../App";
    import { db } from "../../data/db";
    import {
        albumPlaylist,
        currentSong,
        playlist,
        playlistIsAlbum,
        query,
        rightClickedAlbum
    } from "../../data/store";
    import AlbumItem from "../AlbumItem.svelte";
    import * as musicMetadata from "music-metadata-browser";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { lookForArt } from "../../data/LibraryImporter";
    import audioPlayer from "../AudioPlayer";
    import { fade, fly } from "svelte/transition";
    import { cubicInOut } from "svelte/easing";
    import md5 from "md5";
    import AlbumMenu from "../AlbumMenu.svelte";

    let isLoading = true;

    let cachedAlbums: Album[] = [];

    $: albums = liveQuery(async () => {
        let resultsArray: Album[] = [];
        resultsArray = await db.albums.orderBy("title").toArray();

        console.log("albums", resultsArray);

        isLoading = false;
        cachedAlbums = resultsArray;
        return resultsArray;
    });

    $: queriedAlbums =
        $albums && $query.query.length
            ? $albums.filter((a) =>
                  a.title
                      .toString()
                      .trim()
                      .toLowerCase()
                      .startsWith($query.query.trim().toLowerCase())
              )
            : [];

    $: {
        if ($albums && $albums?.length) {
            loadData();
        }
    }

    async function showCurrentlyPlayingAlbum() {
        if (!$currentSong) return;
        // Find the album currently playing
        const currentAlbum = await db.albums.get(
            md5(`${$currentSong.artist} - ${$currentSong.album}`)
        );
        let tracks = await db.songs
            .where("id")
            .anyOf(currentAlbum.tracksIds)
            .sortBy("trackNumber");

        $albumPlaylist = tracks;
        $playlistIsAlbum = true;
    }

    if ($playlist && !$playlistIsAlbum) {
        showCurrentlyPlayingAlbum();
    }

    let showSingles = false;

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

    let minWidth = 220;

    $: count = $query.query?.length ? queriedAlbums?.length : $albums?.length;

    $: displayTracks = minWidth > 300;

    let showAlbumMenu = false;
    let pos;
    let highlightedAlbum;

    function onRightClick(e, album, idx) {
        highlightedAlbum = album.id;
        $rightClickedAlbum = album;
        showAlbumMenu = true;
        pos = { x: e.clientX, y: e.clientY };
    }
</script>

<AlbumMenu
    bind:showMenu={showAlbumMenu}
    bind:pos
    onClose={() => {
        highlightedAlbum = null;
    }}
/>
<div class="grid-container">
    <div class="header">
        <h1>Albums</h1>
        {#if count}<p>{count} {count === 1 ? "album" : "albums"}</p>{/if}
        <label
            >show singles
            <input type="checkbox" bind:checked={showSingles} /></label
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

    {#if $playlist?.length && $playlistIsAlbum}
        <div class="now-playing" in:fly={{ duration: 200, x: -200 }}>
            <h1>Now playing</h1>
            <div class="album-info">
                <p>{$currentSong.album}</p>
                <p>{$currentSong.artist}</p>
            </div>
            <div class="tracks">
                {#each $albumPlaylist as track, idx}
                    <div
                        class="track"
                        class:playing={$currentSong.id === track.id}
                        on:click={() => {
                            audioPlayer.playSong(track);
                        }}
                    >
                        <p>{idx + 1}.</p>
                        <p>{track.title}</p>
                        {#if $currentSong.id === track.id}<iconify-icon
                                icon="heroicons-solid:volume-up"
                            />{/if}
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    {#if isLoading}
        <div class="loading" out:fade={{ duration: 90, easing: cubicInOut }}>
            <p>💿 one sec...</p>
        </div>
    {:else}
        {#if $query.query?.length && queriedAlbums?.length}
            <div
                class="grid"
                class:show={$query.query?.length}
                style="grid-template-columns: repeat(auto-fit, minmax({minWidth}px, 0.33fr));width: 100%;"
            >
                {#each queriedAlbums as album, idx (album.id)}
                    <div
                        on:contextmenu|preventDefault={(e) =>
                            onRightClick(e, album, idx)}
                    >
                        <AlbumItem
                            {album}
                            highlighted={highlightedAlbum === album.id}
                        />
                    </div>
                {/each}
            </div>
        {/if}
        <div
            class="grid"
            class:show={$albums && $query.query?.length === 0}
            style="grid-template-columns: repeat(auto-fit, minmax({minWidth}px, 0.33fr));width: 100%;"
        >
            {#if $albums}
                {#each $albums as album, idx (album.id)}
                    {#if (showSingles && album.trackCount > 0) || (!showSingles && album.trackCount > 1)}
                        <div
                            on:contextmenu|preventDefault={(e) =>
                                onRightClick(e, album, idx)}
                        >
                            <AlbumItem
                                {album}
                                highlighted={highlightedAlbum === album.id}
                            />
                        </div>
                    {/if}
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style lang="scss">
    .grid-container {
        overflow-x: hidden;
        overflow-y: auto;
        display: grid;
        height: 100%;
        grid-template-rows: auto 1fr;
        grid-template-columns: auto 1fr;
        background-color: #242026b3;
        position: relative;
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
    }
    .grid {
        grid-column: 2;
        grid-row: 2;
        width: 100%;
        display: none;
        height: fit-content;
        padding: 1em;
        gap: 10px;
        min-width: 0; // hack to make the grid respect wrap
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
    }

    label {
        display: flex;
        flex-direction: row-reverse;
        gap: 4px;
        align-items: center;
        color: #949c9f;
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
            background: #4a4d4e;
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
    .now-playing {
        h1 {
            font-family: "Snake";
            margin: 0 0.5em 0.5em;
            text-align: left;
        }
        display: flex;
        flex-direction: column;
        /* background: rgba(36, 34, 34, 0.943); */
        backdrop-filter: blur(8px);
        /* border: 1px solid rgba(128, 128, 128, 0.117); */
        border-right: 1px solid rgba(128, 128, 128, 0.117);
        /* box-shadow: -40px 20px 30px 30px rgba(0, 0, 0, 0.159); */
        /* box-shadow: 2px 2px 50px 100px rgba(72, 16, 128, 0.181); */

        color: white;
        bottom: 0;
        border-radius: 4px;
        z-index: 11;
        grid-column: 1;
        grid-row: 2;
        /* min-width: 180px;
        width: max-content; */
        height: 100vh;
        max-width: 300px;
        width: 300px;
        top: 0;
        overflow-y: auto;
        position: sticky;
        padding-top: 2em;
        padding-bottom: 2em;

        .album-info {
            display: flex;
            flex-direction: column;
            margin: 0 1em 1em;
            p {
                margin: 0;
                text-align: left;
                &:nth-child(2) {
                    opacity: 0.6;
                    position: relative;
                    &:before {
                        content: "by ";
                        opacity: 0.4;
                    }
                }
            }
        }
    }

    .tracks {
        display: flex;
        flex-direction: column;
        justify-content: space-evenly;
        /* background-color: rgb(36, 32, 38); */

        .track {
            padding: 0.35em 1em;
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 4px;
            user-select: none;
            border-radius: 3px;
            color: #bab5cb;
            cursor: default;
            p {
                margin: 0;
                text-align: left;
                &:nth-child(1) {
                    opacity: 0.4;
                    width: 20px;
                }
            }

            &:hover {
                background-color: rgba(0, 0, 0, 0.087);
            }

            &.playing {
                color: #dad4ed;
                font-weight: bold;
                iconify-icon {
                    color: #7f61dd;
                }
            }
            &:not(:nth-child(1)) {
                border-top: 1px solid rgba(255, 255, 255, 0.05);
            }
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
