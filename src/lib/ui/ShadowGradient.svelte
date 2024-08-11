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
            color-mix(in srgb, var(--panel-background) 10%, transparent) 70%,
            var(--panel-background)
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
    }
</style>
