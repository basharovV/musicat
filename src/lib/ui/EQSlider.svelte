<script lang="ts">
    import { currentThemeObject } from "../../theming/store";
    import { createEventDispatcher } from "svelte";
    import { tweened } from "svelte/motion";
    import { cubicOut } from "svelte/easing";

    export let value: number;
    export let min = -12;
    export let max = 12;
    export let step = 0.1;
    export let label: string;

    const dispatch = createEventDispatcher();

    // 1. Initialize the tweened store
    const displayValue = tweened(value, {
        duration: 400,
        easing: cubicOut,
    });

    // 2. Reactively update the tween when the 'value' prop changes from outside
    $: displayValue.set(value);

    function handleInput(e: Event) {
        const target = e.target as HTMLInputElement;
        const newValue = parseFloat(target.value);

        // Update the prop directly so the parent knows
        value = newValue;

        // We set the tween with 0 duration on manual drag
        // to prevent "fighting" the user's mouse
        displayValue.set(newValue, { duration: 0 });

        dispatch("update", newValue);
    }
    $: {
        const thumbColor = $currentThemeObject["secondary"] || "#444";
        const thumbActiveColor = $currentThemeObject["accent"] || "#444";
        const thumbSvg = btoa(`
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <circle cx="12" cy="12" r="9" fill="${thumbColor}" stroke="white" stroke-width="1"/>
            </svg>
        `);

        const thumbActiveSvg = btoa(`
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <circle cx="12" cy="12" r="9" fill="${thumbActiveColor}" stroke="white" stroke-width="1"/>
            </svg>
        `);

        document.documentElement.style.setProperty(
            "--eq-thumb-url",
            `url('data:image/svg+xml;base64,${thumbSvg}')`,
        );

        document.documentElement.style.setProperty(
            "--eq-thumb-active-url",
            `url('data:image/svg+xml;base64,${thumbActiveSvg}')`,
        );
    }
</script>

<div class="eq-band">
    <span class="gain-value">
        {$displayValue > 0 ? "+" : ""}{$displayValue.toFixed(1)}
    </span>

    <div class="slider-container">
        <input
            type="range"
            {min}
            {max}
            {step}
            value={$displayValue}
            on:input={handleInput}
            class="vertical-slider"
        />
        <div class="center-line"></div>
    </div>

    <label class="freq-label">{label}</label>
</div>

<style lang="scss">
    .eq-band {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 45px;
        user-select: none;
        height: 100%;
    }

    .gain-value {
        font-size: 0.65rem;
        font-family: monospace;
        color: var(--secondary);
        margin-bottom: 10px;
        height: 12px;
    }

    .slider-container {
        position: relative;
        height: 150px;
        width: 30px;
        display: flex;
        justify-content: center;
        align-items: center;

        .center-line {
            position: absolute;
            width: 2px;
            height: 100%;
            background: var(--muted);
            border-radius: 1px;
            pointer-events: none;
        }
    }

    .vertical-slider {
        -webkit-appearance: none;
        appearance: none;
        width: 150px; // This is the height of the slider
        height: 30px;
        background: transparent;
        transform: rotate(-90deg);
        cursor: pointer;
        z-index: 2;

        &::-webkit-slider-thumb {
            -webkit-appearance: none;
            height: 24px;
            width: 24px;
            background: var(--eq-thumb-url);
            background-size: contain;
            background-repeat: no-repeat;
            margin-top: -2px;
            filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
            &:active {
                background: var(--eq-thumb-active-url);
            }
        }

        &::-moz-range-thumb {
            height: 24px;
            width: 24px;
            background: var(--eq-thumb-url);
            background-size: contain;
            border: none;
        }
    }

    .freq-label {
        font-size: 0.7rem;
        margin-top: 10px;
        color: #888;
        font-weight: bold;
    }
</style>
