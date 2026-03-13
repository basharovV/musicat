<script lang="ts">
    import { onMount } from "svelte";
    import { current, uiPreferences } from "../../data/store";
    import { AudioVisualiser } from "./AudioVisualiser";
    import { isIAPlaying } from "./WebAudioPlayer";
    import { WebAudioVisualiser } from "./WebAudioVisualiser";
    import { openContextMenu } from "../ui/ContextMenu";
    import tippy from "svelte-tippy";

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

        return () => {
            analyser?.tearDown();
        };
    });
</script>

<div
    class="container"
    bind:this={container}
    use:tippy={{
        content: "Click to switch analyzer",
        placement: "top",
    }}
    on:click={() => {
        if ($uiPreferences.audioAnalyzer.analyzerType === "frequency") {
            $uiPreferences.audioAnalyzer.analyzerType = "time";
        } else {
            $uiPreferences.audioAnalyzer.analyzerType = "frequency";
        }
    }}
>
    <canvas
        bind:this={canvas}
        class:hidden={$current.song === null}
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
        width: 100%;
        height: 100%;
        pointer-events: all;

        mask-image: linear-gradient(
            to right,
            transparent 0%,
            #242026 15%,
            #242026 85%,
            transparent 100%
        );

        &:hover {
            cursor: pointer;
            mask-image: none;
            border: 1px solid var(--panel-separator);
            background: color-mix(in srgb, var(--inverse) 10%, transparent);
            border-radius: 5px;
        }
    }
</style>
