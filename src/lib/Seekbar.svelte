<script lang="ts">
    import { playerTime, seekTime } from "../data/store";

    let playerState = {
        relativePosition: 0,
        loadProgress: 0,
        currentPosition: 0
    };

    // seconds
    export let duration = 0;

    $: elapsedTime = `${(~~($playerTime / 60))
        .toString()
        .padStart(2, "0")}:${(~~($playerTime % 60))
        .toString()
        .padStart(2, "0")}`;

    $: durationText = `${(~~(duration / 60)).toString().padStart(2, "0")}:${(~~(
        duration % 60
    ))
        .toString()
        .padStart(2, "0")}`;

    $: playheadPos = ($playerTime / duration) * 100;

    $: console.log(playheadPos);
    
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

    const onSeek = async (e) => {
        const percent = e.offsetX / e.target.offsetWidth;
        seekTime.set(duration * percent);
    };
</script>

<div class="container">
    <div
        class="seekbar-container"
        on:mouseenter={() => (showHoverHead = true)}
        on:mouseleave={() => (showHoverHead = false)}
        on:mousemove={onSeekHover}
        on:click={onSeek}
    >
        <div class="seekbar" bind:this={seekBar} class:hovered={showHoverHead}>
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
        </div>
    </div>
    <p class="elapsed-time">{elapsedTime} / {durationText}</p>
</div>

<style lang="scss">
    .container {
        display: flex;
        flex-direction: column;
        height: auto;
        padding: 0 1em 0;
        align-items: center;
        justify-content: space-between;

        /* background: rgba(0, 0, 0, 0.954); */
        color: white;
        /* box-shadow: -5px -10px 5px rgba(0, 0, 0, 0.071); */
        z-index: 4;
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
        background: rgba(255, 255, 255, 0.262);
        height: 2px;
        width: 100%;
        overflow: visible;
        align-items: center;
        display: flex;
        position: relative;

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
            fill: rgb(0, 197, 108);
            position: absolute;
            overflow: visible;
            pointer-events: none;
            transition: all 0.16s cubic-bezier(0.075, 0.82, 0.165, 1);

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

    .elapsed-time {
        opacity: 0.5;
        font-size: 12px;
    }
</style>
