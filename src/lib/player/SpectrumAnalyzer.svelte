<script lang="ts">
    import { onMount } from "svelte";
    import { currentSong, isFullScreenVisualiser } from "../../data/store";
    import audioPlayer from "./AudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import { AudioVisualiser } from "./AudioVisualiser";

    let canvas: HTMLCanvasElement;
    let container: HTMLDivElement;
    let width;
    let height;
    let analyser: AudioVisualiser;

    onMount(() => {
        width = container.clientWidth;
        height = container.clientHeight;

        analyser = new AudioVisualiser(
            audioPlayer.getCurrentAudioFile(),
            canvas
        );

        isFullScreenVisualiser.subscribe((isFullScreen) => {
            console.log("w", width);
            console.log("h", height);
            if (isFullScreen) {
                canvas.width = window.innerWidth;
                canvas.height = window.innerHeight;
                analyser && analyser.setCanvas(canvas);
            } else {
                canvas.width = width;
                canvas.height = height;
                analyser && analyser.setCanvas(canvas);
            }
        });
    });
</script>

<div
    class="container"
    bind:this={container}
    class:full-screen={$isFullScreenVisualiser}
    class:mini={!$isFullScreenVisualiser}
>
    <canvas bind:this={canvas} class:hidden={$currentSong === null} />

    {#if $isFullScreenVisualiser}
        <div class="icon">
            <Icon
                icon="ph:wave-sine-duotone"
                onClick={() =>
                    ($isFullScreenVisualiser = !$isFullScreenVisualiser)}
                color="#474747"
            />
        </div>
    {/if}
</div>

<style lang="scss">
    .hidden {
        opacity: 0;
    }

    canvas {
        width: 100%;
        height: auto;
        opacity: 1;
    }
    .container {
        z-index: 1;
        position: absolute;
        bottom: 4px;
        pointer-events: none;
        width: 100%;

        &.mini {
            /* position: relative;
            width: 100%;
            height: 100%;
            top: -7px; */
            opacity: 0.2;
        }

        &.full-screen {
            position: fixed;
            top: 0;
            bottom: 0;
            right: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
            background-color: #242026;
            z-index: 50;
            pointer-events: visible;
            opacity: 1;
        }

        .icon {
            top: 2em;
            left: 2em;
            position: fixed;
        }
    }
</style>
