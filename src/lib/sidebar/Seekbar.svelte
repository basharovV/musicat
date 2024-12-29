<script lang="ts">
    import { currentThemeObject } from "../../theming/store";

    // seconds
    export let duration = 0;
    export let playerTime = 0;
    export let onSeek = (val) => {};
    export let buffered: TimeRanges | null = null; // For Web Audio player
    export let style: "normal" | "thin" = "normal";
    export let showProgress = false;
    let bufferedRanges = [];
    // Function to update buffered ranges

    const updateBufferedRanges = () => {
        const ranges = [];
        for (let i = 0; i < buffered.length; i++) {
            ranges.push({
                start: buffered.start(i),
                end: buffered.end(i)
            });
        }
        bufferedRanges = ranges;
        console.log("bufferedRanges", bufferedRanges);
    };

    $: {
        if (buffered) {
            updateBufferedRanges();
        }
    }

    $: playheadPos = Math.min((playerTime / duration) * 100, 100);

    // $: console.log("Duration", duration);
    // $: console.log("Playhead", playheadPos);

    $: hoverheadPosPx = 0;

    let seekBar;

    $: hoverTime =
        seekBar &&
        `${(~~((duration * (hoverheadPosPx / seekBar.offsetWidth)) / 60))
            .toString()
            .padStart(2, "0")}:${(~~(
            (duration * (hoverheadPosPx / seekBar.offsetWidth)) %
            60
        ))
            .toString()
            .padStart(2, "0")}`;

    let showHoverHead = false;

    const onSeekHover = (e) => {
        hoverheadPosPx = e.offsetX > 0 ? e.offsetX : 0;
    };
</script>

<div class="container" class:thin={style === "thin"}>
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
        role="progressbar"
        class="seekbar-container"
        on:mouseenter={() => (showHoverHead = true)}
        on:mouseleave={() => (showHoverHead = false)}
        on:mousemove={onSeekHover}
        on:click={(e) => {
            const percent = e.offsetX / e.target.offsetWidth;
            onSeek && onSeek(duration * percent);
        }}
    >
        <div
            class="seekbar"
            bind:this={seekBar}
            class:hovered={showHoverHead}
            class:light={$currentThemeObject.type === "light"}
        >
            <svg
                class="playhead"
                style="left:{playheadPos}%;"
                viewBox="0 0 20 20"
            >
                <circle r="9" cy="10" />
            </svg>
            {#if showHoverHead}
                <div
                    class="hoverhead-container"
                    style="transform: translateX({hoverheadPosPx}px);"
                >
                    <svg class="hoverhead" viewBox="0 0 20 20">
                        <circle
                            width="8"
                            height="8"
                            cy="9"
                            r="10"
                            stroke="white"
                            stroke-width="2"
                        />
                    </svg>
                    <div class="hoverhead-tooltip">
                        <p>{hoverTime}</p>
                    </div>
                </div>
            {/if}

            {#if buffered}
                {#each bufferedRanges as range}
                    <div
                        class="seekbar buffered"
                        style="left: {(range.start / duration) * 100}%;
                        width: {((range.end - range.start) / duration) * 100}%;"
                    ></div>
                {/each}
            {/if}

            {#if showProgress}
                <div
                    class="seekbar progress"
                    style="width: {playheadPos}%;"
                ></div>
            {/if}
        </div>
    </div>
</div>

<style lang="scss">
    .container {
        display: flex;
        flex-direction: column;
        height: auto;
        width: 100%;
        align-items: center;
        justify-content: space-between;

        /* background: rgba(0, 0, 0, 0.954); */
        color: white;
        /* box-shadow: -5px -10px 5px rgba(0, 0, 0, 0.071); */
        z-index: 4;
        * {
            user-select: none;
        }
    }

    .thin {
        .seekbar {
            height: 0.75px;
            background: linear-gradient(
                to right,
                transparent,
                color-mix(in srgb, var(--background) 76%, white) 50%,
                transparent
            );
            &.light {
                background: linear-gradient(
                    to right,
                    transparent,
                    color-mix(in srgb, var(--background) 56%, black) 50%,
                    transparent
                );
            }
        }
    }

    p {
        margin: 0;
    }

    @keyframes scroll {
        from {
            left: 0%;
        }
        to {
            left: -100%;
        }
    }

    .seekbar-container {
        width: 100%;
        height: 15px;
        align-items: center;
        display: flex;
        /* cursor: ew-resize; */
    }

    .seekbar {
        background-color: color-mix(in srgb, var(--background) 76%, white);
        height: 2px;
        width: 100%;
        overflow: visible;
        align-items: center;
        display: flex;
        position: relative;
        z-index: 0;

        &.light {
            background-color: color-mix(in srgb, var(--background) 56%, black);
        }

        .buffered {
            position: absolute;
            top: 0;
            left: 0;
            bottom: 0;
            background-color: color-mix(in srgb, var(--type-bw-inverse) 36%, black);
            z-index: 1;
            transition: all 0.2s ease-in-out;
        }

        .progress {
            position: absolute;
            top: 0;
            left: 0;
            bottom: 0;
            background: linear-gradient(
                to right,
                transparent,
                color-mix(in srgb, var(--accent) 56%, black) 98%,
                transparent
            );
            z-index: 10;
            transition: all 0.2s ease-in-out;
        }

        .hoverhead-container {
            pointer-events: none;
            position: absolute;
            height: 12px;
            width: 12px;
            left: -5px;
            z-index: 20;

            .hoverhead-tooltip {
                position: absolute;
                top: -25px;
                left: -10px;
                font-size: 0.7em;
                background: black;
                padding: 0 0.3em;
                border-radius: 3px;
            }
        }

        svg {
            height: 10px;
            width: 10px;
            fill: var(--transport-seekbar-hoverhead);
            position: absolute;
            overflow: visible;
            pointer-events: none;
            transition: all 0.16s cubic-bezier(0.075, 0.82, 0.165, 1);
            z-index: 3;
            top: -4px;

            &.hoverhead {
                fill: none;
                pointer-events: none;
                cursor: pointer;

                top: 2px;
                > rect {
                    stroke: white;
                    stroke-width: 2px;
                }
            }
        }
    }
</style>
