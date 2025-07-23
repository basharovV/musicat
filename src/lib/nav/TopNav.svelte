<script lang="ts">
    import type { UiView } from "../../App";
    /* This navigation component is only used when the app window is small < 410px width */

    import { isQueueOpen, uiView } from "../../data/store";
    import Dropdown from "../ui/Dropdown.svelte";

    const views = [
        { value: "queue", label: "Queue" }, // can't be set with uiView, needs to be toggled specifically
        { value: "library", label: "Library" },
        { value: "albums", label: "Albums" },
        { value: "map", label: "Map" },
        { value: "wiki", label: "Wiki" },
    ];

    $: selected = views.find((v) => v.value === $uiView);
</script>

<div class="container" data-tauri-drag-region>
    <Dropdown options={views} {selected} onSelect={uiView.set} />
</div>

<style lang="scss">
    .container {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: auto;
        margin: 2px;
        z-index: 10;
    }
</style>
