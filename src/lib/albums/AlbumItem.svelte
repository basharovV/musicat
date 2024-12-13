<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import type { Album, Song } from "../../App";
    import { db } from "../../data/db";
    import {
        albumPlaylist,
        currentSong,
        draggedAlbum,
        draggedSongs,
        isPlaying,
        playlist,
        playlistType
    } from "../../data/store";
    import audioPlayer from "../player/AudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import { currentThemeObject } from "../../theming/store";
    import LL from "../../i18n/i18n-svelte";

    export let album: Album; // to display album data
    export let highlighted = false;
    export let showInfo = true;

    // console.log("highlight", highlighted);
    let isHovered = false;
    async function playPauseToggle() {
        if (
            $playlistType === "album" &&
            $currentSong?.album.toLowerCase() === album.title.toLowerCase()
        ) {
            if ($isPlaying) {
                audioPlayer.pause();
            } else {
                audioPlayer.play(true);
            }
        } else {
            let tracks = await db.songs
                .where("id")
                .anyOf(album.tracksIds)
                .toArray();
            tracks = tracks.sort((a, b) => {
                return a.trackNumber - b.trackNumber;
            });
            if (tracks) audioPlayer.playSong(tracks[0]);
            $playlist = tracks;
            $albumPlaylist = tracks;
            $playlistType = "album";
        }
    }

    $: isPlayingCurrentAlbum =
        $currentSong?.album.toLowerCase() === album.title.toLowerCase();
</script>

<div
    in:fade={{ duration: 150 }}
    class="container"
    class:hovered={isHovered && album.artwork}
    class:playing={isPlayingCurrentAlbum}
    class:highlighted
>
    {#if album.artwork}
        <div class="cd-img"><img async src="images/cd-hq.webp" /></div>
    {/if}

    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        class="cd"
        on:mouseenter={() => {
            isHovered = true;
        }}
        on:mouseleave={() => {
            isHovered = false;
        }}
        on:mousedown|preventDefault={async () => {
            let tracks = await db.songs
                .where("id")
                .anyOf(album.tracksIds)
                .toArray();
            tracks = tracks.sort((a, b) => {
                return a.trackNumber - b.trackNumber;
            });
            $draggedSongs = tracks;
            $draggedAlbum = album;
        }}
    >
        <div class="hinge" />
        <div class="artwork-container">
            <!-- svelte-ignore a11y-missing-attribute -->
            <img
                class="texture"
                src="images/textures/soft-wallpaper.png"
                loading="lazy"
                async
            />
            <div class="artwork-frame">
                {#if album.artwork}
                    <img
                        alt="Artwork"
                        type={album.artwork.format}
                        class="artwork"
                        src={album.artwork.src}
                        loading="lazy"
                        async
                    />
                {:else}
                    <!-- svelte-ignore a11y-missing-attribute -->
                    <div class="artwork-placeholder">
                        <Icon icon="mdi:music-clef-treble" />
                        <!-- svelte-ignore a11y-missing-attribute -->
                        <img
                            class="cd-placeholder"
                            src="images/cd-hq.png"
                            loading="lazy"
                            async
                        />
                        <!-- <small>No art</small> -->
                    </div>
                {/if}
                {#if isHovered || ($playlistType === "album" && isPlayingCurrentAlbum)}
                    <div class="play-button-container">
                        <div
                            class={$playlistType === "album" &&
                            $isPlaying &&
                            isPlayingCurrentAlbum
                                ? "pause-button"
                                : "play-button"}
                            on:click={playPauseToggle}
                        >
                            <Icon
                                icon={$playlistType === "album" &&
                                $isPlaying &&
                                isPlayingCurrentAlbum
                                    ? "fe:pause"
                                    : "fe:play"}
                                size={25}
                                color="white"
                            />
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    </div>
    {#if showInfo}
        <p class="title">{album.displayTitle ?? album.title}</p>
        <p class="artist">{album?.artist}</p>
        <div class="info">
            {#if album?.year > 0}
                <small>{album.year}</small>
                <small>•</small>
            {/if}
            <small>{album?.tracksIds.length} {$LL.albums.item.tracksLabel()}</small>
        </div>
    {/if}
</div>

<style lang="scss">
    .playing {
        transform: scale(1.1);
        z-index: 9;
        
        .cd {
            z-index: 8;
            /* box-shadow: 2px 2px 30px 20px rgba(39, 0, 178, 0.181) !important; */
            box-shadow: 2px 2px 50px 40px
                color-mix(in srgb, var(--library-playing-bg) 50%, transparent) !important;
            .hinge {
                background-color: rgba(167, 164, 173, 0.078);
            }
        }

        .cd-img {
            transform: translate(15%, -10px) rotate(140deg) !important;
        }
        .title {
            background-color: var(--accent-secondary);
            border-radius: 4px;
            color: var(--text);
            z-index: 20;
        }
        .artist,
        .info {
            z-index: 20;
        }
    }

    .hovered:not(.playing) {
        z-index: 8;

        .cd {
            z-index: 6;
        }

        .cd-img {
            transform: translate(12%, -5px) rotate(130deg) !important;
        }
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

    .highlighted {
        .cd {
            border: 1px solid rgb(233, 185, 255);
        }
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
        transition: all 0.6s cubic-bezier(0.075, 0.82, 0.165, 1);
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
            color: var(--text);
            max-lines: 2;
        }

        small {
            opacity: 0.3;
            font-weight: bold;
            font-size: 12px;
            letter-spacing: 0.2px;
            line-height: initial;
        }

        .cd-img {
            position: absolute;
            width: 100%;
            object-fit: cover;
            /* transition: left 0.2s cubic-bezier(0.075, 0.82, 0.165, 1); */
            transition: all 0.8s cubic-bezier(0.075, 0.82, 0.165, 1);
            transform: translateX(5%) translateY(2%) rotate(0deg);
            /* transform-origin: 5px 5px; */

            > img {
                width: 95%;
            }
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
            border-left: 1px solid
                color-mix(in srgb, var(--inverse) 10.32%, transparent);
            width: 5%;
            background-color: color-mix(
                in srgb,
                var(--inverse) 4.32%,
                transparent
            );
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
                    border: 1px solid
                        color-mix(
                            in srgb,
                            var(--library-playing-bg) 60%,
                            transparent
                        );
                    background-color: #25222b;
                    border-radius: 50px;
                    width: 40px;
                    height: 40px;
                    display: flex;
                    z-index: 10;

                    &:hover {
                        background-color: var(--library-playing-bg);
                    }
                    &:active {
                        background-color: var(--library-playing-bg);
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
                }
            }
        }
    }
</style>
