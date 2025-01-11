<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { playbackSpeed } from "../../data/store";
    import { debounce } from "lodash-es";

    export let selected = false;
    export let onClick = () => {};
    let isAdjustingSpeed = false;
    let isFirstDeltaRun = true;
    let container;
    function scalePlaybackSpeedToRange(playbackSpeed) {
        const minInput = 0.3;
        const maxInput = 3;
        const minOutput = 1;
        const maxOutput = 200;

        // Ensure playbackSpeed is within the valid range
        playbackSpeed = Math.max(minInput, Math.min(maxInput, playbackSpeed));
        if (playbackSpeed === 1) {
            return 100;
        }
        console.log("playbackSpeed", playbackSpeed);
        // Scale the playbackSpeed to the output range
        const result =
            minOutput +
            ((playbackSpeed - minInput) / (maxInput - minInput)) *
                (maxOutput - minOutput);
        console.log("result", result);
        return result;
    }

    let previousX = 0;
    let offset = 0;
    let previousTime = 0;

    function onMouseMove(ev) {
        if (!isAdjustingSpeed || performance.now() - previousTime < 100) return;
        previousTime = performance.now();
        let newSpeed = $playbackSpeed + ev.movementX * 0.1;
        if (newSpeed < 0.3) {
            newSpeed = 0.3;
        } else if (newSpeed > 3) {
            newSpeed = 3;
        }
        $playbackSpeed = newSpeed;

        invoke("playback_speed_control", {
            event: {
                playback_speed: $playbackSpeed,
            },
        });
    }

    function onMouseUp(ev) {
        isAdjustingSpeed = false;
        isFirstDeltaRun = true;
        document.removeEventListener("mousemove", onMouseMove);
    }
</script>

<div
    class="toggle-button"
    class:selected
    class:adjusting={isAdjustingSpeed}
    bind:this={container}
    on:click={() => {
        selected = !selected;
    }}
    on:mousedown|preventDefault={() => {
        isAdjustingSpeed = true;
        document.addEventListener("mouseup", onMouseUp);
        document.addEventListener("mousemove", onMouseMove);
    }}
    on:dblclick={() => {
        $playbackSpeed = 1;
        invoke("playback_speed_control", {
            event: {
                playback_speed: $playbackSpeed,
            },
        });
    }}
>
    <p>{$playbackSpeed.toFixed(1)}X</p>
</div>

<style lang="scss">
    .toggle-button {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 5px;
        cursor: default;

        border: 1px solid rgba(128, 128, 128, 0.159);
        border-radius: 4px;
        padding: 0 4px;
        position: relative;
        cursor: ew-resize;
        transition: all 0.5s ease-in-out;
        min-width: 0px;
        &:hover {
            background-color: rgba(128, 128, 128, 0.191);
        }
        &:active {
            background-color: rgba(128, 128, 128, 0.391);
        }
        p {
            color: var(--text-secondary);
            margin: 0 0 1px 0;
            line-height: normal;
            font-size: 0.85em;
            text-align: center;
            width: 100%;
        }
        &.adjusting {
            min-width: 40px;
            border: 1px solid white;
        }
    }
</style>
