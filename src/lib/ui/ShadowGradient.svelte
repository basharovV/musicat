<script lang="ts">
    import { currentThemeObject } from "../../theming/store";

    export let type: "top" | "bottom" = "top";
</script>

<div class={`${type} ${$currentThemeObject.type}`}></div>

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
            color-mix(in srgb, var(--shadow-gradient-bg) 10%, transparent) 70%,
            var(--shadow-gradient-bg)
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
            color-mix(in srgb, var(--shadow-gradient-bg) 10%, transparent) 70%,
            var(--shadow-gradient-bg)
        );
    }
</style>
