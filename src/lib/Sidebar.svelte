<script lang="ts">
    import { window as tauriWindow } from "@tauri-apps/api";
    import { register, unregisterAll } from "@tauri-apps/api/globalShortcut";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { LogicalSize } from "@tauri-apps/api/window";
    import hotkeys from "hotkeys-js";
    import * as musicMetadata from "music-metadata-browser";
    import { onDestroy, onMount } from "svelte";
    import tippy from "svelte-tippy";
    import { lookForArt } from "../data/LibraryImporter";
    import {
        currentSong,
        currentSongArtworkSrc,
        currentSongIdx,
        isInfoPopupOpen,
        isMiniPlayer,
        isPlaying,
        os,
        queriedSongs,
        query,
        singleKeyShortcutsEnabled,
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
        if (!$isMiniPlayer && height <= 210 && width <= 210) {
            $isMiniPlayer = true;
            console.log("setting to false");
            await tauriWindow.getCurrent().setDecorations(false);
        } else if ($isMiniPlayer && (height > 210 || width > 210)) {
            $isMiniPlayer = false;
            console.log("setting to true");
            await tauriWindow.getCurrent().setDecorations(true);
        }
    }

    let searchInput: HTMLInputElement;

    onMount(() => {
        height = window.innerHeight;
        window.onresize = onResize;
        onResize(); // run once
        searchInput.onfocus = (evt) => {
            $singleKeyShortcutsEnabled = false;
        };
        searchInput.onblur = (evt) => {
            $singleKeyShortcutsEnabled = true;
        };
    });

    let widthToRestore = 0;
    let heightToRestore = 0;

    function toggleMiniPlayer() {
        if (!$isMiniPlayer) {
            widthToRestore = window.innerWidth;
            heightToRestore = window.innerHeight;
            tauriWindow.getCurrent().setSize(new LogicalSize(210, 210));
        } else {
            if (widthToRestore && heightToRestore) {
                tauriWindow
                    .getCurrent()
                    .setSize(new LogicalSize(widthToRestore, heightToRestore));
            } else {
                tauriWindow
                    .getCurrent()
                    .setSize(new LogicalSize(1100, 700));
            }
        }
    }
</script>

<sidebar
    class:has-current-song={$currentSong}
    class:empty={!$currentSong}
    data-tauri-drag-region
