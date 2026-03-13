<script>
    import { onMount } from "svelte";

    export let theme = "dark";

    // --- ADJUST THESE CONSTANTS ---
    const NOISE_OPACITY = 0.04; // Base transparency (0.0 to 1.0)
    const NOISE_DENSITY = 0.06; // Contrast/Graininess (0.0 to 1.0)
    const SAMPLE_SIZE = 512; // Size of the tile (higher = less repeating)
    // ------------------------------

    let canvas;
    let noiseDataUrl = ""; // This will hold the generated image string

    const generateNoise = () => {
        // Create an off-screen canvas if the bind hasn't caught yet
        const tempCanvas = canvas || document.createElement("canvas");
        const ctx = tempCanvas.getContext("2d");

        tempCanvas.width = SAMPLE_SIZE;
        tempCanvas.height = SAMPLE_SIZE;

        const imageData = ctx.createImageData(SAMPLE_SIZE, SAMPLE_SIZE);
        const data = imageData.data;
        const brightness = theme === "dark" ? 255 : 0;

        for (let i = 0; i < data.length; i += 4) {
            data[i] = brightness;
            data[i + 1] = brightness;
            data[i + 2] = brightness;

            // Calculate Alpha: (Base + Random Variance) * 255
            const alpha = (NOISE_OPACITY + Math.random() * NOISE_DENSITY) * 255;
            data[i + 3] = Math.floor(alpha);
        }

        ctx.putImageData(imageData, 0, 0);
        // Crucial: Update the reactive variable
        noiseDataUrl = tempCanvas.toDataURL();
    };

    // Run on mount and whenever theme changes
    onMount(generateNoise);
    $: if (theme) generateNoise();
</script>

<canvas bind:this={canvas} style="display: none;"></canvas>

<div class="noise-container">
    <div
        class="noise-overlay"
        style:background-image="url({noiseDataUrl})"
    ></div>
    <div class="content">
        <slot />
    </div>
</div>

<style>
    .noise-container {
        position: relative;
        width: 100%;
        height: 100%;
        /* Ensure there is a background color to see the noise against */
        background-color: transparent;
        overflow: hidden;
    }

    .noise-overlay {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        background-repeat: repeat;
        pointer-events: none;
        z-index: 1;
        /* Keeps the noise from looking blurry */
        image-rendering: pixelated;
    }

    .content {
        position: relative;
        z-index: 2;
    }
</style>
