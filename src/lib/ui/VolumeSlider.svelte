<script lang="ts">
    import { volume } from "../../data/store";
</script>

<div class="volume">
    <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        bind:value={$volume}
        class="slider"
        id="myRange"
    />
</div>

<style lang="scss">
    $thumb_size: 20px;
    .volume {
        width: 100%;
        display: flex;
        align-items: center;

        input {
            -webkit-appearance: none;
            width: 100%;
            height: 5px;
            background: color-mix(in srgb, var(--inverse) 20%, transparent);
            outline: none;
            opacity: 1;
            border-radius: 3px;
            -webkit-transition: 0.2s;
            transition: opacity 0.2s;

            &::-webkit-slider-thumb {
                -webkit-appearance: none;
                appearance: none;
                width: $thumb_size;
                height: $thumb_size;
                background: url("/images/volume-up.svg");
            }

            &::-moz-range-thumb {
                width: $thumb_size;
                height: $thumb_size;
                background: #04aa6d;
            }
        }
    }
</style>
