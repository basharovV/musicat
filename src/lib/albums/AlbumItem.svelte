<script lang="ts">
    import { fade } from "svelte/transition";
    import type { Album } from "../../App";
    import { db } from "../../data/db";
    import {
        current,
        draggedAlbum,
        draggedSongs,
        isPlaying,
    } from "../../data/store";
    import audioPlayer from "../player/AudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import LL from "../../i18n/i18n-svelte";
    import { setQueue } from "../../data/storeHelper";

    export let album: Album; // to display album data
    export let highlighted = false;
    export let showInfo = true;

    let isHovered = false;
    async function playPauseToggle() {
        if (isPlayingCurrentAlbum) {
            audioPlayer.togglePlay();
        } else {
            const tracks = await db.songs
                .where("id")
                .anyOf(album.tracksIds)
                .toArray();

            tracks.sort((a, b) => {
                return a.trackNumber - b.trackNumber;
            });

            setQueue(tracks, 0);
        }
    }

    $: isPlayingCurrentAlbum =
        $current.song?.album.toLowerCase() === album.title.toLowerCase();
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
                {#if isHovered || isPlayingCurrentAlbum}
                    <div
                        class={$isPlaying && isPlayingCurrentAlbum
                            ? "play-button-container pause-button"
                            : "play-button-container play-button"}
                    >
                        <div class="button" on:click={playPauseToggle}>
                            <Icon
                                icon={$isPlaying && isPlayingCurrentAlbum
                                    ? "fe:pause"
                                    : "fe:play"}
                                size={25}
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
            <small
                >{album?.tracksIds.length}
                {$LL.albums.item.tracksLabel()}</small
            >
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
                color-mix(in srgb, var(--album-playing-shadow) 50%, transparent) !important;
            .hinge {
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 10.32%,
                    transparent
                );
                backdrop-filter: blur(8px);
                border-left: 1px solid
                    color-mix(in srgb, var(--inverse), transparent 50%);
                box-shadow:
                    inset -0.75px -0.5px rgba(255, 255, 255, 0.1),
                    inset + 0.75px +0.5px rgba(255, 255, 255, 0.025),
                    3px 2px 10px rgba(0, 0, 0, 0.25),
                    inset 0px 0px 10px 5px rgba(255, 255, 255, 0.025),
                    inset 0px 0px 40px 5px rgba(255, 255, 255, 0.025);
            }
            .artwork-frame {
                &::after {
                    content: "";
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    background-image: url("images/cd-shimmer.png");
                    background-repeat: no-repeat;
                    z-index: 20;
                    animation: playing-shimmer 3.5s
                        cubic-bezier(0.4, 0, 0.41, 0.97) 1;
                    pointer-events: none;
                }
            }
        }

        .cd-img {
            transform: translate(15%, -10px) rotate(140deg) !important;
        }
        .title {
            background-color: var(--album-playing-title-bg);
            border-radius: 4px;
            color: var(--text);
            z-index: 20;
            margin-top: 3px !important;
        }
        .artist,
        .info {
            z-index: 20;
        }
    }

    @keyframes playing-shimmer {
        0% {
            background-position: -40% 110%;
            background-size: 250%;
        }
        100% {
            background-position: 120% -30%;
            background-size: 250%;
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
                color-mix(in srgb, var(--inverse) 20.32%, transparent);
            width: 5%;
            background-color: color-mix(
                in srgb,
                var(--inverse) 9.32%,
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
                    border-radius: 50px;
                    width: 40px;
                    height: 40px;
                    display: flex;
                    z-index: 10;

                    &.play-button {
                        border: 1px solid
                            color-mix(
                                in srgb,
                                var(--album-playing-play-border) 60%,
                                transparent
                            );
                        background-color: var(--album-playing-play-bg);

                        .button {
                            position: relative;
                            left: 1px;
                            font-size: 2em;
                            margin: auto;
                            align-self: center;
                            color: var(--album-playing-play-icon);
                        }

                        &:hover {
                            background-color: var(
                                --album-playing-play-hover-bg
                            );

                            .button {
                                color: var(--album-playing-play-hover-icon);
                            }
                        }
                    }

                    &.pause-button {
                        border: 1px solid
                            color-mix(
                                in srgb,
                                var(--album-playing-pause-border) 60%,
                                transparent
                            );
                        background-color: var(--album-playing-pause-bg);

                        .button {
                            font-size: 2.2em;
                            margin: auto;
                            align-self: center;
                            color: var(--album-playing-pause-icon);
                        }

                        &:hover {
                            background-color: var(
                                --album-playing-pause-hover-bg
                            );

                            .button {
                                color: var(--album-playing-pause-hover-icon);
                            }
                        }
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
