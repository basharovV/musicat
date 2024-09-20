<script lang="ts">
    import { liveQuery } from "dexie";
    import type { ArtistProject, Song, SongProject } from "src/App";
    import { db } from "../../data/db";
    import {
        isDraggingExternalFiles,
        isFullScreenLyrics,
        isScrapbookShown,
        songbookSelectedArtist,
        userSettings
    } from "../../data/store";

    import { blur } from "svelte/transition";
    import ArtistInfo from "../your-music/ArtistInfo.svelte";
    import ContentDropzone from "../your-music/ContentDropzone.svelte";
    import Music from "../your-music/Music.svelte";
    import Scrapbook from "../your-music/Scrapbook.svelte";
    import YourArtists from "../your-music/YourArtists.svelte";
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import { startWatchingScrapbookFolder } from "../../data/FolderWatcher";
    import { onDestroy } from "svelte";
    let selectedSong: Song;
    let selectedSongProject: SongProject;
    let songProjectSelection;

    let scrapbookSize = 350;
    const SCRAPBOOK_MIN_SIZE = 300;

    let selectedTab;

    let container: HTMLElement;
    let isResizing = false;
    let showCloseScrapbookPrompt = false;
    function onResize(e: MouseEvent) {
        e.stopPropagation();
        e.preventDefault();
        const containerWidth = window.innerWidth;
        if (containerWidth - e.clientX > SCRAPBOOK_MIN_SIZE) {
            showCloseScrapbookPrompt = false;
            scrapbookSize = containerWidth - e.clientX;
        } else if (containerWidth - e.clientX < SCRAPBOOK_MIN_SIZE - 10) {
            showCloseScrapbookPrompt = true;
        }
    }
    function startResizeListener() {
        isResizing = true;
        container.addEventListener("mousemove", onResize);
        document.addEventListener("mouseup", stopResizeListener);
    }
    function stopResizeListener() {
        isResizing = false;
        container?.removeEventListener("mousemove", onResize);
        if (showCloseScrapbookPrompt) {
            $isScrapbookShown = false;
            showCloseScrapbookPrompt = false;
        }
    }

    let unlistenFolderWatch: UnlistenFn;
    userSettings.subscribe(async (_) => {
        unlistenFolderWatch && unlistenFolderWatch();
        unlistenFolderWatch = await startWatchingScrapbookFolder();
    });

    onDestroy(() => {
        unlistenFolderWatch();
        container?.removeEventListener("mousemove", onResize);
        document?.removeEventListener("mouseup", stopResizeListener);
    });
</script>

<container
    bind:this={container}
    class:scrapbook-open={$isScrapbookShown}
    style={$isScrapbookShown
        ? `grid-template-columns: 1fr auto ${scrapbookSize}px;`
        : "grid-template-columns: 1fr auto;"}
>
    {#if $isDraggingExternalFiles}
        <ContentDropzone songProject={selectedSongProject} />
    {/if}
    <header>
        <YourArtists />
    </header>
    <section class="content" class:full-screen={$isFullScreenLyrics}>
        <Music />
    </section>

    <resize-handle
        on:mousedown={startResizeListener}
        class:resizing={isResizing}
    />

    {#if $isScrapbookShown}
        <section class="scrapbook">
            <div class="content">
                {#if showCloseScrapbookPrompt}
                    <div
                        class="close-scrapbook-prompt"
                        transition:blur={{ duration: 100 }}
                    >
                        <h2>Close scrapbook</h2>
                    </div>
                {/if}
                <Scrapbook />
            </div>
        </section>
    {/if}
    <img class="bulb" src="images/bulby_bulb.png" alt="" />
</container>

<style lang="scss">
    .bulb {
        position: fixed;
        bottom: 0;
        right: 4em;
        width: 150px;
        z-index: 0;
        display: flex;
        opacity: 0.3;
    }
    .arrow {
        transform: rotate(90deg);
        width: 20px;
        top: 5px;
        position: relative;
    }

    h2 {
        margin: 0;
        font-family: "Snake";
        font-size: 3em;
        color: #bbb9b9;
    }

    container {
        text-align: left;
        display: grid;
        grid-template-columns: 1fr auto auto;
        gap: 5px;
        grid-template-rows: auto 1fr;
        margin: 5px 5px 5px 0;
        border-radius: 5px;
        overflow: hidden;

        &.scrapbook-open {
            /* grid-template-columns: 1fr 350px; */ // INLINE STYLED
            grid-template-rows: auto 1fr;
            grid-template-columns: 1fr auto auto;
        }

        header {
            grid-row: 1;
            grid-column: 1;
            z-index: 2;
        }

        .scrapbook {
            background-color: transparent;
            grid-row: 1 / 3;
            grid-column: 3;
            height: 100%;
            overflow: auto;
            position: relative;
            background-color: var(--panel-background);
            border-radius: 5px;
            border: 0.7px solid
                color-mix(in srgb, var(--type-bw-inverse) 20%, transparent);

            .close-scrapbook-prompt {
                position: absolute;
                top: 0;
                bottom: 0;
                left: 0;
                right: 0;
                display: flex;
                align-items: center;
                justify-content: center;
                color: white;
                background-color: #33303ce2;
                backdrop-filter: blur(5px);
                z-index: 10;
            }
            .content {
                padding-top: 0;
            }
        }

        resize-handle {
            grid-column: 3;
            grid-row: 1 / 3;
            display: block;
            display: block;
            position: relative;
            height: 100%;
            width: 4px;
            left: -6px;
            z-index: 19;

            &::after {
                content: "";
                display: block;
                position: absolute;
                margin: auto;
                top: 0;
                bottom: 0;
                left: -2px;
                right: 0;
                width: 4px;
                box-sizing: border-box;
                background: url("/images/resize-handle.svg") no-repeat center;
                opacity: 0.3;
                /* border: 2px dotted color-mix(in srgb, var(--inverse) 90%, transparent); */
                z-index: 10;
            }
            &:hover,
            &.resizing {
                background-color: var(--accent-secondary);
            }

            cursor: ew-resize;
        }
    }

    .content {
        width: 100%;
        grid-column: 1;
        grid-row: 2;
        z-index: 1;
        overflow: hidden;
        &.full-screen {
            z-index: 22;
            overflow: visible;
        }
    }
    hr {
        border-top: 1px solid rgba(255, 255, 255, 0.093);
        border-bottom: none;
        border-left: none;
        border-right: none;
    }
</style>
