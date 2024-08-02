<script lang="ts">
    import { Toaster } from "svelte-french-toast";
    import {
        bottomBarNotification,
        draggedAlbum,
        draggedScrapbookItems,
        draggedSongs,
        droppedFiles,
        emptyDropEvent,
        fileToDownload,
        foldersToWatch,
        hoveredFiles,
        isInfoPopupOpen,
        isLyricsOpen,
        isMiniPlayer,
        isQueueOpen,
        isSettingsOpen,
        isTrackInfoPopupOpen,
        isWaveformOpen,
        os,
        selectedPlaylistId,
        selectedSmartQuery,
        uiView
    } from "./data/store";

    import { type UnlistenFn } from "@tauri-apps/api/event";
    import { appWindow } from "@tauri-apps/api/window";
    import { onDestroy, onMount } from "svelte";
    import { getLocaleFromNavigator, init, register } from "svelte-i18n";
    import { fade, fly } from "svelte/transition";
    import { db } from "./data/db";
    import { startWatching } from "./data/FolderWatcher";
    import { importPaths, startImportListener } from "./data/LibraryImporter";
    import { findQuery } from "./data/SmartQueries";
    import { setLocale } from "./i18n/i18n-svelte";
    import { loadLocale } from "./i18n/i18n-util.sync";
    import DownloadPopup from "./lib/internet-archive/DownloadPopup.svelte";
    import BottomBar from "./lib/library/BottomBar.svelte";
    import Dropzone from "./lib/library/Dropzone.svelte";
    import LyricsView from "./lib/library/LyricsView.svelte";
    import PlaylistHeader from "./lib/library/PlaylistHeader.svelte";
    import SmartPlaylistHeader from "./lib/library/SmartPlaylistHeader.svelte";
    import TrackInfoPopup from "./lib/library/TrackInfoPopup.svelte";
    import InfoPopup from "./lib/settings/InfoPopup.svelte";
    import SettingsPopup from "./lib/settings/SettingsPopup.svelte";
    import Sidebar from "./lib/sidebar/Sidebar.svelte";
    import CursorInfo from "./lib/ui/CursorInfo.svelte";
    import AlbumView from "./lib/views/AlbumsView.svelte";
    import AnalyticsView from "./lib/views/AnalyticsView.svelte";
    import ArtistsToolkitView from "./lib/views/ArtistsToolkitView.svelte";
    import CanvasLibraryView from "./lib/views/CanvasLibraryView.svelte";
    import InternetArchiveView from "./lib/views/InternetArchiveView.svelte";
    import MapView from "./lib/views/MapView.svelte";
    import NotesView from "./lib/views/NotesView.svelte";
    import QueueOptions from "./lib/views/QueueOptions.svelte";
    import QueueView from "./lib/views/QueueView.svelte";
    import ThemeWrapper from "./theming/ThemeWrapper.svelte";
    import { startMenuListener } from "./window/EventListener";

    console.log("locale", getLocaleFromNavigator());

    register("en", () => import("./i18n/en"));
    register("es", () => import("./i18n/es"));

    init({
        fallbackLocale: "en",
        initialLocale: getLocaleFromNavigator()
    });

    loadLocale("en");
    setLocale("en");

    startMenuListener();
    startImportListener();

    let unlistenFileDrop: UnlistenFn;
    let unlistenFolderWatch: UnlistenFn;
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
                            importPaths(evt.payload.paths, true);
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
                    if ($uiView === "library" && $hoveredFiles.length) {
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

        foldersToWatch.subscribe(async (_) => {
            unlistenFolderWatch && unlistenFolderWatch();
            unlistenFolderWatch = await startWatching();
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
        $draggedAlbum = null;
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
            mouseX = 0;
            mouseY = 0;
        }
    });

    onDestroy(() => {
        unlistenFileDrop();
        unlistenFolderWatch();
    });

    $: showCursorInfo = $draggedSongs.length > 0 && mouseX + mouseY > 0;

    $: selectedPlaylist = db.playlists.get($selectedPlaylistId);
    $: selectedQuery = findQuery($selectedSmartQuery);

    $: if ($bottomBarNotification?.timeout) {
        setTimeout(() => {
            $bottomBarNotification = null;
        }, $bottomBarNotification.timeout);
    }
</script>