>
    <h1 class="app-title" on:click={openInfoWindow}>Musicat</h1>
    <!-- <div class="knob">
    <Knob bind:value={volumeKnob} max={100} min={0} pixelRange={200} />
  </div> -->
    <div class="top">
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
        </div>

        <menu>
            <items>
                <item>
                    <iconify-icon icon="fluent:library-20-filled" />Music</item
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
        <hr />

        <div class="track-info-content">
            <iconify-icon
                use:tippy={{
                    theme: $isMiniPlayer ? "hidden" : "",
                    content: "Toggle the mini player.",
                    placement: "right"
                }}
                class="mini-toggle"
                icon={$isMiniPlayer
                    ? "gg:arrows-expand-up-right"
                    : "gg:arrows-expand-down-left"}
                on:click={() => toggleMiniPlayer()}
            />
            <img alt="cd gif" class="cd-gif" src="images/cd6.gif" />

            <div class="info">
                {#if title}
                    <p class="title">{title}</p>
                {/if}
                {#if artist}
                    <p class="artist">{artist}</p>
                {/if}
                {#if album}
                    <small>{album}</small>
                {/if}
                {#if !artist && !title && !album}
                    <p class="is-placeholder">Take control of your library</p>
                {/if}
            </div>
        </div>
    </div>
    {#if codec}
        <div class="file">
            <p>{codec}</p>
            {#if bitrate}<p>{bitrate} bit</p>{/if}
            <p>{sampleRate} smpls</p>
        </div>
    {/if}

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
                        <iconify-icon
                            icon="mdi:music-clef-treble"
                            on:click={playPrev}
                        />
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
            <iconify-icon icon="fe:backward" on:click={playPrev} />
            <iconify-icon
                on:click={togglePlayPause}
                icon={$isPlaying ? "fe:pause" : "fe:play"}
            />
            <iconify-icon icon="fe:forward" on:click={playNext} />
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
    $sidebar_primary_color: #242026;
    $sidebar_secondary_color: rgb(234, 226, 226);
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
    sidebar:hover {
        @media only screen and (max-height: 210px) {
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
                opacity: 1;
                height: 100%;
            }

            .track-info,
            .bottom,
            .mini-toggle {
                opacity: 0;
                top: 10px;
                transition: opacity 0.2s ease-in;
            }
        }
        @media only screen and (max-height: 260px) {
            .track-info {
                small {
                    display: none;
                }
                .cd-gif {
                    /* display: none; */
                }
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
                margin-top: 3em;
                hr {
                    display: none;
                }
            }
        }
        @media only screen and (max-height: 380px) {
            .top {
                position: relative;
            }
            .file {
                top: 100px;
            }
        }
        @media only screen and (max-height: 365px) {
            menu,
            .app-title,
            .artwork-container {
                display: none;
            }
            .track-info {
                top: 0px !important;
            }
        }
        @media only screen and (max-height: 420px) and (min-height: 360px) {
            menu,
            .artwork-container,
            .app-title {
                display: none;
            }
            .track-info {
                top: 40px !important;
            }
            .file {
                top: 100px !important;
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
                top: 50px !important;
            }
        }
        @media only screen and (max-height: $xsmall_y_breakpoint) {
            .track-info,
            .top {
                display: none !important;
            }
        }
    }

    .track-info-content {
        width: 100%;
        position: relative;
    }

    .mini-toggle {
        pointer-events: all;
        font-size: 20px;

        position: absolute;
        top: 5px;
        right: 10px;
        color: rgb(115, 115, 115);

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }
    }
    .app-title {
        font-family: "2Peas";
        width: fit-content;
        font-size: 2em;
        opacity: 0.2;
        user-select: none;
        margin: 1em auto 0;

        cursor: default;
        &:hover {
            opacity: 0.5;
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

        background-color: $sidebar_primary_color;
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
            font-size: 13px;
            letter-spacing: 1px;
            color: rgb(181, 182, 186);

            cursor: default;
            &:hover {
                color: rgb(255, 255, 255);
            }
            &:active {
                color: rgb(130, 130, 130);
            }

            iconify-icon {
                margin-right: 5px;
                font-size: 15px;
                text-align: center;
                vertical-align: middle;
                opacity: 0.4;
            }
        }
    }

    .top {
        width: 100%;
        position: sticky;
        top: 0;
    }

    .bottom {
        width: 100%;
        position: sticky;
        top: 400px;
    }

    .search-container {
        padding: 1em;
        width: 100%;
        background-color: $sidebar_primary_color;

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
                color: rgb(128, 126, 126);
            }
            &:focus {
                /* outline: 1px solid #5123dd; */
                background-color: #504c4c;
            }
            background-color: #6061703a;

            border: 1px solid rgb(63, 63, 63);
        }
    }

    .track-info {
        height: 100%;
        width: 100%;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        background-color: $sidebar_primary_color;
        position: sticky;
        top: 110px;
        cursor: default;
        user-select: none;
        pointer-events: none;
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
        height: 200px;
        pointer-events: none;

        .artwork-frame {
            width: 100%;
            height: 100%;
            box-sizing: border-box;
            /* border-radius: 3px; */
            /* border: 1px solid rgb(94, 94, 94); */
            display: flex;
            align-items: center;
            justify-content: center;

            .artwork-placeholder {
                opacity: 0.2;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 1em;
                iconify-icon {
                    /* margin-top: 0.7em; */
                }
            }
        }
    }

    .file {
        position: sticky;
        top: 265px;
        background-color: $sidebar_primary_color;

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
    .info {
        background-color: $sidebar_primary_color;
        color: white;
        width: 100%;
        padding: 0.8em 0.3em;

        .artist {
            white-space: nowrap;
            font-weight: bold;
            font-size: 0.9em;
            z-index: 1;
        }
        .title {
            white-space: nowrap;
            font-weight: bold;
        }
        small {
            white-space: nowrap;
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
        border-radius: 2px;
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
                background: url("images/volume-up.svg");
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

        &:hover {
            opacity: 0.5;
        }

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }
    }
</style>
