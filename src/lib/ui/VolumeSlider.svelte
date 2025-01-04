<script lang="ts">
    import { volume } from "../../data/store";
    import { currentThemeObject } from "../../theming/store";

    $: {
        const thumbBg =
            btoa(`<svg width="20" height="20" fill="none" xmlns="http://www.w3.org/2000/svg">
            <g clip-path="url(#a)">
                <path d="M10 20c5.523 0 10-4.477 10-10S15.523 0 10 0 0 4.477 0 10s4.477 10 10 10Z" fill="${$currentThemeObject["transport-volume-thumb-bg"]}"/>
                <path fill-rule="evenodd" clip-rule="evenodd" d="M9.589 5.382a.666.666 0 0 1 .411.616v8a.666.666 0 0 1-1.138.471L6.391 12H4.667A.667.667 0 0 1 4 11.33V8.665a.667.667 0 0 1 .667-.667H6.39l2.471-2.471a.667.667 0 0 1 .727-.145Zm3.516-.098a.667.667 0 0 1 .942 0A6.648 6.648 0 0 1 16 9.998a6.648 6.648 0 0 1-1.953 4.714.666.666 0 0 1-.942-.943A5.313 5.313 0 0 0 14.667 10a5.313 5.313 0 0 0-1.562-3.772.667.667 0 0 1 0-.943ZM11.219 7.17a.666.666 0 0 1 .943 0A3.989 3.989 0 0 1 13.333 10a3.988 3.988 0 0 1-1.171 2.828.667.667 0 1 1-.943-.944A2.655 2.655 0 0 0 12 9.998a2.656 2.656 0 0 0-.781-1.885.667.667 0 0 1 0-.944Z" fill="${$currentThemeObject["transport-volume-thumb-icon"]}"/>
            </g>
            <defs>
                <clipPath id="a">
                    <path fill="#fff" d="M0 0h20v20H0z"/>
                </clipPath>
            </defs>
        </svg>
        `);

        document.documentElement.style.setProperty(
            "--transport-volume-thumb-url",
            `url('data:image/svg+xml;base64,${thumbBg}')`,
        );
    }
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
                background: var(--transport-volume-thumb-url);
            }

            &::-moz-range-thumb {
                width: $thumb_size;
                height: $thumb_size;
                background: var(--transport-volume-thumb-secondary);
            }
        }
    }
</style>
