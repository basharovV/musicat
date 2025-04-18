<script lang="ts">
    import { fade } from "svelte/transition";
    import Icon from "../ui/Icon.svelte";
    import LL from "../../i18n/i18n-svelte";
    import CanvasLibrary from "../library/CanvasLibrary.svelte";
    import { liveQuery } from "dexie";
    import audioPlayer from "../player/AudioPlayer";
    import { albumColumnOrder, current, isPlaying } from "../../data/store";
    import { setQueue } from "../../data/storeHelper";
    import { HEADER_HEIGHT, ROW_HEIGHT } from "./util";
    import type { Album, Song } from "../../App";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    export let album: Album;
    export let onUnselect: () => void;
    export let tracks: Song[];

    let isHovered = false;

    $: isPlayingCurrentAlbum =
        $current.song?.album.toLowerCase() === album?.title.toLowerCase();

    $: canvasHeight = HEADER_HEIGHT + tracks?.length * ROW_HEIGHT + 8;

    async function playPauseToggle() {
        if ($current.song?.album.toLowerCase() === album.title.toLowerCase()) {
            if ($isPlaying) {
                audioPlayer.pause();
            } else {
                audioPlayer.play(true);
            }
        } else {
            setQueue(tracks, 0);
        }
    }

    function unselect() {
        onUnselect && onUnselect();
    }
</script>

{#if album}
    <div
        in:fade={{ duration: 150 }}
        class="container"
        class:hovered={isHovered && album.artwork}
    >
        <div class="info-container">
            <div
                class="artwork-container"
                on:mouseenter={() => {
                    isHovered = true;
                }}
                on:mouseleave={() => {
                    isHovered = false;
                }}
            >
                <!-- svelte-ignore a11y-missing-attribute -->
                <img
                    class="texture"
                    src="images/textures/soft-wallpaper.png"
                    loading="lazy"
                    on:contextmenu|preventDefault={() => {}}
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
                    {#if isHovered}
                        <div
                            class={$isPlaying && isPlayingCurrentAlbum
                                ? "play-button-container pause-button"
                                : "play-button-container play-button"}
                        >
                            <div
                                class="button"
                                on:click|stopPropagation={playPauseToggle}
                            >
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
            <div class="info-frame">
                <p class="title">{album.displayTitle ?? album.title}</p>
                {#if album.artist}
                    <p class="artist">{album.artist}</p>
                {/if}
                <div class="info">
                    {#if album.year > 0}
                        <small>{album.year}</small>
                        <small>•</small>
                    {/if}
                    <small
                        >{album.tracksIds.length}
                        {$LL.albums.item.tracksLabel()}</small
                    >
                </div>
            </div>
            <div class="close">
                <ButtonWithIcon
                    onClick={unselect}
                    icon="material-symbols:close"
                    iconSize={30}
                    theme="transparent"
                    noOutline={true}
                />
            </div>
        </div>
        <div class="songs" style="height: {canvasHeight}px">
            <CanvasLibrary
                bind:columnOrder={$albumColumnOrder}
                allSongs={liveQuery(() => tracks)}
            />
        </div>
    </div>
{/if}

<style lang="scss">
    .container {
        width: 100%;
        padding: 1em 0 1em 8px;

        .info-container {
            display: flex;

            .info-frame {
                margin-left: 2em;
                display: flex;
                flex-direction: column;
                justify-content: flex-end;
                text-align: left;
                flex-grow: 1;

                .title {
                    border-radius: 20px;
                    font-size: 32px;
                }
                .artist {
                    opacity: 0.7;
                    margin-top: 20px;
                    font-size: 20px;
                }

                .info {
                    display: flex;
                    flex-direction: row;
                    width: 100%;
                    gap: 4px;
                    justify-content: left;
                    align-items: left;
                    margin-top: 10px;
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
                    font-size: 16px;
                    letter-spacing: 0.2px;
                    line-height: initial;
                }
            }

            .artwork-container {
                max-width: 300px;
                padding: 0em;
                width: 100%;
                height: fit-content;
                aspect-ratio: 1;
                opacity: 1;
                box-sizing: content-box;
                z-index: 0;
                border-top-right-radius: 3px;
                border-bottom-right-radius: 3px;
                user-select: none;
                position: relative;

                .artwork-frame {
                    width: 100%;
                    height: 100%;
                    box-sizing: border-box;
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
                                    color: var(
                                        --album-playing-pause-hover-icon
                                    );
                                }
                            }
                        }
                    }
                }

                .artwork {
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
        }

        .songs {
            position: relative;
            display: grid;
            grid-template-columns: 1fr;
            grid-template-rows: 1fr;
            margin: 3.5px 5px 0 0;
            border-radius: 5px;
            box-sizing: content-box;
            overflow: hidden;
            border-top: 0.7px solid
                color-mix(in srgb, var(--inverse) 40%, transparent);
            border-bottom: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
        }
    }
</style>
