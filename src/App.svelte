<script lang="ts">
    import { Toaster } from "svelte-french-toast";
    import {
        isDraggingFiles,
        isInfoPopupOpen,
        isMiniPlayer, isSettingsOpen, isTrackInfoPopupOpen, uiView
    } from "./data/store";

    import { onMount } from "svelte";
    import Dropzone from "./lib/Dropzone.svelte";
    import InfoPopup from "./lib/InfoPopup.svelte";
    import SettingsPopup from "./lib/SettingsPopup.svelte";
    import Sidebar from "./lib/Sidebar.svelte";
    import TrackInfoPopup from "./lib/TrackInfoPopup.svelte";
    import ArtistsToolkitView from "./lib/views/ArtistsToolkitView.svelte";
    import LibraryView from "./lib/views/LibraryView.svelte";
    import { startMenuListener } from "./window/EventListener";

    startMenuListener();

    function onDragEnter(e) {
        e.preventDefault();

        e.dataTransfer.dropEffect = "copy";
        isDraggingFiles.set(true);
        console.log("drag enter");
    }

    function onPageClick() {
        $isInfoPopupOpen = false;
    }

    // App start
    onMount(() => {
        
    })
</script>

<!-- <svelte:body on:click={onPageClick} /> -->
<Toaster />

{#if $isSettingsOpen}
    <div class="info">
        <SettingsPopup />
    </div>
{/if}

{#if $isInfoPopupOpen}
    <div class="info">
        <InfoPopup />
    </div>
{/if}

{#if $isTrackInfoPopupOpen}
    <div class="info">
        <TrackInfoPopup />
    </div>
{/if}

{#if $isDraggingFiles}
    <Dropzone />
{/if}

<main on:dragenter={onDragEnter} class:mini-player={$isMiniPlayer}>
    <Sidebar />

    {#if $uiView === "library" || $uiView === 'smart-query'}
        <LibraryView />
    {:else if $uiView === "your-music"}
        <ArtistsToolkitView />
    {/if}
</main>

<style lang="scss">
    main {
        display: grid;
        grid-template-columns: auto 1fr;
        width: 100vw;
        height: 100vh;
        opacity: 1;
        position: relative;
        background-color: #242026c2;

        &.mini-player {
            border-radius: 5px;
            overflow: hidden;
        }

        @media only screen and (max-width: 320px) {
            grid-template-columns: 1fr;
        }
    }

    .info {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 30;
        display: flex;
        background-color: #242026b9;
    }
</style>
