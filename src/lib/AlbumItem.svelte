<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import * as musicMetadata from "music-metadata-browser";
    import { onMount } from "svelte";
    import type { Song } from "../App";
    import { db } from "../data/db";
    import { lookForArt } from "../data/LibraryImporter";
    import { albumPlaylist, currentSong, isPlaying, playlist, playlistIsAlbum } from "../data/store";
    import audioPlayer from "./AudioPlayer";

    export let album: string; // to display album data
    export let tracks: Song[];
    export let artworkFormat;
    let artworkBuffer: Buffer;
    export let artworkSrc;
    export let displayTracks = false;

    $: firstTrack = tracks && tracks[0];

    let isHovered = false;
    function playPauseToggle() {
        if ($currentSong?.album === album) {
            if ($isPlaying) {
                audioPlayer.pause();
            } else {
                audioPlayer.play();
            }
        } else {
            if (tracks) audioPlayer.playSong(tracks[0]);
            $playlist = tracks;
            $albumPlaylist = tracks;
            $playlistIsAlbum = true;
        }
    }

    $: isPlayingCurrentAlbum = $currentSong?.album === album;
</script>

<div
    class="container"
    class:hovered={isHovered && artworkSrc && artworkFormat}
    class:playing={isPlayingCurrentAlbum}
