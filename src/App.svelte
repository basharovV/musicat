<script lang="ts">
    import { Toaster } from "svelte-french-toast";
    import {
        draggedScrapbookItems,
        draggedSongs,
        droppedFiles,
        emptyDropEvent,
        hoveredFiles,
        isDraggingExternalFiles,
        isInfoPopupOpen,
        isMiniPlayer,
        isSettingsOpen,
        isTrackInfoPopupOpen,
        isWelcomeSeen,
        uiView
    } from "./data/store";

    import { type UnlistenFn } from "@tauri-apps/api/event";
    import { appWindow } from "@tauri-apps/api/window";
    import { onDestroy, onMount } from "svelte";
    import Dropzone from "./lib/Dropzone.svelte";
    import InfoPopup from "./lib/InfoPopup.svelte";
    import SettingsPopup from "./lib/SettingsPopup.svelte";
    import Sidebar from "./lib/Sidebar.svelte";
    import TrackInfoPopup from "./lib/TrackInfoPopup.svelte";
    import ArtistsToolkitView from "./lib/views/ArtistsToolkitView.svelte";
    import LibraryView from "./lib/views/LibraryView.svelte";
    import { startMenuListener } from "./window/EventListener";
    import { addFolder, addPaths } from "./data/LibraryImporter";
    import AlbumView from "./lib/views/AlbumsView.svelte";
    import WelcomeView from "./lib/views/WelcomeView.svelte";
    import CursorInfo from "./lib/ui/CursorInfo.svelte";
    import MapView from "./lib/views/MapView.svelte";
    import AnalyticsView from "./lib/views/AnalyticsView.svelte";

    startMenuListener();

    let unlistenFileDrop: UnlistenFn;

    // function onDragEnter(e) {
    //     e.preventDefault();

    //     e.dataTransfer.dropEffect = "copy";
    //     isDraggingFiles.set(true);
    //     console.log("drag enter");
    // }

    function onCloseAppInfo() {
        if ($isInfoPopupOpen) {
            $isInfoPopupOpen = false;
        }
    }

    let mouseX;
    let mouseY;

    let showDropzone = false;

    /**
     * Listen for native file drop and hover events here.
     *
     * How this will be handled depends on the current UI view shown:
     * - Main library view: Import track(s)
     * - Artist's toolkit view: Add to scrapbook or song project
     */
    onMount(async () => {
        unlistenFileDrop = await appWindow.onFileDropEvent((evt) => {
            switch (evt.payload.type) {
                case "drop":
                    showDropzone = false;
                    console.log("paths:", evt.payload);
                    if (evt.payload.paths.length > 0) {
                        if ($uiView === "library") {
                            addPaths(evt.payload.paths);
                        } else if ($uiView === "your-music") {
                            $droppedFiles = evt.payload.paths;
                        }
                    } else {
                        // This is a temporary hack to get internal drag and drop working,
                        // while also supporting external files dropped into the app.
                        $emptyDropEvent = { x: mouseX, y: mouseY };
                    }

                    // Let the destination ui handle the files, and set the array to empty again.
                    break;
                case "hover":
                    $hoveredFiles = evt.payload.paths;
                    console.log("files:", $hoveredFiles);
                    if ($uiView === "library") {
                        showDropzone = true;
                    }
                    break;
                case "cancel":
                    $droppedFiles = [];
                    $hoveredFiles = [];
                    if ($uiView === "library") {
                        showDropzone = false;
                    }
                    console.log("files:", $hoveredFiles);
                    break;
            }
        });
    });

    function onDragMove(evt: MouseEvent) {
        mouseX = evt.x;
        mouseY = evt.y;
    }

    draggedScrapbookItems.subscribe((items) => {
        if (items.length > 0) {
            // Track mouse position to see where it gets dropped (hack)
            document.addEventListener("dragover", onDragMove);
        } else {
            document.removeEventListener("dragover", onDragMove);
        }
    });

    function onMouseMove(evt: MouseEvent) {
        mouseX = evt.clientX;
        mouseY = evt.clientY;
    }

    function onMouseUp() {
        $draggedSongs = [];
        mouseX = 0;
        mouseY = 0;
    }

    draggedSongs.subscribe((songs) => {
        if (songs.length > 0) {
            // Track mouse position to see where it gets dropped
            document.addEventListener("mousemove", onMouseMove);
            document.addEventListener("mouseup", onMouseUp);
        } else {
            document.removeEventListener("mousemove", onMouseMove);
            document.removeEventListener("mouseup", onMouseUp);
        }
    });

    onDestroy(() => {
        unlistenFileDrop();
    });

    $: showCursorInfo = $draggedSongs.length > 0;
</script>

<!-- <svelte:body on:click={onPageClick} /> -->
<Toaster />

<CursorInfo show={showCursorInfo} x={mouseX} y={mouseY} />

{#if $isSettingsOpen}
    <div class="info">
        <SettingsPopup />
    </div>
{/if}

{#if $isInfoPopupOpen}
    <div class="info">
        <InfoPopup onClickOutside={onCloseAppInfo} />
    </div>
{/if}

{#if $isTrackInfoPopupOpen}
    <div class="info">
        <TrackInfoPopup />
    </div>
{/if}

{#if showDropzone}
    <Dropzone />
{/if}

<main class:mini-player={$isMiniPlayer}>
    {#if $isWelcomeSeen}
        <Sidebar />
    {/if}
    {#if !$isWelcomeSeen}
        <WelcomeView />
    {:else if $uiView === "library" || $uiView === "smart-query"}
        <LibraryView />
    {:else if $uiView === "albums"}
        <AlbumView />
    {:else if $uiView === "your-music"}
        <ArtistsToolkitView />
    {:else if $uiView === "map"}
        <MapView />
    {:else if $uiView === "analytics"}
        <AnalyticsView />
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
