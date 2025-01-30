<script lang="ts">
    import { Stage as Stg } from "konva/lib/Stage";
    import { Rect as Rct } from "konva/lib/shapes/Rect";
    import { onMount } from "svelte";

    import Konva from "konva";
    import { debounce } from "lodash-es";
    import { Layer, Rect, Stage } from "svelte-konva";
    import { currentThemeObject } from "../../theming/store";

    export let yPercent = 0;
    export let onScroll = async (scrollProgress: number) => {};
    export let topPadding = 0;
    export let height = 100;

    let onScrollDebounced = debounce(onScroll, 10);

    let layer;
    $: if (
        scrollbar &&
        yPercent !== undefined &&
        height !== undefined &&
        layer
    ) {
        let newY = yPercent * (height - scrollbarHeight - topPadding);
        scrollbar.y(yPercent * (height - scrollbarHeight - topPadding));
    }

    let container: HTMLDivElement;
    let width = 10;

    let stage: Stg;

    let scrollbar: Rct;
    let scrollbarHeight = 100;

    let isHovering = false;
    const PADDING = 5;
    const IDLE_PIXEL_RATIO = 1.8;

    onMount(() => {
        width = container.getBoundingClientRect().width;
        height = container.getBoundingClientRect().height;

        Konva.pixelRatio = IDLE_PIXEL_RATIO;
        if (stage) {
            stage.getLayers().forEach((l) => {
                l.canvas.setPixelRatio(IDLE_PIXEL_RATIO);
                l.draw();
            });
        }
    });
</script>

<div bind:this={container}>
    <Stage config={{ width, height }} bind:handle={stage}>
        <Layer config={{ y: topPadding }} bind:handle={layer}>
            <!-- Scrollbars -->
            <Rect
                bind:handle={scrollbar}
                on:mouseenter={() => {
                    stage.container().style.cursor = "grab";
                    isHovering = true;
                }}
                on:mouseleave={() => {
                    isHovering = false;
                }}
                config={{
                    height: scrollbarHeight,
                    width: 8,
                    fill: $currentThemeObject[
                        isHovering
                            ? "library-scrollbar-hover-bg"
                            : "library-scrollbar-bg"
                    ],
                    draggable: true,
                    cornerRadius: 6,
                    dragBoundFunc: function (pos) {
                        pos.y = Math.max(
                            topPadding,
                            Math.min(pos.y, height - scrollbarHeight),
                        );
                        onScroll(
                            (pos.y - topPadding) /
                                (height - scrollbarHeight - topPadding),
                        );
                        pos.x = 0;
                        return pos;
                    },
                }}
            />
        </Layer>
    </Stage>
</div>

<style lang="scss">
    div {
        position: absolute;
        right: 0;
        width: 10px;
        height: 100%;
    }
</style>
