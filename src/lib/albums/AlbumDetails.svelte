<script lang="ts">
    import { fade } from "svelte/transition";
    import type { Album, Song } from "../../App";
    import { albumColumns } from "../../data/store";
    import { clickOutside } from "../../utils/ClickOutside";
    import CanvasLibrary from "../library/CanvasLibrary.svelte";

    export let album: Album;
    export let onUnselect: () => void;
    export let tracks: Song[];
    export let anchorTopLeft = { x: 0, y: 0, height: 0, width: 0 };
    export let containerWidth = 0;

    let x = anchorTopLeft.x;
    let y = anchorTopLeft.y;
    let width = 0;
    let height = 0;

    let position: "above" | "below" = "below";

    let container: HTMLDivElement;
    let isLoaded = false;

    const HEADER_HEIGHT = 30 + 20;
    const ROW_HEIGHT = 26;
    const MARGIN = 10;

    $: if (isLoaded && anchorTopLeft) {
        const parentBounds = container.parentElement.getBoundingClientRect();

        // Width
        const dynamicWidth = containerWidth > 1200 ? 0.6 : 0.8;
        width = Math.min(containerWidth * dynamicWidth, 800);

        // Desired content height
        const desiredHeight =
            HEADER_HEIGHT + 30 + album.tracksIds.length * ROW_HEIGHT;

        // Available vertical space
        const spaceBelow =
            parentBounds.y +
            parentBounds.height -
            (anchorTopLeft.y + anchorTopLeft.height) -
            MARGIN;

        const spaceAbove = anchorTopLeft.y - parentBounds.y - MARGIN;

        // Choose position with more space
        position = spaceBelow >= spaceAbove ? "below" : "above";

        const availableSpace = position === "below" ? spaceBelow : spaceAbove;

        // Max height: 80% of parent AND available space
        const maxHeight = Math.min(parentBounds.height * 0.8, availableSpace);

        height = Math.min(desiredHeight, maxHeight);

        // X positioning (unchanged logic)
        const adjustedX = anchorTopLeft.x - width / 2 + anchorTopLeft.width / 2;

        if (adjustedX + width > parentBounds.x + parentBounds.width) {
            x = parentBounds.x + parentBounds.width - width - MARGIN;
        } else if (adjustedX < parentBounds.x) {
            x = parentBounds.x + MARGIN;
        } else {
            x = adjustedX;
        }

        // Y positioning — never overlaps anchor
        if (position === "below") {
            y = anchorTopLeft.y + anchorTopLeft.height + MARGIN;
        } else {
            y = anchorTopLeft.y - height - MARGIN;
        }
    }

    let isHovered = false;

    function unselect() {
        onUnselect && onUnselect();
    }
</script>

{#if album}
    <svg
        class="arrow"
        class:position
        style={`left: ${anchorTopLeft.x + anchorTopLeft.width / 2}px;
            top: ${position === "below" ? y - 12 : y + height - 3};
            transform: rotate(${position === "above" ? 180 : 0}deg);`}
        width="28"
        height="12"
        viewBox="0 0 28 12"
    >
        <!-- Smooth Gaussian-like bump -->
        <path
            d="
            M 2 12
            C 8 12, 10 0, 14 0
            C 18 0, 20 12, 26 12
        "
            fill="var(--popup-body-bg)"
            stroke="var(--panel-primary-border-accent1)"
            stroke-width="1.2"
        />
    </svg>

    <div
        in:fade={{ duration: 150 }}
        class="container"
        class:hovered={isHovered && album.artwork}
        style={`left: ${x}px ;top: ${y}px;width: ${width}px;height: ${height}px`}
        use:clickOutside={unselect}
        bind:this={container}
    >
        <div class="info-container">
            <div class="info-frame">
                <small class="title">{album.displayTitle ?? album.title}</small>
                <small>•</small>
                {#if album.artist}
                    <small class="artist">{album.artist}</small>
                {/if}
                <small>•</small>
                {#if album.year > 0}
                    <small>{album.year}</small>
                {/if}
                <!-- <small
                    >{album.tracksIds.length}
                    {$LL.albums.item.tracksLabel()}</small
                > -->
            </div>
        </div>
        <div class="songs" style={`height: ${height - HEADER_HEIGHT}px`}>
            <CanvasLibrary
                columnOrder={albumColumns}
                songsArray={tracks}
                bind:shouldRender={isLoaded}
            />
        </div>
    </div>
{/if}

<style lang="scss">
    .container {
        position: fixed;
        height: auto;
        width: min(35%, 800px);
        display: grid;
        grid-template-rows: auto auto;
        padding: 0.5em;
        background-color: var(--popup-body-bg);
        backdrop-filter: blur(8px);
        box-shadow: 0px 5px 40px var(--overlay-shadow);
        border-radius: 5px;
        border: 1px solid var(--panel-primary-border-accent1);
        .info-container {
            display: flex;
            overflow: hidden;
            height: 30px;

            .info-frame {
                margin-left: 5px;
                display: flex;
                flex-direction: row;
                justify-content: flex-start;
                align-items: center;
                text-align: left;
                flex-grow: 1;
                max-width: 100%;
                overflow: hidden;
                text-overflow: ellipsis;
                gap: 5px;

                .title {
                    opacity: 1;
                }
                .artist {
                    opacity: 0.5;
                }
                small {
                    white-space: nowrap;
                    opacity: 0.3;
                    font-weight: bold;
                    letter-spacing: 0.2px;
                    line-height: 18px;
                    font-size: 14px;
                    text-overflow: ellipsis;
                }
            }

            .artwork-container {
                padding: 0em;
                width: auto;
                height: 100%;
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
            margin: 5px 0 0 0;
            border-radius: 5px;
            overflow: hidden;
            border-top: 0.7px solid
                color-mix(in srgb, var(--inverse) 40%, transparent);
            border-bottom: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
        }
    }

    .arrow {
        position: fixed;
        height: 15px;
        width: 20px;
        box-shadow: 0px 5px 40px var(--overlay-shadow);
        z-index: -1;
    }
</style>
