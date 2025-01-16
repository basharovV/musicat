<script lang="ts">
    import {
        isQueueOpen,
        isSidebarOpen,
        os,
        uiPreferences,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import AlbumOptions from "./AlbumOptions.svelte";
</script>

<div class="header" data-tauri-drag-region>
    <h1
        class:window-controls-offset={!$isSidebarOpen &&
            !$isQueueOpen &&
            $os === "macos"}
        data-tauri-drag-region
    >
        {$LL.albums.title()}
    </h1>
    <!-- {#if count}<p>{count} {count === 1 ? "album" : "albums"}</p>{/if} -->
    <AlbumOptions />
</div>

<style lang="scss">
    .header {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 20px;
        grid-column: 1 /3;
        grid-row: 1;

        h1 {
            margin: 0;
            font-family: "Snake";
            flex-grow: 1;
            text-align: left;
            font-size: 3em;
            opacity: 0.3;
            margin-left: 0.1em;
            color: var(--header-text);
            opacity: var(--header-opacity);

            &.window-controls-offset {
                margin-left: 70px;
            }
        }
    }
</style>
