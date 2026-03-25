<script lang="ts">
    import { debounce } from "lodash-es";
    import tippy from "svelte-tippy";
    import { cubicInOut } from "svelte/easing";
    import { fly } from "svelte/transition";
    import { runScan } from "../../data/LibraryUtils";

    import { liveQuery } from "dexie";
    import { db } from "../../data/db";
    import {
        bottomBarNotification,
        importStatus,
        isFolderWatchUpdate,
        isLyricsOpen,
        isQueueOpen,
        isSidebarOpen,
        isWaveformOpen,
        nextUpSong,
        userSettings,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import Equalizer from "../player/Equalizer.svelte";
    import { isIAPlaying } from "../player/WebAudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import ToggleButton from "../ui/ToggleButton.svelte";

    let right;
    let nextUp;

    $: scanningStatusText = $importStatus.isImporting
        ? `Scanning: ${$importStatus.currentFolder}`
        : "Click to re-scan folders.";

    let showVisualiser = true;
    let visualiserWidth = 0;

    $: counts = liveQuery(async () => {
        if ($importStatus.isImporting) {
            return undefined;
        }
        const artists = await (
            await db.songs.orderBy("artist").uniqueKeys()
        ).length;
        const albums = await (
            await db.songs.orderBy("album").uniqueKeys()
        ).length;
        const songs = await db.songs.count();
        return { songs, artists, albums };
    });

    nextUpSong.subscribe((nextUp) => {
        setTimeout(() => {
            onResize();
        }, 200);
    });

    function onResize() {
        if (nextUp?.getBoundingClientRect()) {
            // calculate remaining space for spectroscope visualizer
            const nextUpRight = nextUp?.getBoundingClientRect()?.right;
            const rightXPos = right?.getBoundingClientRect()?.left;
            const diff = Math.abs(nextUpRight - rightXPos);
            // console.log("diff", diff);
            visualiserWidth = Math.min(150, diff - 13);
            showVisualiser = window.innerWidth > 900 && diff > 150;
        }
    }
</script>

<svelte:window on:resize={debounce(onResize, 5)} />

<bottom-bar data-tauri-drag-region class:sidebar-collapsed={!$isSidebarOpen}>
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="left" data-tauri-drag-region>
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <ToggleButton
            isSelected={$isQueueOpen}
            onClick={() => {
                $isQueueOpen = !$isQueueOpen;
            }}
            icon="mdi:playlist-music"
            iconSize={14}
            text={$LL.bottomBar.queue()}
        />
        {#if !$isIAPlaying}
            <ToggleButton
                isSelected={$isLyricsOpen}
                onClick={() => {
                    $isLyricsOpen = !$isLyricsOpen;
                }}
                icon="material-symbols:lyrics"
                iconSize={14}
                text={$LL.bottomBar.lyrics()}
            />
        {/if}
        <ToggleButton
            isSelected={$isWaveformOpen}
            onClick={() => ($isWaveformOpen = !$isWaveformOpen)}
            icon="ph:wave-sine-duotone"
            iconSize={14}
            text={$LL.bottomBar.waveform()}
        />
        <Equalizer />
        {#if !$isIAPlaying && $nextUpSong}
            <div class="next-up" bind:this={nextUp}>
                <p class="label">{$LL.bottomBar.nextUp()}:</p>
                <p class="song">
                    {$nextUpSong.title ?? $nextUpSong.file}
                </p>
            </div>
        {/if}
    </div>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="right" bind:this={right}>
        {#if $userSettings.foldersToWatch.length}
            <div
                class="refresh-icon"
                class:scanning={$importStatus.isImporting ||
                    $isFolderWatchUpdate}
                use:tippy={{
                    content: scanningStatusText,
                    placement: "top",
                }}
            >
                <Icon
                    icon="tabler:refresh"
                    size={15}
                    onClick={() => {
                        runScan();
                    }}
                />
            </div>
        {/if}
        {#if $counts !== undefined}
            <div
                class="stats"
                in:fly={{
                    y: 30,
                    duration: 150,
                    easing: cubicInOut,
                }}
            >
                <p class="songs">
                    {$counts.songs}
                    {$LL.bottomBar.stats.songs()}
                </p>
                <!-- <p class="artists">
                    {$counts.artists}
                    {$LL.bottomBar.stats.artists()}
                </p>
                <p class="albums">
                    {$counts.albums}
                    {$LL.bottomBar.stats.albums()}
                </p> -->
            </div>
        {:else}
            <p>...</p>
        {/if}

        {#if $bottomBarNotification}
            <div
                class="notification"
                transition:fly={{
                    y: 30,
                    duration: 150,
                    easing: cubicInOut,
                }}
            >
                <p>
                    <!-- Folder updated blab alb alfkbafadfoijadsofi -->
                    {$bottomBarNotification.text}
                </p>
            </div>
        {/if}
    </div>
</bottom-bar>

<style lang="scss">
    bottom-bar {
        color: var(--text);
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 1em;
        justify-content: flex-end;
        padding: 0 1em 0 0;
        width: 100%;
        height: 30px;

        &.sidebar-collapsed {
            padding-left: 10px;
        }

        p {
            color: var(--text);
            margin: 0;
        }
        @media only screen and (max-width: 600px) {
            p:not(:nth-child(1)) {
                display: none;
            }
        }

        .left {
            display: flex;
            flex-direction: row;
            flex-grow: 1;
            align-items: center;
            position: relative;

            @media only screen and (max-width: 500px) {
                .lossy-selector {
                    display: none;
                }
            }

            .next-up {
                display: flex;
                flex-direction: row;
                gap: 5px;
                max-width: 250px;
                padding: 0 1.5em;
                @media only screen and (max-width: 1050px) {
                    display: none;
                }
                .label {
                    opacity: 0.5;
                }

                .song {
                    overflow: hidden;
                    text-align: start;
                    text-overflow: ellipsis;
                }
                p {
                    margin: 0;
                    white-space: nowrap;
                    font-size: 0.9em;
                    user-select: none;
                    cursor: default;
                    color: var(--text-secondary);
                }
            }
        }

        .right {
            display: flex;
            flex-direction: row;
            flex-grow: 1;
            align-items: center;
            position: relative;
            gap: 1em;
            justify-content: flex-end;
        }

        .refresh-icon {
            display: flex;
            transform: rotate(0deg);
            &.scanning {
                animation: rotate 1.5s linear infinite normal forwards;
            }

            @media only screen and (max-width: 800px) {
                display: none;
            }
        }

        .notification {
            position: absolute;
            right: -6px;
            top: 0;
            bottom: 0;
            padding: 0 1em 0 5em;
            margin: 0;
            height: 100%;
            border-radius: 7px;
            background: linear-gradient(
                to right,
                transparent 0%,
                #242026e5 14%,
                #242026fb 100%
            );
            display: flex;
            align-items: center;
            width: max-content;
            p {
                color: var(--text);
                font-size: 13px;
                margin: 0;
            }
        }

        .stats {
            display: inline-flex;
            gap: 10px;
            white-space: nowrap;
            p {
                user-select: none;
                cursor: default;
                color: var(--text-secondary);
            }

            @media only screen and (max-width: 800px) {
                .albums {
                    display: none;
                }
            }
            @media only screen and (max-width: 700px) {
                .artists {
                    display: none;
                }
            }
        }
    }

    @keyframes rotate {
        0% {
            transform: rotate(0deg);
            opacity: 1;
        }
        50% {
            transform: rotate(180deg);
            opacity: 0.7;
            color: cyan;
        }
        100% {
            transform: rotate(360deg);
            opacity: 1;
        }
    }
</style>
