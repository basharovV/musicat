<script lang="ts">
    import { window as tauriWindow } from "@tauri-apps/api";
    import { emit } from "@tauri-apps/api/event";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import {
        appWindow, currentMonitor, LogicalSize, PhysicalPosition
    } from "@tauri-apps/api/window";
    import hotkeys from "hotkeys-js";
    import { throttle } from "lodash-es";
    import * as musicMetadata from "music-metadata-browser";
    import { onMount } from "svelte";
    import tippy from "svelte-tippy";
    import { lookForArt } from "../data/LibraryImporter";
    import {
        currentSong,
        currentSongArtworkSrc,
        currentSongIdx,
        isInfoPopupOpen,
        isMiniPlayer,
        isPlaying, isTrackInfoPopupOpen,
        os,
        queriedSongs,
        query,
        rightClickedTrack, singleKeyShortcutsEnabled, uiView, userSettings,
        volume
    } from "../data/store";
    import AudioPlayer from "./AudioPlayer";
    import Seekbar from "./Seekbar.svelte";
    import SpectrumAnalyzer from "./SpectrumAnalyzer.svelte";
    import "./tippy.css";

    // What to show in the sidebar
    let title;
    let artist;
    let album;
    let artworkFormat;
    let artworkBuffer: Buffer;
    let artworkSrc;
    let codec;

    let duration;

    $: if (codec === "MPEG 1 Layer 3") {
        codec = "MP3";
    }

    let bitrate;
    let sampleRate;

    // Shortcuts
    hotkeys("space", function (event, handler) {
        // Prevent the default refresh event under WINDOWS system
        event.preventDefault();
        AudioPlayer.togglePlay();
    });

    // Shortcuts
    if ($os === "Darwin") {
        hotkeys("cmd+f", function (event, handler) {
            searchInput.focus();
        });
    } else if ($os === "Windows_NT" || $os === "Linux") {
        hotkeys("ctrl+f", function (event, handler) {
            searchInput.focus();
        });
    }

    currentSong.subscribe(async (song) => {
        if (song) {
            const metadata = await musicMetadata.fetchFromUrl(
                convertFileSrc(song.path)
            );
            console.log("metadata", metadata);
            title = metadata.common.title;
            artist = metadata.common.artist;
            album = metadata.common.album;
            codec = metadata.format.codec;
            bitrate = metadata.format.bitsPerSample;
            sampleRate = metadata.format.sampleRate;
            duration = metadata.format.duration;
            if (metadata.common.picture?.length) {
                artworkFormat = metadata.common.picture[0].format;
                artworkBuffer = metadata.common.picture[0].data;
                artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                    "base64"
                )}`;
                // console.log("artworkSrc", artworkSrc);
                $currentSongArtworkSrc = {
                    src: artworkSrc,
                    format: artworkFormat,
                    size: {
                        width: metadata.common.picture[0]["width"],
                        height: metadata.common.picture[0]["height"]
                    }
                };
            } else {
                artworkSrc = null;
                const artwork = await lookForArt(
                    $currentSong.path,
                    $currentSong.file
                );
                if (artwork) {
                    artworkSrc = artwork.artworkSrc;
                    artworkFormat = artwork.artworkFormat;
                    $currentSongArtworkSrc = {
                        src: artworkSrc,
                        format: artworkFormat,
                        size: {
                            width: 200,
                            height: 200
                        }
                    };
                }
            }
        }
    });

    function togglePlayPause() {
        if (!AudioPlayer.audioFile.src) {
            AudioPlayer.playSong($queriedSongs[$currentSongIdx]);
        } else {
            AudioPlayer.togglePlay();
        }
    }

    function playNext() {
        AudioPlayer.playSong($queriedSongs[++$currentSongIdx]);
    }

    function playPrev() {
        if ($currentSongIdx === 1) {
            return;
        }
        AudioPlayer.playSong($queriedSongs[--$currentSongIdx]);
    }

    function openTrackInfo() {
        if ($currentSong) {
            $rightClickedTrack = $currentSong;
            $isTrackInfoPopupOpen = true;
        }
    }

    function openInfoWindow() {
        // const webview = new WebviewWindow("theUniqueLabel", {
        //   url: "path/to/page.html",
        // });
        console.log("clicked");
        $isInfoPopupOpen = true;
    }
    let height = 0;
    let width = 0;
    let hasDecorations = false;

    async function onResize() {
        height = window.innerHeight;
        width = window.innerWidth;
        hasDecorations = await tauriWindow.getCurrent().isDecorated();
        if (!$isMiniPlayer && height <= 220 && width <= 210) {
            $isMiniPlayer = true;
            console.log("setting to false");
            // await tauriWindow.getCurrent().setDecorations(false);
            emit("hide-toolbar");
        } else if ($isMiniPlayer && (height > 220 || width > 210)) {
            $isMiniPlayer = false;
            console.log("setting to true");
            emit("show-toolbar");
            // await tauriWindow.getCurrent().setDecorations(true);
        }
    }

    let searchInput: HTMLInputElement;

    onMount(() => {
        height = window.innerHeight;
        window.onresize = throttle(() => {
            onResize();
        }, 200);
        onResize(); // run once
        searchInput.onfocus = (evt) => {
            $singleKeyShortcutsEnabled = false;
        };
        searchInput.onblur = (evt) => {
            $singleKeyShortcutsEnabled = true;
        };
    });

    let miniToggleBtn: HTMLElement;
    let widthToRestore = 0;
    let heightToRestore = 0;
    let paddingPx = 40;
    let isMiniToggleHovered = false;
    let isMiniPlayerHovered = false;

    /**
     * We handle the hover event manually because otherwise
     * the hover state gets stuck when resizing and moving the window
     */
    function onMiniToggleMouseOver() {
        isMiniToggleHovered = true;
    }
    function onMiniToggleMouseOut() {
        isMiniToggleHovered = false;
    }
    function onMiniPlayerMouseOver() {
        isMiniPlayerHovered = true;
    }
    function onMiniPlayerMouseOut() {
        isMiniPlayerHovered = false;
    }

    async function toggleMiniPlayer() {
        if (!$isMiniPlayer) {
            widthToRestore = window.innerWidth;
            heightToRestore = window.innerHeight;

            emit("hide-toolbar");
            await tauriWindow.getCurrent().hide();
            await tauriWindow.getCurrent().setSize(new LogicalSize(210, 210));
            const monitor = await currentMonitor();
            const windowSize = await tauriWindow.getCurrent().innerSize();
            switch ($userSettings.miniPlayerLocation) {
                case "bottom-left":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x + paddingPx,
                                monitor.position.y +
                                    monitor.size.height -
                                    windowSize.height -
                                    paddingPx
                            )
                        );
                    break;
                case "bottom-right":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x +
                                    monitor.size.width -
                                    windowSize.width -
                                    paddingPx,
                                monitor.position.y +
                                    monitor.size.height -
                                    windowSize.height -
                                    paddingPx
                            )
                        );
                    break;
                case "top-left":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x + paddingPx,
                                monitor.position.y +
                                    ($os === "Darwin"
                                        ? paddingPx + 40
                                        : paddingPx)
                            )
                        );
                    break;
                case "top-right":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x +
                                    monitor.size.width -
                                    windowSize.width -
                                    paddingPx,
                                monitor.position.y +
                                    ($os === "Darwin"
                                        ? paddingPx + 40
                                        : paddingPx)
                            )
                        );
                    break;
            }

            await tauriWindow.getCurrent().show();
            await tauriWindow.getCurrent().setAlwaysOnTop(true);
            isMiniPlayerHovered = false; // By default we want to show the pretty artwork
        } else {
            await tauriWindow.getCurrent().hide();
            if (widthToRestore && heightToRestore) {
                await tauriWindow
                    .getCurrent()
                    .setSize(new LogicalSize(widthToRestore, heightToRestore));
            } else {
                await tauriWindow
                    .getCurrent()
                    .setSize(new LogicalSize(1100, 750));
            }

            await tauriWindow.getCurrent().center();
            await tauriWindow.getCurrent().show();
            await tauriWindow.getCurrent().setAlwaysOnTop(false);
        }

        isMiniToggleHovered = false;
    }

    appWindow.listen("tauri://focus", evt => {
        isMiniPlayerHovered = true;
    })
</script>

<!-- svelte-ignore a11y-mouse-events-have-key-events -->
<sidebar
    class:has-current-song={$currentSong}
    class:empty={!$currentSong}
    class:hovered={isMiniPlayerHovered}
    on:mouseenter|preventDefault|stopPropagation={onMiniPlayerMouseOver}
    on:mouseleave={onMiniPlayerMouseOut}
    data-tauri-drag-region
>
    <!-- <div class="knob">
    <Knob bind:value={volumeKnob} max={100} min={0} pixelRange={200} />
  </div> -->
    <div class="top">
        <div class="top-header">
            <h1 class="app-title" on:click={openInfoWindow}>Musicat</h1>
            <div class="search-container">
                <input
                    bind:this={searchInput}
                    class="search"
                    type="text"
                    placeholder="Search ({$os === 'Darwin'
                        ? 'Cmd + F'
                        : 'Ctrl + F'})"
                    bind:value={$query.query}
                />
                <div class="search-icon">
                    <iconify-icon icon="ion:search" />
                </div>
            </div>
        </div>

        <menu>
            <items>
                <item
                    class:selected={$uiView === "library"}
                    on:click={() => {
                        $uiView = "library";
                    }}
                >
                    <iconify-icon
                        icon="fluent:library-20-filled"
                    />Library</item
                >
                <item
                    class:selected={$uiView === "smart-query"}
                    on:click={() => {
                        $uiView = "smart-query";
                    }}
                >
                    <iconify-icon icon="fluent:search-20-filled" />Smart Query</item
                >
                <item
                    class:selected={$uiView === "your-music"}
                    on:click={() => {
                        $uiView = "your-music";
                    }}
                >
                    <iconify-icon icon="mdi:music-clef-treble" />Artist's
                    toolkit</item
                >
                <!-- <item> <iconify-icon icon="mdi:playlist-music" />Playlists</item> -->

                <!-- <hr />
    <item>
      <iconify-icon
        icon="iconoir:album-carousel"
      />Samples</item
    > -->
            </items>
        </menu>
    </div>

    <div class="track-info">
        <!-- <hr /> -->

        <div class="track-info-content">
            <!-- svelte-ignore a11y-mouse-events-have-key-events -->
            <iconify-icon
                use:tippy={{
                    theme: $isMiniPlayer ? "hidden" : "",
                    content: "Toggle the mini player.",
                    placement: "right"
                }}
                bind:this={miniToggleBtn}
                class="mini-toggle"
                class:hovered={isMiniToggleHovered}
                on:mouseover={onMiniToggleMouseOver}
                on:mouseout={onMiniToggleMouseOut}
                icon={$isMiniPlayer
                    ? "gg:arrows-expand-up-right"
                    : "gg:arrows-expand-down-left"}
                on:click={() => toggleMiniPlayer()}
            />
            <img alt="cd gif" class="cd-gif" src="images/cd6.gif" />

            <div class="info">
                {#if $currentSong}
                    {#if title}
                        <p class="title">{title}</p>
                    {:else}
                        <p class="title">{$currentSong?.file}</p>
                    {/if}
                    {#if artist}
                        <p class="artist">{artist}</p>
                    {/if}
                    {#if !title && !album && !artist}
                        <button
                            class="add-metadata-btn"
                            on:click={openTrackInfo}>Add metadata</button
                        >
                    {/if}
                    {#if album}
                        <small>{album}</small>
                    {/if}
                {:else}
                    <p class="is-placeholder">Take control of your library</p>
                {/if}
            </div>
        </div>

        {#if codec}
            <div class="file">
                <p>{codec}</p>
                {#if bitrate}<p>{bitrate} bit</p>{/if}
                <p>{sampleRate} smpls</p>
            </div>
        {/if}
    </div>

    <div class="spectrum">
        <SpectrumAnalyzer />
    </div>
    {#if $currentSong}
        <div class="artwork-container">
            <div class="artwork-frame">
                {#if artworkSrc && artworkFormat}
                    <img
                        alt="Artwork"
                        type={artworkFormat}
                        class="artwork"
                        src={artworkSrc}
                    />
                {:else}
                    <div class="artwork-placeholder">
                        <img alt="placeholder" src="icon.png" />
                        <!-- <iconify-icon icon="mdi:music-clef-treble" /> -->
                        <!-- <small>No art</small> -->
                    </div>
                {/if}
            </div>
        </div>
    {/if}
    <div class="bottom" data-tauri-drag-region>
        <div class="seekbar">
            <Seekbar {duration} />
        </div>
        <transport>
            <iconify-icon
                icon="fe:backward"
                class:disabled={$currentSongIdx === 0}
                on:click={playPrev}
            />
            <iconify-icon
                on:click={togglePlayPause}
                icon={$isPlaying ? "fe:pause" : "fe:play"}
            />
            <iconify-icon
                icon="fe:forward"
                class:disabled={$currentSongIdx === $queriedSongs.length - 1}
                on:click={playNext}
            />
        </transport>

        <div class="volume">
            <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                bind:value={$volume}
                class="slider"
                id="myRange"
            />
        </div>
    </div>
</sidebar>

<style lang="scss">
    $thumb_size: 22px;
    $mini_y_breakpoint: 400px;
    $xsmall_y_breakpoint: 320px;
    $sidebar_primary_color: transparent;
    $sidebar_secondary_color: #242026;
    sidebar {
        position: relative;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        align-items: flex-end;
        height: 100vh;
        max-width: 210px;
        min-width: 210px;
        border-right: 1px solid #ececec1c;
        background-color: $sidebar_primary_color;
        overflow: hidden;
    }
    @media only screen and (max-width: 210px) {
        sidebar {
            /* max-width: 100% !important; */
        }
    }
    sidebar.hovered {
        @media only screen and (max-height: 210px) and (max-width: 210px) {
            .track-info,
            .bottom,
            .mini-toggle {
                opacity: 1 !important;
            }

            .artwork-container {
                opacity: 0.1 !important;
            }
        }
    }

    .has-current-song {
        @media only screen and (max-height: 210px) and (max-width: 210px) {
            .track-info,
            .bottom,
            .mini-toggle {
                opacity: 0;
            }

            .artwork-container {
                opacity: 1 !important;
            }
        }

        @media only screen and (max-height: 210px) and (min-width: 211px) {
            .track-info,
            .mini-toggle {
                top: 20px !important;
            }
            .bottom {
                height: 120px;
            }
            .artwork-container {
                opacity: 0.2 !important;
            }
        }

        @media only screen and (max-height: 210px) {
            .cd-gif,
            .search-container {
                display: none;
            }
            .bottom {
                top: 0px;
            }

            .track-info,
            .info {
                background-color: transparent;
            }
            .artwork-container {
                display: flex !important;
                top: 0;
                bottom: 0;
                position: absolute;
                height: 100%;
            }

            .track-info,
            .bottom,
            .mini-toggle {
                top: 10px;
                transition: opacity 0.2s ease-in;
            }

            .add-metadata-btn {
                display: none;
            }
        }
        @media only screen and (max-height: 260px) {
            .track-info {
                justify-content: flex-start;
                small {
                    display: none;
                }
                .cd-gif {
                    /* display: none; */
                }
            }

            .add-metadata-btn {
                display: none;
            }
        }

        @media only screen and (max-height: 260px) and (max-width: 211px) {
            .mini-toggle {
                position: fixed;
            }
        }

        @media only screen and (max-height: 280px) {
            .track-info {
                .artist {
                    /* display: none; */
                }
            }
        }
        @media only screen and (max-height: 305px) {
            .file {
                display: none;
            }
            .track-info {
                hr {
                    opacity: 0;
                }
            }
        }
        @media only screen and (max-height: 380px) {
            .top {
                /* position: relative; */
            }
            .file {
                /* top: 100px; */
            }
        }
        @media only screen and (max-height: 391px) {
            menu {
                display: none;
            }

            .top,
            .track-info,
            .bottom {
                position: relative;
                top: 0;
            }
            .artwork-container {
                position: absolute;
                border: none;
                opacity: 0;
            }
        }
        @media only screen and (max-height: 420px) and (min-height: 360px) {
            .app-title {
                opacity: 0;
            }
            menu {
                display: none;
            }
            .track-info {
                /* top: 40px !important; */
            }
        }
    }

    .empty {
        @media only screen and (max-height: $mini_y_breakpoint) {
            menu,
            .app-title {
                display: none;
            }
            .track-info {
                top: 0px !important;
                .track-info-content {
                    height: 100%;
                    justify-content: flex-start;
                }
            }
        }
        @media only screen and (max-height: $xsmall_y_breakpoint) {
            .top {
                display: none !important;
            }

            .track-info {
                .is-placeholder {
                    display: none;
                }
            }
        }
    }

    hr {
        width: 100%;
        border-top: 1px solid rgba(141, 139, 139, 0.139);
        border-bottom: none;
        border-left: none;
        border-right: none;
    }
    menu {
        width: 100%;
        margin: 0;
        user-select: none;
        padding: 1em;
        margin-block-start: 0;

        background-color: $sidebar_secondary_color;
        items {
            display: flex;
            flex-direction: column;
            border-radius: 3px;
            gap: 3px;
        }

        item {
            text-transform: uppercase;
            font-weight: bold;
            text-align: left;
            width: fit-content;
            padding: 0.3em 0.5em;
            font-size: 12px;
            letter-spacing: 0.2px;
            color: rgb(143, 144, 147);
            width: 100%;
            border-radius: 3px;
            box-sizing: border-box;
            border: 1px solid transparent;

            cursor: default;
            &.selected {
                color: white;
                iconify-icon {
                    color: #45fffcf3;
                }
            }

            &:not(.selected) {
                &:active {
                    color: rgb(130, 130, 130);
                }
            }

            iconify-icon {
                margin-right: 5px;
                font-size: 15px;
                text-align: center;
                vertical-align: middle;
            }
        }
    }

    .top {
        width: 100%;
        height: 100%;
        position: sticky;
        background-color: $sidebar_secondary_color;
        top: 0;
        z-index: 1;
        transition: height 1s ease-in-out;

        .top-header {
            height: 80px;
            position: sticky;
            top: 0;
        }

        .app-title {
            font-family: "2Peas";
            width: fit-content;
            font-size: 2em;
            opacity: 0.2;
            user-select: none;
            margin: 1em auto 0;
            transition: height 1s ease-in-out;
            cursor: default;
            &:hover {
                opacity: 0.5;
            }
        }
    }

    .bottom {
        width: 100%;
        position: sticky;
        top: 400px;
        z-index: 3;
    }

    .search-container {
        padding: 1em;
        width: 100%;
        position: relative;
        background-color: $sidebar_secondary_color;

        .search {
            margin: 0;
            width: 100%;
            height: 30px;
            border-radius: 3px;
            padding-left: 5px;
            font-size: 13px;
            color: rgb(197, 193, 193);
            backdrop-filter: blur(8px);
            z-index: 10;
            &::placeholder {
                color: rgb(103, 100, 100);
            }
            &:focus {
                /* outline: 1px solid #5123dd; */
                background-color: #504c4c;
                &::placeholder {
                    color: rgb(151, 147, 147);
                }
            }
            background-color: transparent;

            border: 1px solid rgb(63, 63, 63);
        }
        .search-icon {
            position: absolute;
            right: 15px;
            top: 5px;
            bottom: 0;
            height: fit-content;
            padding: 5px;
            margin: auto 0;
            > iconify-icon {
                font-size: 17px;
                color: rgb(115, 115, 115);
                pointer-events: none;
            }
        }
    }

    .track-info {
        /* height: 150px; */
        min-height: 150px;
        width: 100%;
        background-color: $sidebar_secondary_color;
        position: sticky;
        top: 110px;
        cursor: default;
        user-select: none;
        pointer-events: none;
        border-top: 0.7px solid #ffffff23;
        z-index: 2;
    }

    .track-info-content {
        top: 0;
        max-height: 170px;
        width: 100%;
        height: fit-content;
        position: sticky;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
    }

    .mini-toggle {
        pointer-events: all;
        font-size: 20px;

        position: absolute;
        top: 15px;
        right: 8px;
        color: rgb(115, 115, 115);
        padding: 3px;

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }

        &.hovered {
            background-color: rgba(0, 0, 0, 0.457);
            border-radius: 4px;
        }
    }
    .info {
        background-color: $sidebar_primary_color;
        color: white;
        width: 100%;
        padding: 0.8em 0.3em;

        .artist {
            white-space: nowrap;
            font-weight: 500;
            font-size: 0.9em;
            opacity: 0.9;
            z-index: 1;
        }
        .title {
            white-space: nowrap;
            font-weight: bold;
        }
        small {
            white-space: nowrap;
            opacity: 0.7;
        }
        * {
            margin: 0;
        }

        .is-placeholder {
            font-family: "Snake", Courier, monospace;
            font-size: 2em;
            line-height: 1.2em;
            margin: 0 1em;
        }
        .add-metadata-btn {
            border-radius: 4px;
            font-size: 13px;
            color: rgb(166, 140, 207);
            margin-top: 5px;
            height: 25px;
            padding: 2px 10px;
            pointer-events: all;
        }
    }

    .file {
        position: absolute;
        bottom: 0px;
        background-color: $sidebar_secondary_color;

        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
        padding: 8px 0;
        user-select: none;
        pointer-events: none;
        width: 100%;
        color: rgb(125, 125, 125);

        p {
            background-color: rgba(85, 85, 85, 0.162);
            padding: 0em 0.6em;
            margin: 0;
            border-radius: 2px;
            font-size: 0.7em;
            line-height: 1.5em;
            font-weight: 600;
            border-top: 1px solid rgb(49, 49, 49);
            /* border-bottom: 1px dashed rgb(49, 49, 49); */
            font-family: monospace;
            text-transform: uppercase;
        }
    }
    .spectrum {
        z-index: 1;
        position: absolute;
        bottom: -5px;
        pointer-events: none;
        opacity: 0.2;
    }

    .cd-gif {
        margin-top: 1em;
        width: 30px;
        height: auto;
        margin-left: 10px;
        align-self: center;
        z-index: 0;
    }

    .artwork-container {
        padding: 0em;
        width: 100%;
        height: 210px;
        margin: auto;
        pointer-events: none;
        opacity: 1;
        box-sizing: content-box;
        border-top: 0.7px solid #ffffff23;
        border-bottom: 0.7px solid #ffffff23;
        z-index: 0;

        .artwork-frame {
            width: 100%;
            height: 100%;
            box-sizing: border-box;
            /* border-radius: 3px; */
            /* border: 1px solid rgb(94, 94, 94); */
            display: flex;
            align-items: center;
            justify-content: center;
            > img {
                object-fit: cover;
            }
            .artwork-placeholder {
                opacity: 0.8;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 1em;
                z-index: -1;
                img {
                    width: 80%;
                }
                iconify-icon {
                    /* margin-top: 0.7em; */
                }
            }
        }
    }

    @keyframes marquee {
        0% {
            transform: translate(0%, 0);
        }
        100% {
            transform: translate(-100%, 0);
        }
    }

    img {
        width: 100%;
        height: 100%;
    }

    .seekbar {
        width: 100%;
    }
    transport {
        /* background-color: rgb(255, 255, 255); */
        padding: 0em 1em 0 1em;
        width: 100%;
        color: white;
        z-index: 2;
    }

    .volume {
        padding: 0 2em 1em;
        width: 100%;

        input {
            -webkit-appearance: none;
            width: 100%;
            height: 5px;
            background: #474747;
            outline: none;
            opacity: 1;
            border-radius: 3px;
            -webkit-transition: 0.2s;
            transition: opacity 0.2s;

            &::-webkit-slider-thumb {
                -webkit-appearance: none;
                appearance: none;
                width: $thumb_size;
                height: $thumb_size;
                background: url("/images/volume-up.svg");
            }

            &::-moz-range-thumb {
                width: $thumb_size;
                height: $thumb_size;
                background: #04aa6d;
            }
        }
    }

    iconify-icon {
        font-size: 40px;
        &.disabled {
            pointer-events: none;
            color: #474747;
        }
        &:hover {
            opacity: 0.5;
        }

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }
    }
</style>
