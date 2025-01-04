<script lang="ts">
    import type { Event } from "@tauri-apps/api/event";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { onDestroy, onMount } from "svelte";
    import md5 from "md5";
    import { db } from "../../data/db";
    import type { Song } from "../../App";
    import {
        fileToDownload,
        scrollToSong,
        uiView,
        userSettings,
    } from "../../data/store";
    import { fly } from "svelte/transition";
    import Icon from "../ui/Icon.svelte";
    import ProgressBar from "../ui/ProgressBar.svelte";
    const appWindow = getCurrentWebviewWindow();

    let unlistenFn;

    let downloadProgress = 0;

    let songAdded: Song | null = null;

    async function checkSongAddedToLibrary() {
        let downloadLocation = $fileToDownload.downloadLocation;
        setTimeout(async () => {
            const filehash = md5(downloadLocation);
            const song = await db.songs.get(filehash);
            if (song) {
                songAdded = song;
            }
        }, 2000);
    }

    async function goToSong() {
        $scrollToSong = songAdded;
        $uiView = "library";
        $fileToDownload = null;
    }

    onMount(async () => {
        unlistenFn = await appWindow.listen(
            "download-progress",
            async (event: Event<number>) => {
                const progress = event.payload;
                console.log("progress", event.payload);
                if (progress === 100) {
                    checkSongAddedToLibrary();
                }
                downloadProgress = event.payload;
            },
        );
    });

    onDestroy(() => {
        unlistenFn && unlistenFn();
        $fileToDownload = null;
    });
</script>

<div class="container" transition:fly={{ duration: 200 }}>
    <div class="download-info">
        {#if downloadProgress < 100}
            <p>Downloading...</p>
        {:else}
            <p>Downloaded</p>
        {/if}

        <ProgressBar percent={downloadProgress} />

        {#if downloadProgress === 100}
            <p>Saved to {$fileToDownload?.downloadLocation}</p>
        {/if}

        {#if songAdded}
            <div class="song-info" transition:fly={{ duration: 200 }}>
                <p>
                    <b>Track added to library.</b><span on:click={goToSong}
                        >Go to track →</span
                    >
                </p>
            </div>
        {/if}
    </div>
    <div class="download-options">
        <Icon
            icon="lucide:chevron-down"
            onClick={() => {
                $fileToDownload = null;
            }}
        />
    </div>
</div>

<style lang="scss">
    .container {
        position: fixed;
        max-width: 350px;
        min-width: 300px;
        bottom: 4em;
        right: 2em;
        display: grid;
        grid-template-columns: 1fr auto;
        border-radius: 5px;
        background-color: var(--overlay-bg);
        backdrop-filter: blur(8px);
        margin: 0.25em;
        align-items: center;
        justify-content: space-between;
        border: 1px solid color-mix(in srgb, var(--inverse) 20%, transparent);
        cursor: default;
        z-index: 10;
        p {
            margin: 0;
        }
        .download-info {
            padding: 0.5em 1em;
            overflow: hidden;
            display: flex;
            flex-direction: column;
            p {
                opacity: 0.7;
                font-size: 13px;
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
                text-align: start;
                user-select: none;
                color: var(--text);
            }
        }

        .download-options {
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 0.5em 1em;
        }
    }
</style>
