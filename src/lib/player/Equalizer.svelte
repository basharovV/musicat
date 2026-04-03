<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { fade } from "svelte/transition";
    import {
        equalizerSettings,
        isEqualizerOpen,
        isLyricsHovered,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { clickOutside } from "../../utils/ClickOutside";
    import Dropdown from "../ui/Dropdown.svelte";
    import EQSlider from "../ui/EQSlider.svelte";
    import ToggleButton from "../ui/ToggleButton.svelte";
    import {
        EQ_LABELS,
        EQ_PRESET_OPTIONS,
        EQ_PRESETS,
    } from "./EqualizerPresets";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    // Winamp-style standard bands
    let bands = [...$equalizerSettings.settings.bands];

    function resetAllBands() {
        equalizerSettings.reset();
        bands = [...$equalizerSettings.settings.bands];
    }

    $: currentPresetOption = {
        value: $equalizerSettings.settings.name || $LL.bottomBar.selectPreset(),
        label: $equalizerSettings.settings.name || $LL.bottomBar.selectPreset(),
    };

    function selectPreset(presetName: string) {
        const preset = EQ_PRESETS.find((p) => p.name === presetName);
        console.log("preset", preset, presetName, EQ_PRESETS);
        if (!preset) return;
        $equalizerSettings = {
            isEnabled: true,
            settings: {
                name: preset.name,
                bands: [...preset.bands.map((b) => ({ ...b }))],
            },
        };
        bands = [...preset.bands];
    }

    function onBandChange() {
        $equalizerSettings = {
            isEnabled: true,
            settings: {
                name: "Custom",
                bands: [...bands.map((b) => ({ ...b }))],
            },
        };
    }
</script>

<div class="equalizer">
    <ToggleButton
        isSelected={$isEqualizerOpen || $equalizerSettings.isEnabled}
        onClick={() => ($isEqualizerOpen = !$isEqualizerOpen)}
        icon="cli:equalizer"
        iconSize={14}
        text={$LL.bottomBar.equalizer()}
    />

    {#if $isEqualizerOpen}
        <div
            in:fade={{ duration: 150 }}
            class="container"
            use:clickOutside={() => ($isEqualizerOpen = false)}
            on:mouseenter={() => {
                $isLyricsHovered = true;
            }}
            on:mouseleave={() => {
                $isLyricsHovered = false;
            }}
        >
            <div class="header">
                <Dropdown
                    options={EQ_PRESET_OPTIONS}
                    selected={currentPresetOption}
                    onSelect={selectPreset}
                />
                <ButtonWithIcon
                    size="xsmall"
                    theme="transparent"
                    onClick={resetAllBands}
                    text="RESET"
                ></ButtonWithIcon>
            </div>
            <div class="sliders-wrapper">
                {#each bands as band, i}
                    <EQSlider
                        bind:value={band.gain}
                        label={EQ_LABELS[i]}
                        on:update={onBandChange}
                    />
                {/each}
            </div>
        </div>

        <svg class="arrow" width="28" height="12" viewBox="0 0 28 12">
            <path
                d="M 2 12 C 8 12, 10 0, 14 0 C 18 0, 20 12, 26 12"
                fill="var(--overlay)"
                stroke="var(--border)"
                stroke-width="1.2"
            />
        </svg>
    {/if}
</div>

<style lang="scss">
    @use "../../styles/mixins" as *;
    .equalizer {
        position: relative;
    }

    .container {
        @include popup;
        position: absolute;
        bottom: 36px;
        left: -240px; // Center it better based on wider layout
        padding: 1.2rem;

        z-index: 100;
        width: 420px;

        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
            h4 {
                margin: 0;
                font-size: 0.75rem;
                letter-spacing: 1px;
                text-transform: uppercase;
                color: var(--primary);
            }
            .reset-all {
                background: transparent;
                border: 1px solid rgba(255, 255, 255, 0.1);
                color: #888;
                font-size: 0.6rem;
                padding: 2px 6px;
                cursor: pointer;
                border-radius: 3px;
                &:hover {
                    color: #fff;
                    border-color: #fff;
                }
            }
        }
    }
    .sliders-wrapper {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        height: 180px;
        padding: 0 10px 10px 10px;
        position: relative;
        border-radius: 5px;

        // 1. The Subtle Background Grid
        &::before {
            content: "";
            position: absolute;
            /* Adjust 'top' and 'bottom' to match the actual 
               vertical span of your slider track */
            top: 23px;
            bottom: 38px;
            left: 0;
            right: 0;

            // horizontal lines every 15px
            background-image: linear-gradient(
                var(--border) 1px,
                transparent 1px
            );
            background-size: 100% 14px; // The mask fades the left and right 10% of the container
            mask-image: linear-gradient(
                to right,
                transparent 0%,
                black 20%,
                black 80%,
                transparent 100%
            );
            opacity: 0.9;
            pointer-events: none;
        }

        // 2. The Absolute Zero Line
        &::after {
            content: "";
            position: absolute;
            left: 0;
            right: 0;
            height: 1px;
            background: var(--border);

            /* ADJUST THIS PERCENTAGE:
               If your slider goes from +12 to -12, 50% is correct.
               But if the container includes labels, try 42% or 45%. 
            */
            top: 43.5%;

            pointer-events: none;
            mask-image: linear-gradient(
                to right,
                transparent 0%,
                black 20%,
                black 80%,
                transparent 100%
            );
            box-shadow: 0 0 8px var(--shadow);
        }
    }

    .slider-column {
        display: flex;
        flex-direction: column;
        align-items: center;
        flex: 1;
        gap: 8px;

        .gain-label {
            font-size: 0.65rem;
            font-family: monospace;
            color: #666;
            height: 14px;
        }

        .freq-label {
            font-size: 0.7rem;
            font-weight: bold;
            color: #999;
            margin-top: 4px;
        }

        .range-container {
            position: relative;
            height: 120px;
            display: flex;
            justify-content: center;

            .track-line {
                position: absolute;
                left: 50%;
                top: 0;
                bottom: 0;
                width: 2px;
                background: rgba(255, 255, 255, 0.9);
                transform: translateX(-50%);
                z-index: 1;
                pointer-events: none;

                // Horizontal ticks for that Winamp look
                background-image: linear-gradient(
                    to bottom,
                    transparent 0%,
                    transparent 48%,
                    rgba(0, 0, 0, 0.8) 50%,
                    transparent 52%
                );
                background-size: 100% 20px;
            }
        }
    }

    .arrow {
        position: absolute;
        bottom: 25.3px;
        left: 0;
        right: 0;
        margin: auto;
        rotate: 180deg;
        z-index: 101;
    }
</style>
