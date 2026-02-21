<script lang="ts">
    import { debounce } from "lodash-es";
    import { Toaster } from "svelte-french-toast";
    import {
        bottomBarNotification,
        canShowInfoPopup,
        draggedScrapbookItems,
        draggedSongs,
        droppedFiles,
        emptyDropEvent,
        fileToDownload,
        foldersToWatch,
        genericSongOrder,
        hoveredFiles,
        isCompactView,
        isLyricsOpen,
        isMiniPlayer,
        isQueueOpen,
        isSidebarFloating,
        isSidebarOpen,
        isSidebarShowing,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isWaveformOpen,
        isWikiOpen,
        librarySongOrder,
        os,
        popupOpen,
        selectedPlaylistFile,
        selectedSmartQuery,
        songToSeparate,
        uiView,
        uiViewToRestore,
    } from "./data/store";

    import { type UnlistenFn } from "@tauri-apps/api/event";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { onDestroy, onMount } from "svelte";
    import { getLocaleFromNavigator, init, register } from "svelte-i18n";
    import { cubicInOut, cubicOut } from "svelte/easing";
    import { blur, fade, fly } from "svelte/transition";
    import { startWatchingLibraryFolders } from "./data/FolderWatcher";
    import { importPaths, startImportListener } from "./data/LibraryUtils";
    import { findQuery } from "./data/SmartQueries";
    import { setLocale } from "./i18n/i18n-svelte";
    import { loadLocale } from "./i18n/i18n-util.sync";
    import AlbumsHeader from "./lib/albums/AlbumsHeader.svelte";
    import DownloadPopup from "./lib/internet-archive/DownloadPopup.svelte";
    import BottomBar from "./lib/library/BottomBar.svelte";
    import Dropzone from "./lib/library/Dropzone.svelte";
    import LyricsView from "./lib/library/LyricsView.svelte";
    import PlaylistHeader from "./lib/library/PlaylistHeader.svelte";
    import SmartPlaylistHeader from "./lib/library/SmartPlaylistHeader.svelte";
    import TagCloud from "./lib/library/TagCloud.svelte";
    import ToDeleteHeader from "./lib/library/ToDeleteHeader.svelte";
    import TrackInfoPopup from "./lib/info/TrackInfoPopup.svelte";
    import audioPlayer from "./lib/player/AudioPlayer";
    import InfoPopup from "./lib/settings/InfoPopup.svelte";
    import SettingsPopup from "./lib/settings/SettingsPopup.svelte";
    import Sidebar from "./lib/sidebar/Sidebar.svelte";
    import SmartQueryBuilder from "./lib/smart-query/SmartQueryBuilder.svelte";
    import CursorInfo from "./lib/ui/CursorInfo.svelte";
    import Icon from "./lib/ui/Icon.svelte";
    import AlbumView from "./lib/views/AlbumsView.svelte";
    import AnalyticsView from "./lib/views/AnalyticsView.svelte";
    import ArtistsToolkitView from "./lib/views/ArtistsToolkitView.svelte";
    import CanvasLibraryView from "./lib/views/CanvasLibraryView.svelte";
    import InternetArchiveView from "./lib/views/InternetArchiveView.svelte";
    import MapView from "./lib/views/MapView.svelte";
    import NotesView from "./lib/views/NotesView.svelte";
    import PrunePopup from "./lib/views/PrunePopup.svelte";
    import QueueView from "./lib/views/QueueView.svelte";
    import TopBar from "./lib/views/TopBar.svelte";
    import WikiView from "./lib/views/WikiView.svelte";
    import ThemeWrapper from "./theming/ThemeWrapper.svelte";
    import {
        startErrorListener,
        startMenuListener,
    } from "./window/EventListener";
    import { resetDraggedSongs } from "./data/storeHelper";
    import TopNav from "./lib/nav/TopNav.svelte";
    import { get } from "svelte/store";
    import type { Locales } from "./i18n/i18n-types";
    import StemSeparatePopup from "./lib/library/StemSeparatePopup.svelte";

    const appWindow = getCurrentWebviewWindow();

    let localeLoaded = false;

    function initLocale() {
        const savedLocale = (localStorage.getItem("locale") ||
            navigator.languages.find((l) => !l.includes("-")) ||
            "en") as Locales;
        console.log("locale", savedLocale);

        register("en", () => import("./i18n/en"));
        register("es", () => import("./i18n/es"));
        register("cn", () => import("./i18n/cn"));

        init({
            fallbackLocale: "en",
            initialLocale: savedLocale,
        });

        loadLocale(savedLocale);
        setLocale(savedLocale);
        localeLoaded = true;
    }

    startMenuListener();
    startImportListener();
    startErrorListener();

    let unlistenFileDrop: UnlistenFn;
    let unlistenFolderWatch: UnlistenFn;
    // function onDragEnter(e) {
    //     e.preventDefault();

    //     e.dataTransfer.dropEffect = "copy";
    //     isDraggingFiles.set(true);
    //     console.log("drag enter");
    // }

    function onCloseAppInfo() {
        if ($popupOpen === "info") {
            $popupOpen = null;
        }
    }

    let mouseX;
    let mouseY;

    let showDropzone = false;

    let wikiPanelSize = 500;
    const WIKI_PANEL_MIN_SIZE = 300;

    /**
     * Listen for native file drop and hover events here.
     *
     * How this will be handled depends on the current UI view shown:
     * - Main library view: Import track(s)
     * - Artist's toolkit view: Add to scrapbook or song project
     */
    onMount(async () => {
        initLocale();
        appWindow.emit("opened");
        // File associations: check for opened urls on the window
        // @ts-expect-error
        console.log("window opened urls: ", window.openedUrls);

        window["onFileOpen"] = (urls) => {
            console.log("onFileOpen: ", urls);
            audioPlayer.handleOpenedUrls(urls);
        };

        unlistenFileDrop = await appWindow.onDragDropEvent((evt) => {
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
                case "enter":
                    $hoveredFiles = evt.payload.paths;
                    console.log("files:", $hoveredFiles);
                    if ($uiView === "library" && $hoveredFiles.length) {
                        showDropzone = true;
                    }
                    break;
                case "leave":
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
            unlistenFolderWatch = await startWatchingLibraryFolders();
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
        resetDraggedSongs();

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

    let showMiniPlayer = window.innerHeight <= 220 && window.innerWidth <= 210;

    $: innerHeight = window.innerHeight;
    $: innerWidth = window.innerWidth;
    $: selectedQuery = findQuery($selectedSmartQuery);
    $: showCursorInfo = $draggedSongs.length > 0 && mouseX + mouseY > 0;
    $: showMainPanel = innerWidth >= 300;

    $: {
        if (innerWidth < 410 && !get(isCompactView)) {
            isCompactView.set(true);
            if (get(isQueueOpen)) {
                uiViewToRestore.set(get(uiView));
                uiView.set("queue");
            }
        } else if (innerWidth >= 410 && get(isCompactView)) {
            isCompactView.set(false);
            if (get(uiView) === "queue") {
                isQueueOpen.set(true);
                uiView.set(get(uiViewToRestore));
            } else {
                isQueueOpen.set(false);
            }
        }
    }

    $: {
        if (!$uiView.match(/^(queue|wiki)$/)) {
            uiViewToRestore.set($uiView);
        }
    }

    $: showQueue = !showMiniPlayer && !$isCompactView && $isQueueOpen;

    $: showSidebar =
        showMiniPlayer ||
        (innerHeight >= 650 &&
            $isSidebarOpen &&
            ((!$isWikiOpen && innerWidth >= 660) ||
                ($isWikiOpen &&
                    ((!showQueue && innerWidth >= 880) ||
                        (showQueue && innerWidth >= 1000)))));
    $: showWiki =
        $isWikiOpen &&
        ((!showQueue && innerWidth >= 500) || (showQueue && innerWidth >= 660));
    $: isQueueAutoWidth = !showMiniPlayer && innerWidth < 520;
    $: isSplitView = showMainPanel && showQueue && innerWidth < 520;

    $: if ($bottomBarNotification?.timeout) {
        setTimeout(() => {
            $bottomBarNotification = null;
        }, $bottomBarNotification.timeout);
    }

    $: {
        canShowInfoPopup.set(innerWidth >= 750);
    }

    $: {
        $isSidebarShowing = showSidebar;

        if (showSidebar) {
            $isSidebarFloating = false;
        }
    }

    let container: HTMLElement;
    let isResizing = false;
    let showCloseWikiPrompt = false;
    function onWikiResize(e: MouseEvent) {
        e.stopPropagation();
        e.preventDefault();
        const containerWidth = window.innerWidth;
        console.log("container width", containerWidth);
        console.log("clientX", e.pageX);
        if (containerWidth - e.clientX > WIKI_PANEL_MIN_SIZE) {
            showCloseWikiPrompt = false;
            wikiPanelSize = containerWidth - e.clientX;
        } else if (containerWidth - e.clientX < WIKI_PANEL_MIN_SIZE - 10) {
            showCloseWikiPrompt = true;
        }
    }
    function startResizeListener() {
        isResizing = true;

        container.addEventListener("mousemove", onWikiResize);
        document.addEventListener("mouseup", stopResizeListener);
    }
    function stopResizeListener() {
        if (isResizing) {
            isResizing = false;

            container.removeEventListener("mousemove", onWikiResize);
            container.removeEventListener("mouseup", stopResizeListener);

            if (showCloseWikiPrompt) {
                $isWikiOpen = false;
                showCloseWikiPrompt = false;
            }
        }
    }

    function onResize() {
        showMiniPlayer = window.innerHeight <= 220 && window.innerWidth <= 210;
        innerHeight = window.innerHeight;
        innerWidth = window.innerWidth;
    }
</script>

<svelte:window on:resize={debounce(onResize, 5)} />
{#if localeLoaded}
    <ThemeWrapper>
        <!-- <svelte:body on:click={onPageClick} /> -->
        <Toaster />

        <CursorInfo show={showCursorInfo} x={mouseX} y={mouseY} />

        {#if $popupOpen === "settings"}
            <div class="info">
                <SettingsPopup />
            </div>
        {:else if $popupOpen === "info"}
            <div class="info">
                <InfoPopup onClickOutside={onCloseAppInfo} />
            </div>
        {:else if $popupOpen === "track-info"}
            <div class="info">
                <TrackInfoPopup />
            </div>
        {/if}

        {#if showDropzone}
            <Dropzone />
        {/if}

        <main
            class:mini-player={$isMiniPlayer}
            class:queue-view={!showMainPanel && showQueue}
            class:split-view={isSplitView}
            class:floating-sidebar={!showSidebar}
            class:transparent={$os === "macos"}
            class:compact={$isCompactView}
            bind:this={container}
        >
            <div
                class="sidebar"
                class:visible={showSidebar || $isSidebarFloating}
                class:floating={!showSidebar}
            >
                {#if showSidebar || $isSidebarFloating}
                    <Sidebar floating={$isSidebarFloating} />
                {/if}
            </div>

            {#if $isSidebarFloating}
                <div class="sidebar-collapse">
                    <Icon
                        icon="tabler:layout-sidebar-left-collapse"
                        size={22}
                        onClick={() => {
                            $isSidebarFloating = false;
                        }}
                    />
                </div>
            {:else if !showSidebar}
                <div class="sidebar-expand">
                    <Icon
                        icon="tabler:layout-sidebar-left-expand"
                        size={22}
                        onClick={() => {
                            if (
                                !showMiniPlayer &&
                                (innerWidth < 660 || innerHeight < 650)
                            ) {
                                $isSidebarFloating = true;
                            } else {
                                $isSidebarOpen = true;
                            }
                        }}
                    />
                </div>
            {/if}

            <div class="queue" class:visible={showQueue}>
                {#if showQueue}
                    <div
                        class="queue-container"
                        transition:fly={{
                            duration: 200,
                            x: -150,
                            easing: cubicOut,
                        }}
                    >
                        <QueueView autoWidth={isQueueAutoWidth} />
                    </div>
                {/if}
            </div>

            <div class="top-nav" data-tauri-drag-region>
                {#if $isCompactView || !showSidebar}
                    <TopNav />
                {/if}
            </div>

            <div class="header">
                {#if $uiView === "playlists"}
                    <div class="content" data-tauri-drag-region>
                        {#if $selectedPlaylistFile}
                            <PlaylistHeader
                                playlist={$selectedPlaylistFile}
                                bind:songOrder={$genericSongOrder}
                            />
                        {/if}
                    </div>
                {:else if $uiView === "to-delete"}
                    <div class="content" data-tauri-drag-region>
                        <ToDeleteHeader />
                    </div>
                {:else if $uiView === "smart-query" || $uiView === "favourites"}
                    <div class="content" data-tauri-drag-region>
                        {#await selectedQuery then query}
                            <SmartPlaylistHeader selectedQuery={query} />
                        {/await}
                    </div>
                {:else if $uiView === "albums"}
                    <div class="content" data-tauri-drag-region>
                        <AlbumsHeader />
                    </div>
                {/if}
            </div>

            <div class="subheader">
                {#if $isTagCloudOpen}
                    <div
                        class="content"
                        transition:fly={{
                            y: -10,
                            duration: 200,
                            easing: cubicInOut,
                        }}
                    >
                        <TagCloud />
                    </div>
                {:else if $uiView.match(/^(smart-query)/)}
                    {#if $isSmartQueryBuilderOpen}
                        <div
                            class="content"
                            transition:fly={{
                                y: -10,
                                duration: 200,
                                easing: cubicInOut,
                            }}
                        >
                            <SmartQueryBuilder />
                        </div>
                    {/if}
                {/if}
            </div>

            {#if showMainPanel}
                <div class="panel {$uiView}" class:compact={$isCompactView}>
                    {#if $uiView === "library" || $uiView === "favourites"}
                        <CanvasLibraryView bind:songOrder={$librarySongOrder} />
                    {:else if $uiView === "playlists" || $uiView === "to-delete" || $uiView.startsWith("smart-query")}
                        <CanvasLibraryView bind:songOrder={$genericSongOrder} />
                    {:else if $uiView === "albums"}
                        <AlbumView />
                    {:else if $uiView === "your-music"}
                        <ArtistsToolkitView />
                    {:else if $uiView === "map"}
                        <MapView bind:songOrder={$librarySongOrder} />
                    {:else if $uiView === "analytics"}
                        <AnalyticsView bind:songOrder={$librarySongOrder} />
                    {:else if $uiView === "internet-archive"}
                        <InternetArchiveView />
                    {:else if $uiView === "prune"}
                        <PrunePopup />
                    {:else if $uiView === "queue"}<QueueView autoWidth={true} />
                    {:else if $uiView === "wiki"}
                        <div
                            class="wiki-container"
                            style={`width: 100%;overflow:hidden;`}
                            transition:fly={{ duration: 200, x: -200 }}
                        >
                            {#if showCloseWikiPrompt}
                                <div
                                    class="close-wiki-prompt"
                                    transition:blur={{ duration: 100 }}
                                >
                                    <h2>Close wiki</h2>
                                </div>
                            {/if}
                            <WikiView />
                        </div>
                    {/if}
                </div>
            {/if}

            {#if $isLyricsOpen}
                <div class="lyrics" transition:fade={{ duration: 150 }}>
                    <LyricsView right={$isWikiOpen ? wikiPanelSize + 15 : 0} />
                </div>
            {/if}
            <div class="waveform">
                <div transition:fly={{ duration: 200, y: 50 }}>
                    {#if $isWaveformOpen}
                        <NotesView />
                    {/if}
                </div>
            </div>

            <div class="wiki">
                {#if showWiki}
                    <div
                        class="wiki-container"
                        style={`width: ${wikiPanelSize}px;`}
                        transition:fly={{ duration: 200, x: -200 }}
                    >
                        {#if showCloseWikiPrompt}
                            <div
                                class="close-wiki-prompt"
                                transition:blur={{ duration: 100 }}
                            >
                                <h2>Close wiki</h2>
                            </div>
                        {/if}
                        <WikiView />
                    </div>
                {/if}
            </div>

            <div class="bottom-bar" class:full-width={!showSidebar}>
                {#if !showSidebar}
                    <div class="top" in:fly={{ duration: 200, y: 30 }}>
                        <TopBar />
                    </div>
                {/if}
                <div class="bottom" in:fly={{ duration: 200, y: -30 }}>
                    <BottomBar />
                </div>
            </div>

            {#if $fileToDownload}
                <DownloadPopup />
            {/if}

            {#if $songToSeparate}
                <StemSeparatePopup />
            {/if}

            {#if showWiki}
                <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                <resize-handle
                    role="separator"
                    on:mousedown={startResizeListener}
                    class:resizing={isResizing}
                />
            {/if}
        </main>
    </ThemeWrapper>
{/if}

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

    * {
        user-select: none;
    }

    main {
        border-radius: 11px;
        border: 0.5px solid
            color-mix(in srgb, var(--type-bw-inverse) 20%, transparent);
        box-sizing: border-box;
        display: grid;
        grid-template-columns: auto auto 1fr auto auto; // Sidebar, queue, panel, resizer, wiki
        grid-template-rows: auto auto auto 1fr auto auto; // (padding), header, tags/smartplaylist builder, panel, waveform, topbar, bottombar
        width: 100vw;
        height: 100vh;
        opacity: 1;
        position: relative;
        background-color: var(--background, initial);

        @media screen and (max-width: 210px) and (max-height: 210px) {
            grid-template-columns: auto; // Sidebar, queue, panel, resizer, wiki
        }

        &.mini-player {
            border-radius: 5px;
            overflow: hidden;
        }

        .sidebar {
            width: 100%;
            grid-row: 1 / 8;
            grid-column: 1;
            width: 0;
            overflow: hidden;
            &.visible {
                width: 210px;
            }
            &.floating {
                position: fixed;
                z-index: 16;
                height: 100vh;
                background-color: color-mix(
                    in srgb,
                    var(--background) 100%,
                    transparent 0%
                );
            }
        }

        .sidebar-expand,
        .sidebar-collapse {
            position: absolute;
            margin: auto;
            z-index: 20;
            transition: all 0.1s ease-in-out;
            top: 50%;
        }

        .sidebar-expand {
            left: -12px;

            &:hover {
                transform: scale(1.3) translateX(5px);
            }
        }

        .sidebar-collapse {
            left: 198px;

            &:hover {
                transform: scale(1.3);
            }
        }

        @keyframes appear {
            from {
                opacity: 0;
            }
            to {
                opacity: 0.6;
            }
        }

        .top-nav {
            grid-row: 1;
            grid-column: 2 / 6;
            div {
                height: 30px;
            }
        }

        .nav {
            grid-row: 2;
            grid-column: 3;
        }

        &.compact .header > .content {
            margin-top: -22px;
        }

        .header {
            grid-row: 2;
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
        .subheader {
            grid-row: 3;
            grid-column: 3;

            .content {
                width: 100%;
                margin: 5px 0 2px 0px;
            }
        }

        .panel {
            grid-row: 4;
            grid-column: 3;
            display: grid;
            overflow: hidden;
            box-shadow: -4px 3px 20px 5px var(--panel-shadow-bg);
            border: 1px solid var(--panel-primary-border-accent1);
            border-bottom-left-radius: 5px;
            border-bottom-right-radius: 5px;
            border-bottom: 0.7px solid var(--panel-secondary-border-main);
            margin: 3.5px 5px 0 0;
            border-radius: 5px;
            box-sizing: border-box;

            &.internet-archive,
            &.your-music {
                border: none;
            }

            &.compact {
                box-shadow: -4px 8px 20px 5px var(--panel-shadow-bg);
            }
        }

        .waveform {
            grid-row: 5;
            grid-column: 2 / 6;
        }

        .top-bar {
            grid-row: 6;
            grid-column: 2 / 6;
        }

        .bottom-bar {
            position: relative;
            width: 100%;
            z-index: 15;
            grid-row: 7;
            grid-column: 2 / 6;
            @media only screen and (max-width: 600px) {
                .top {
                    margin: 5px 0 5px 5px;
                }
                .bottom {
                    display: none;
                }
            }
            .bottom {
                margin-top: 5px;
                margin-bottom: 5px;

                @media only screen and (max-height: 649px) {
                    margin: 5px;
                }
            }

            &.full-width {
                grid-column: 1 / -1;
            }
        }

        .queue:not(.panel.queue) {
            grid-row: 2/5;
            grid-column: 2;
            overflow: hidden;
            height: 100%;

            &.visible {
                box-shadow: -10px 3px 30px 5px var(--panel-shadow-bg);
            }

            .queue-container {
                height: 100%;
                box-sizing: border-box;
                overflow: hidden;
                border-radius: 5px;
                margin: 4px 8px 0 0;
                display: grid;
                grid-template-rows: 1fr auto;
                gap: 5px;
            }
        }

        .wiki:not(.panel.wiki) {
            grid-row: 2/5;
            grid-column: 5;
            overflow-y: hidden;
            height: 100%;
            position: relative;
            .wiki-container {
                height: 100%;
                box-sizing: border-box;
                overflow: hidden;
                border-bottom: 0.7px solid #ffffff36;
                border-radius: 5px;
                margin: 0px 8px 0 0;
            }

            .close-wiki-prompt {
                position: absolute;
                top: 0;
                bottom: 0;
                left: 0;
                right: 0;
                display: flex;
                align-items: center;
                justify-content: center;
                color: var(--wiki-close-prompt-text);
                background-color: var(--wiki-close-prompt-bg);
                backdrop-filter: blur(5px);
                z-index: 10;
            }
        }

        resize-handle {
            grid-column: 4;
            grid-row: 3 / 6;
            display: block;
            position: relative;
            height: 100%;
            width: 4px;
            left: -2.5px;
            z-index: 100;
            cursor: ew-resize;
            @media screen and (max-width: 210px) {
                display: none;
            }

            &::after {
                content: "";
                display: block;
                position: absolute;
                margin: auto;
                top: 0;
                bottom: 0;
                left: 0;
                right: 0;
                width: 4px;
                box-sizing: border-box;
                background: url("/images/resize-handle.svg") no-repeat center;
                opacity: 0.3;
                z-index: 10;
            }
            &:hover,
            &.resizing {
                background-color: var(--accent-secondary);
            }
        }

        &.floating-sidebar {
            grid-template-columns: auto 1fr auto auto; // queue, panel, resizer, wiki

            .queue:not(.panel.queue) {
                grid-column: 1;
                margin-left: 5px;
            }

            .panel,
            .header {
                grid-column: 2;
            }

            resize-handle {
                grid-column: 3;
            }

            .wiki {
                grid-column: 4;
            }

            .waveform {
                grid-column: 1 / 4;
                margin-left: 5px;
            }
            .top-nav {
                grid-column: 1 / 4;
            }
            .bottom-bar {
                :global(.bottom) {
                    margin-left: 5px;
                }
            }
        }

        &.split-view {
            grid-template-columns: 1fr 1fr; // queue, panel

            .queue {
                grid-column: 1;

                .queue-container {
                    margin: 0 4px 0 0;
                }
            }

            .panel,
            .header {
                grid-column: 2;
            }
        }

        &.queue-view {
            .queue {
                grid-column: 1 / -1;
                margin: 0;

                .queue-container {
                    margin: 0 4px;
                }
            }
        }
    }

    .info {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 1000;
        display: flex;
        background-color: var(--popup-backdrop);
    }
</style>
