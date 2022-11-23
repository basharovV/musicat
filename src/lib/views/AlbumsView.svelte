<script lang="ts">
    import { resolve } from "@tauri-apps/api/path";
    import { liveQuery, type IndexableTypeArray } from "dexie";
    import "iconify-icon";
    import type { Song } from "../../App";
    import { db } from "../../data/db";
    import {
        albumPlaylist,
        currentSong,
        playlist,
        playlistIsAlbum,
        query
    } from "../../data/store";
    import AlbumItem from "../AlbumItem.svelte";
    import * as musicMetadata from "music-metadata-browser";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { lookForArt } from "../../data/LibraryImporter";
    import audioPlayer from "../AudioPlayer";

    let isLoading = true;

    let cachedAlbums: IndexableTypeArray = [];

    $: albums = liveQuery(async () => {
        let resultsArray: IndexableTypeArray = [];
        resultsArray = await db.songs.orderBy("album").uniqueKeys();

        console.log("albums", resultsArray);

        isLoading = false;
        cachedAlbums = resultsArray;
        return resultsArray;
    });

    $: queriedAlbums =
        $albums && $query.query.length
            ? $albums.filter((a) =>
                  a
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

    let showSingles = false;

    async function loadData() {
        await getAlbumTrack($albums);
    }

    let albumsData: {
        [key: string]: {
            tracks: Song[];
            artworkFormat: string;
            artworkSrc: string;
        };
    } = null;

    async function addArtwork(album, track) {
        if (albumsData[album.toString()].tracks?.length) {
            const result = await getArtwork(track);
            albumsData[album.toString()].artworkFormat = result.artworkFormat;
            albumsData[album.toString()].artworkSrc = result.artworkSrc;
        }
    }

    async function getAlbumTrack(albums: IndexableTypeArray) {
        console.log("albumdata", albums);
        await Promise.all(
            albums.map(
                (album) =>
                    new Promise<void>(async (resolve) => {
                        if (albumsData === null) albumsData = {};
                        if (albumsData[album.toString()] === undefined) {
                            albumsData[album.toString()] = {
                                tracks: [],
                                artworkFormat: null,
                                artworkSrc: null
                            };
                        }
                        albumsData[album.toString()].tracks = await db.songs
                            .where("album")
                            .equals(album)
                            .sortBy("trackNumber");
                        addArtwork(
                            album,
                            albumsData[album.toString()].tracks[0]
                        );
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
</script>

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
        <div class="now-playing">
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

    {#if $query.query?.length && queriedAlbums?.length}
        <div
            class="grid"
            class:show={$query.query?.length}
            style="grid-template-columns: repeat(auto-fit, minmax({minWidth}px, 0.33fr));width: 100%;"
        >
            {#each queriedAlbums as album (album)}
                <AlbumItem
                    {album}
                    tracks={albumsData && albumsData[album]
                        ? albumsData[album].tracks
                        : null}
                    artworkSrc={albumsData && albumsData[album]
                        ? albumsData[album].artworkSrc
                        : null}
                    artworkFormat={albumsData && albumsData[album]
                        ? albumsData[album].artworkFormat
                        : null}
                />
            {/each}
        </div>
    {/if}
    <div
        class="grid"
        class:show={$albums && $query.query?.length === 0}
        style="grid-template-columns: repeat(auto-fit, minmax({minWidth}px, 0.33fr));width: 100%;"
    >
        {#if $albums}
            {#each $albums as album (album)}
                {#if !(albumsData && albumsData[album]?.tracks?.length) || (albumsData && showSingles && albumsData[album].tracks.length > 0) || (albumsData && !showSingles && albumsData[album].tracks.length > 1)}
                    <AlbumItem
                        {album}
                        tracks={albumsData && albumsData[album]
                            ? albumsData[album].tracks
                            : null}
                        artworkSrc={albumsData && albumsData[album]
                            ? albumsData[album].artworkSrc
                            : null}
                        artworkFormat={albumsData && albumsData[album]
                            ? albumsData[album].artworkFormat
                            : null}
                        {displayTracks}
                    />
                {/if}
            {/each}
        {/if}
    </div>
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
        display: none;
        height: fit-content;
        padding: 1em;
        gap: 10px;
        /* background-color: rgb(34, 33, 33); */
        /* background-image: url("images/textures/soft-wallpaper.png"); */
        /* background-repeat: repeat; */

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
        display: flex;
        flex-direction: column;
        /* background: rgba(36, 34, 34, 0.943); */
        backdrop-filter: blur(8px);
        /* border: 1px solid rgba(128, 128, 128, 0.117); */
        /* border: 1px solid rgba(128, 128, 128, 0.117); */
        /* box-shadow: -40px 20px 30px 30px rgba(0, 0, 0, 0.159); */
        /* box-shadow: 2px 2px 50px 100px rgba(72, 16, 128, 0.181); */

        color: white;
        right: 3em;
        bottom: 1em;
        border-radius: 4px;
        z-index: 11;
        grid-column: 1;
        grid-row: 2;
        /* min-width: 180px;
        width: max-content; */
        margin: 1em;
        max-height: 80vh;
        max-width: 300px;
        width: 300px;
        overflow-y: auto;
        position: sticky;
        padding-bottom: 2em;
        top: 1em;

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
</style>
