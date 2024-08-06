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
            to bottom,
            var(--shadow-bg) 100%,
            color-mix(in srgb, var(--shadow-bg) 98%, transparent) 2.6%,
            color-mix(in srgb, var(--shadow-bg) 95%, transparent) 5.8%,
            color-mix(in srgb, var(--shadow-bg) 89%, transparent) 9.7%,
            color-mix(in srgb, var(--shadow-bg) 82%, transparent) 14.3%,
            color-mix(in srgb, var(--shadow-bg) 74%, transparent) 19.5%,
            color-mix(in srgb, var(--shadow-bg) 65%, transparent) 25.3%,
            color-mix(in srgb, var(--shadow-bg) 55%, transparent) 31.6%,
            color-mix(in srgb, var(--shadow-bg) 45%, transparent) 38.5%,
            color-mix(in srgb, var(--shadow-bg) 36%, transparent) 45.9%,
            color-mix(in srgb, var(--shadow-bg) 26%, transparent) 53.9%,
            color-mix(in srgb, var(--shadow-bg) 18%, transparent) 62.2%,
            color-mix(in srgb, var(--shadow-bg) 11%, transparent) 71.1%,
            color-mix(in srgb, var(--shadow-bg) 5%, transparent) 80.3%,
            color-mix(in srgb, var(--shadow-bg) 1%, transparent) 90%,
            color-mix(in srgb, var(--shadow-bg) 0%, transparent) 100%
        );
        &.light {
            background: linear-gradient(
                to bottom,
                hsl(320, 0.92%, 50.96%) 0%,
                hsla(320, 0.92%, 50.96%, 0.988) 2.6%,
                hsla(320, 0.92%, 50.96%, 0.952) 5.8%,
                hsla(320, 0.92%, 50.96%, 0.898) 9.7%,
                hsla(320, 0.92%, 50.96%, 0.828) 14.3%,
                hsla(320, 0.92%, 50.96%, 0.745) 19.5%,
                hsla(320, 0.92%, 50.96%, 0.654) 25.3%,
                hsla(320, 0.92%, 50.96%, 0.557) 31.6%,
                hsla(320, 0.92%, 50.96%, 0.458) 38.5%,
                hsla(320, 0.92%, 50.96%, 0.361) 45.9%,
                hsla(320, 0.92%, 50.96%, 0.268) 53.9%,
                hsla(320, 0.92%, 50.96%, 0.184) 62.2%,
                hsla(320, 0.92%, 50.96%, 0.112) 71.1%,
                hsla(320, 0.92%, 50.96%, 0.055) 80.3%,
                hsla(320, 0.92%, 50.96%, 0.016) 90%,
                hsla(320, 0.92%, 50.96%, 0) 100%
            );
        }
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
            color-mix(in srgb, var(--panel-background) 10%, transparent) 70%,
            var(--panel-background)
        );
        &.light {
            background: linear-gradient(
                to top,
                hsl(320, 0.92%, 50.96%) 0%,
                hsla(320, 0.92%, 50.96%, 0.988) 2.6%,
                hsla(320, 0.92%, 50.96%, 0.952) 5.8%,
                hsla(320, 0.92%, 50.96%, 0.898) 9.7%,
                hsla(320, 0.92%, 50.96%, 0.828) 14.3%,
                hsla(320, 0.92%, 50.96%, 0.745) 19.5%,
                hsla(320, 0.92%, 50.96%, 0.654) 25.3%,
                hsla(320, 0.92%, 50.96%, 0.557) 31.6%,
                hsla(320, 0.92%, 50.96%, 0.458) 38.5%,
                hsla(320, 0.92%, 50.96%, 0.361) 45.9%,
                hsla(320, 0.92%, 50.96%, 0.268) 53.9%,
                hsla(320, 0.92%, 50.96%, 0.184) 62.2%,
                hsla(320, 0.92%, 50.96%, 0.112) 71.1%,
                hsla(320, 0.92%, 50.96%, 0.055) 80.3%,
                hsla(320, 0.92%, 50.96%, 0.016) 90%,
                hsla(320, 0.92%, 50.96%, 0) 100%
            );
        }
    }
</style>
