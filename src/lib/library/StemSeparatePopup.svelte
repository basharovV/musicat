<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { onDestroy, onMount } from "svelte";
    import { fly } from "svelte/transition";
    import type { Song, Stem } from "../../App";
    import {
        expandedSongWithStems,
        songToSeparate,
        uiView,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import Icon from "../ui/Icon.svelte";
    import ProgressBar from "../ui/ProgressBar.svelte";
    import { db } from "../../data/db";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    import { size } from "@tauri-apps/plugin-fs";
    const appWindow = getCurrentWebviewWindow();

    let unlistenFn;

    let status: StemSeparationEvent;
    let stems: Stem[] | null = null;
    let songWithStems: Song;

    // let status: StemSeparationEvent = {
    //     event: "progress",
    //     message: "",
    //     progress: 40,
    // };
    // let stems: Stem[] = [
    //     {
    //         name: "stem1",
    //         path: "path1",
    //     },
    // ];

    async function checkStems() {
        stems = await invoke("get_stems", {
            event: {
                song_id: $songToSeparate?.id,
            },
        });

        console.log("Found stems", stems);
        songWithStems = { ...$songToSeparate, stems };
        // Update Song with stems
        await db.songs.put(songWithStems);
    }

    async function cancelSeparation() {
        await invoke("cancel_separation", {
            event: {
                song_id: $songToSeparate?.id,
            },
        });
        $songToSeparate = null;
    }

    async function goToSong() {
        // $scrollToSong = songAdded;
        $expandedSongWithStems = songWithStems;
        $uiView = "library";
        $songToSeparate = null;
    }

    interface StemSeparationEvent {
        event: string;
        message: string;
        progress?: number;
    }

    onMount(async () => {
        unlistenFn = await appWindow.listen<StemSeparationEvent>(
            "stem-separation",
            async (event) => {
                status = event.payload;
                console.log("stem-separation", event.payload);
                if (event.payload.event === "complete") {
                    checkStems();
                }
            },
        );
    });

    onDestroy(() => {
        unlistenFn && unlistenFn();
        $songToSeparate = null;
    });
</script>

<div
    class="container"
    class:active={status?.event === "progress"}
    transition:fly={{ duration: 200 }}
>
    <div class="separate-info">
        {#if status}
            {#if status.event === "progress"}
                <p>
                    {$LL.stemSeparation.loading()}
                    <LoadingSpinner />
                </p>
            {:else if status.event === "complete"}
                <p>{$LL.stemSeparation.complete()}</p>
            {:else if status.event === "error"}
                <p>{$LL.stemSeparation.error(status.message)}</p>
            {/if}
        {:else}
            <p>{$LL.stemSeparation.loading()}</p>
        {/if}

        <div class="progress">
            <ProgressBar percent={status?.progress ?? 0} />
            <p>{status?.progress?.toFixed(0) ?? 0}%</p>
        </div>

        {#if status?.event === "complete" && stems?.length > 0}
            <div class="song-info" transition:fly={{ duration: 200 }}>
                <p>
                    Saved to {stems.map((stem) => stem.name).join(", ")}.
                </p>
            </div>
        {/if}
    </div>
    <div class="separate-options">
        {#if status?.event === "complete" && stems?.length > 0}
            <ButtonWithIcon
                size="small"
                icon="material-symbols:folder"
                onClick={goToSong}
                text={$LL.stemSeparation.showStems()}
                theme="active"
            />
            <ButtonWithIcon
                size="small"
                icon="material-symbols:close"
                onClick={() => {
                    $songToSeparate = null;
                }}
                text={$LL.stemSeparation.close()}
                theme="transparent"
            />
        {/if}
        {#if status?.event === "progress"}
            <ButtonWithIcon
                size="small"
                icon="material-symbols:close"
                onClick={() => cancelSeparation()}
                text={$LL.stemSeparation.cancel()}
                theme="transparent"
            />
        {/if}
    </div>
</div>

<style lang="scss">
    .container {
        position: fixed;
        width: 320px;
        bottom: 4em;
        right: 2em;
        display: grid;
        grid-template-columns: 1fr;
        border-radius: 5px;
        background-color: var(--overlay-bg);
        backdrop-filter: blur(8px);
        margin: 0.25em;
        align-items: center;
        justify-content: space-between;
        border: 1px solid color-mix(in srgb, var(--inverse) 20%, transparent);
        cursor: default;
        box-shadow: 0px 0px 8px 2px var(--menu-shadow);

        @media only screen and (max-width: 500px) {
            visibility: hidden;
        }
        @keyframes pulse {
            0% {
                background-color: var(--overlay-bg);
            }
            50% {
                background-color: color-mix(
                    in srgb,
                    var(--overlay-bg) 100%,
                    var(--accent-secondary) 20%
                );
            }
            100% {
                background-color: var(--overlay-bg);
            }
        }

        &.active {
            animation: pulse 2s infinite;
        }
        z-index: 10;
        p {
            margin: 0;
        }

        .progress {
            display: grid;
            grid-template-columns: 1fr auto;
            gap: 10px;
            align-items: center;
        }
        .separate-info {
            padding: 1em 1em 0.75em;
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
                display: inline-flex;
                align-items: center;
                justify-content: space-between;
                gap: 5px;
            }
        }

        .separate-options {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 0.6em 0.6em;
        }
    }
</style>
