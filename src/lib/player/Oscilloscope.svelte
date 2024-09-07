<script lang="ts">
    import { onMount } from "svelte";
    import { currentSong, isFullScreenVisualiser } from "../../data/store";
    import audioPlayer from "./AudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import { AudioVisualiser } from "./AudioVisualiser";
    import { isIAPlaying } from "./WebAudioPlayer";
    import { WebAudioVisualiser } from "./WebAudioVisualiser";

    let canvas: HTMLCanvasElement;
    let container: HTMLDivElement;
    export let width;
    let height = 30;
    let analyser: AudioVisualiser | WebAudioVisualiser;
    export let show = true;
    let isMounted = false;

    $: if (analyser) {
        if (show) {
            analyser.shouldStopAnimation = false;
            analyser.setupAnalyserAnimation();
        } else {
            analyser.shouldStopAnimation = true;
            analyser.clearCanvas();
        }
    }

    $: if ($isIAPlaying && isMounted && canvas) {
        analyser?.tearDown();
        analyser = new WebAudioVisualiser(canvas);
    } else if (isMounted && canvas) {
        analyser?.tearDown();
        analyser = new AudioVisualiser(canvas);
    }

    onMount(() => {
        width = container.clientWidth;
        isMounted = true;
    });
</script>

<div
    class="container"
    bind:this={container}
    class:full-screen={$isFullScreenVisualiser}
    class:mini={!$isFullScreenVisualiser}
>
    <canvas
        bind:this={canvas}
        class:hidden={$currentSong === null}
        {width}
        {height}
    />
</div>

<style lang="scss">
    .hidden {
        opacity: 0;
    }

    canvas {
        width: 100%;
        height: 100%;
        opacity: 1;
    }
    .container {
        z-index: 1;
        bottom: 4px;
        pointer-events: none;
        width: 100%;

        &.mini {
            /* position: relative;
            width: 100%;
            height: 100%;
            top: -7px; */
            opacity: 0.8;
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