>
    {#if artworkSrc && artworkFormat}
        <div class="cd-img"><img async src="images/cd-hq.webp" /></div>
    {/if}

    <div
        class="cd"
        on:mouseenter={() => {
            isHovered = true;
        }}
        on:mouseleave={() => {
            isHovered = false;
        }}
    >
        <div class="hinge" />
        <div class="artwork-container">
            <img
                class="texture"
                src="images/textures/soft-wallpaper.png"
            />
            <div class="artwork-frame">
                {#if artworkSrc && artworkFormat}
                    <img
                        alt="Artwork"
                        type={artworkFormat}
                        class="artwork"
                        src={artworkSrc}
                    />
                {:else}
                    <div class="artwork-placeholder">
                        <iconify-icon icon="mdi:music-clef-treble" />
                        <img
                            class="cd-placeholder"
                            src="images/cd-hq.png"
                        />
                        <!-- <small>No art</small> -->
                    </div>
                {/if}
                {#if isHovered || isPlayingCurrentAlbum}
                    <div class="play-button-container">
                        <iconify-icon
                            class={$isPlaying && isPlayingCurrentAlbum
                                ? "pause-button"
                                : "play-button"}
                            on:click={playPauseToggle}
                            icon={$isPlaying && isPlayingCurrentAlbum
                                ? "fe:pause"
                                : "fe:play"}
                        />
                    </div>
                {/if}
            </div>
        </div>
    </div>
    <p class="title">{album}</p>
    <p class="artist">{firstTrack?.artist}</p>
    <div class="info">
        <small>{firstTrack?.year}</small>
        <small>•</small>
        <small>{tracks?.length} tracks</small>
    </div>
</div>

<style lang="scss">
    .playing {
        .cd {
            z-index: 8;
            /* box-shadow: 2px 2px 30px 20px rgba(39, 0, 178, 0.181) !important; */
            box-shadow: 2px 2px 50px 100px rgba(72, 16, 128, 0.181) !important;
            .hinge {
                background-color: rgba(167, 164, 173, 0.078);
            }
        }

        .cd-img {
            transform: translate(15%, -10px) rotate(140deg) !important;
        }
        .title {
            background-color: #5123dd;
            border-radius: 4px;
            color: white;
        }
        z-index: 9;
    }

    .hovered:not(.playing) {
        .cd {
            z-index: 6;
        }

        .cd-img {
            transform: translate(12%, -10px) rotate(130deg) !important;
        }
        z-index: 8;
    }

    .hovered,
    .playing {
        .cd {
            box-shadow: 2px 2px 3px 2px rgba(0, 0, 0, 0.181);
        }

        .cd-img {
            filter: saturate(0.9);
            z-index: 0;
        }
    }

    .now-playing {
        position: absolute;
        top: -10px;
    }

    .playing {
        /* border: 1px solid #5123dd; */
    }
    .container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: flex-start;
        position: relative;
        &:hover {
        }
        * {
            cursor: default;
            user-select: none;
        }

        .info {
            display: flex;
            flex-direction: row;
            width: 100%;
            padding: 0 1em;
            gap: 4px;
            justify-content: center;
            align-items: center;
        }

        .title {
            margin-top: 10px;
            padding: 0.1em 0.5em;
            border-radius: 20px;
        }
        .artist {
            opacity: 0.7;
            margin-top: 5px;
            padding: 0.1em 0.7em;
            border-radius: 20px;
        }

        p {
            margin: 0;
            line-height: 1em;
            color: #d2cfcf;
            max-lines: 2;
        }

        small {
            opacity: 0.3;
            font-weight: bold;
            font-size: 12px;
            letter-spacing: 0.2px;
        }

        .cd-img {
            position: absolute;
            width: 100%;
            object-fit: cover;
            /* transition: left 0.2s cubic-bezier(0.075, 0.82, 0.165, 1); */
            transition: all 0.8s cubic-bezier(0.075, 0.82, 0.165, 1);
            > img {
                width: 100%;
            }
            transform: translateX(5px) translateY(-4px) rotate(0deg);
            /* transform-origin: 5px 5px; */
        }

        .texture {
            position: absolute;
            top: 0;
            bottom: 0;
            right: 0;
            left: 0;
            width: 100%;
            object-fit: cover;
            z-index: 6;
            opacity: 0.2;
        }

        .cd-placeholder {
            position: absolute;
            top: 2px;
            bottom: 0;
            right: 0;
            left: 2px;
            width: 100%;
            object-fit: cover;
        }
    }

    .cd {
        display: flex;
        flex-direction: row;
        width: 100%;
        height: auto;
        aspect-ratio: 1.136;
        box-shadow: 2px 2px 3px 2px rgba(0, 0, 0, 0.103);
        box-sizing: border-box;
        .hinge {
            border-left: 1px solid rgba(255, 255, 255, 0.066);
            height: 100%;
            width: 5%;
            background-color: rgba(255, 255, 255, 0.032);
        }

        .artwork-container {
            padding: 0em;
            width: 100%;
            height: fit-content;
            aspect-ratio: 1;
            opacity: 1;
            box-sizing: content-box;
            border-top: 0.55px solid #ffffff0f;
            border-bottom: 0.55px solid #ffffff0c;
            z-index: 0;
            border-top-right-radius: 3px;
            border-bottom-right-radius: 3px;
            user-select: none;
            position: relative;
            &:hover {
                .artwork {
                    /* opacity: 0.9; */
                    filter: brightness(1.2);
                }
            }

            .artwork-frame {
                width: 100%;
                height: 100%;
                box-sizing: border-box;
                /* border-radius: 3px; */
                /* border: 1px solid rgb(94, 94, 94); */
                display: flex;
                align-items: center;
                justify-content: center;
                box-shadow: 3px 2px 4px 1px rgba(0, 0, 0, 0.093) inset;

                .play-button-container {
                    position: absolute;
                    align-self: center;
                    border: 1px solid #6651a3;
                    background-color: #25222b;
                    border-radius: 50px;
                    width: 40px;
                    height: 40px;
                    display: flex;
                    z-index: 10;

                    &:hover {
                        background-color: #5123dd;
                    }
                    &:active {
                        background-color: #4420b1;
                        transform: scale(0.9);
                    }
                    .play-button {
                        position: relative;
                        left: 1px;
                        font-size: 2em;
                        margin: auto;
                        align-self: center;
                    }

                    .pause-button {
                        font-size: 2.2em;
                        margin: auto;
                        align-self: center;
                    }
                }

                > .artwork {
                    object-fit: cover;
                    width: 100%;
                    height: auto;
                    aspect-ratio: 1;
                    z-index: 3;
                }
                .artwork-placeholder {
                    opacity: 0.1;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: 1em;
                    z-index: -1;
                    .artwork {
                        width: 100%;
                    }
                    iconify-icon {
                        /* margin-top: 0.7em; */
                        font-size: 2em;
                        display: none;
                    }
                }
            }
        }
    }
</style>
