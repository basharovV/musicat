<script lang="ts">
    import { currentThemeObject } from "../../theming/store";

    export let type: "top" | "bottom" = "top";
    export let color: string = "var(--panel)";
</script>

<div
    class={`${type} ${$currentThemeObject.variant}`}
    style="--dynamic-color: {color};"
></div>

<style lang="scss">
    .top {
        height: 40px;
        width: 100%;
        position: absolute;
        top: 0;
        right: 0;
        left: 0;
        z-index: 20;
        opacity: 0.8;
        pointer-events: none;
        background: linear-gradient(
            to top,
            transparent,
            /* Use the dynamic variable here */
                color-mix(in srgb, var(--dynamic-color) 10%, transparent) 70%,
            var(--dynamic-color)
        );
    }

    .bottom {
        height: 50px;
        width: 100%;
        position: absolute;
        bottom: 0;
        right: 0;
        left: 0;
        z-index: 10;
        font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
        pointer-events: none;
        background: linear-gradient(
            to bottom,
            transparent,
            /* Use the dynamic variable here */
                color-mix(in srgb, var(--dynamic-color) 60%, transparent) 70%,
            var(--dynamic-color)
        );
    }
</style>