<ThemeWrapper>
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

    <main
        class:mini-player={$isMiniPlayer}
        class:transparent={$os === "Darwin"}
    >
        <div class="sidebar">
            <Sidebar />
        </div>

        <div class="queue">
            {#if $isQueueOpen}
                <div
                    class="queue-container"
                    transition:fly={{ duration: 200, x: -200 }}
                >
                    <!-- <QueueView /> -->
                    <QueueView />
                    <QueueOptions />
                </div>
            {/if}
        </div>

        <div class="header">
            {#if $uiView === "playlists"}
                <!-- <p class="label">playlist:</p> -->
                <div class="content">
                    {#await selectedPlaylist then playlist}
                        <PlaylistHeader {playlist} />
                    {/await}
                </div>
            {:else if $uiView === "smart-query"}
                <!-- <p class="label">playlist:</p> -->
                <div class="content">
                    {#await selectedQuery then query}
                        <SmartPlaylistHeader selectedQuery={query} />
                    {/await}
                </div>
            {/if}
        </div>

        <div class="panel">
            {#if $uiView === "library" || $uiView.match(/^(smart-query|favourites)/)}
                <CanvasLibraryView />
            {:else if $uiView === "playlists"}
                <CanvasLibraryView />
            {:else if $uiView === "albums"}
                <AlbumView />
            {:else if $uiView === "your-music"}
                <ArtistsToolkitView />
            {:else if $uiView === "map"}
                <MapView />
            {:else if $uiView === "analytics"}
                <AnalyticsView />
            {:else if $uiView === "internet-archive"}
                <InternetArchiveView />
            {/if}
        </div>

        {#if $isLyricsOpen}
            <div class="lyrics" transition:fade={{ duration: 150 }}>
                <LyricsView />
            </div>
        {/if}

        {#if $isWaveformOpen}
            <div class="notes" transition:fly={{ duration: 200, y: 50 }}>
                <NotesView />
            </div>
        {/if}

        <div class="bottom-bar">
            <BottomBar />
        </div>

        {#if $fileToDownload}
            <DownloadPopup />
        {/if}
    </main>
</ThemeWrapper>

<style lang="scss">
    :global(html) {
        background-color: var(--background, initial);
        color: var(--text, initial);
        font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
        font-size: 14px;
        line-height: 24px;
        font-weight: 400;
        color-scheme: light dark;

        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
    }

    main {
        display: grid;
        grid-template-columns: auto auto 1fr;
        grid-template-rows: auto 1fr auto auto;
        width: 100vw;
        height: 100vh;
        opacity: 1;
        position: relative;
        background-color: var(--background, initial);

        /* &.transparent {
            background-color: color-mix(in srgb, var(--background, initial) 86%, transparent);
        } */

        &.mini-player {
            border-radius: 5px;
            overflow: hidden;
        }

        .sidebar {
            width: 100%;
            grid-row: 1 / 3;
            grid-column: 1;
        }

        @media only screen and (max-width: 320px) {
            grid-template-columns: 1fr;
        }
        @media only screen and (max-width: 320px) and (min-height: 300px) {
            grid-template-columns: 1fr;
            .sidebar {
                display: none;
            }
            padding-top: 2em;
        }

        .header {
            grid-row: 1;
            grid-column: 3;

            .content {
                display: flex;
                flex-direction: row;
                align-items: center;
                justify-content: space-between;
                gap: 8px;
                height: 28px;
                margin: 5px 5px 2px 0px;
            }
        }

        .panel {
            grid-row: 2;
            grid-column: 3;
            display: grid;
            overflow: hidden;
        }

        .notes {
            grid-row: 3;
            grid-column: 2 / 4;
        }

        .bottom-bar {
            position: relative;
            width: 100%;
            z-index: 15;
            grid-row: 4;
            grid-column: 2 / 4;
            margin-top: 5px;
            margin-bottom: 5px;
        }

        .queue {
            grid-row: 1/3;
            grid-column: 2;
            overflow: hidden;
            height: 100%;

            .queue-container {
                height: 100%;
                box-sizing: border-box;
                overflow: hidden;
                /* border-bottom: 0.7px solid #ffffff36; */
                border-radius: 5px;
                margin: 0px 7.5px 0 0;
                display: grid;
                grid-template-rows: 1fr auto;
                gap: 5px;
                /* display: grid; */
            }
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
        background-color: rgba(30, 26, 31, 0.824);
    }
</style>
