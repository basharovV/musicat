<script lang="ts">
    import "iconify-icon";
    import tippy from "svelte-tippy";
    import { cubicInOut } from "svelte/easing";
    import { fly } from "svelte/transition";
    import { runScan } from "../../data/LibraryImporter";

    import {
        bottomBarNotification,
        importStatus,
        isFolderWatchUpdate,
        nextUpSong
    } from "../../data/store";
    import CompressionSelector from "../ui/CompressionSelector.svelte";
    import Icon from "../ui/Icon.svelte";

    export let counts;

    $: scanningStatusText = $importStatus.isImporting
        ? `Scanning: ${$importStatus.currentFolder}`
        : "Click to re-scan folders.";
</script>

<bottom-bar>
    <div class="left">
        <div class="lossy-selector">
            <CompressionSelector />
        </div>
        {#if $nextUpSong}
            <div class="next-up">
                <p class="label">Next up:</p>
                <p class="song">
                    {$nextUpSong.title ?? $nextUpSong.file}
                </p>
            </div>
        {/if}
    </div>
    {#if $bottomBarNotification}
        <div class="notification">
            <p>
                {$bottomBarNotification.text}
            </p>
        </div>
    {/if}
    <p></p>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        class="refresh-icon"
        class:scanning={$importStatus.isImporting || $isFolderWatchUpdate}
        use:tippy={{
            content: scanningStatusText,
            placement: "top"
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
    {#if $counts !== undefined}
        <div
            class="stats"
            in:fly={{
                y: 30,
                duration: 150,
                easing: cubicInOut
            }}
        >
            <p>{$counts.songs} songs</p>
            <p>{$counts.artists} artists</p>
            <p>{$counts.albums} albums</p>
        </div>
    {:else}
        <p>...</p>
    {/if}
</bottom-bar>

<style lang="scss">
    bottom-bar {
        background-color: rgba(28, 26, 26, 0.645);
        backdrop-filter: blur(8px);
        border-top: 1px solid rgb(51, 51, 51);
        color: white;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 1em;
        justify-content: flex-end;
        padding: 0 1em;
        width: 100%;
        height: 30px;

        p {
            color: rgb(176, 161, 161);
            margin: 0;
        }
        @media only screen and (max-width: 522px) {
            p:not(:nth-child(1)) {
                display: none;
            }
        }

        .left {
            display: flex;
            flex-direction: row;
            flex-grow: 1;
            align-items: center;

            .next-up {
                display: flex;
                flex-direction: row;
                gap: 5px;
                max-width: 250px;
                .label {
                    opacity: 0.5;
                }
                padding: 0 1.5em;

                .song {
                    overflow: hidden;
                    text-align: start;
                    text-overflow: ellipsis;
                }
                p {
                    margin: 0;
                    white-space: nowrap;
                    font-size: 0.9em;
                }
            }

            .lossy-selector {
                display: flex;
                flex-direction: row;
                background-color: #24232332;
                /* border: 1px solid rgba(128, 128, 128, 0.29); */
                border-radius: 3px;
                > div {
                    padding: 1px 10px;
                    cursor: default;
                    &:hover {
                        background-color: #bbb9b92e;
                    }
                    &:active {
                        background-color: #bbb9b923;
                    }
                    &.selected {
                        p {
                            color: rgb(224, 218, 218);
                        }
                        background-color: #35309784;
                    }
                    p {
                        margin: 0;
                        line-height: 1.3em;
                        user-select: none;
                        text-transform: lowercase;
                    }
                }
            }
        }

        .refresh-icon {
            display: flex;
            transform: rotate(0deg);
            &.scanning {
                animation: rotate 1.5s linear infinite normal forwards;
            }
        }

        .notification {
            padding: 0 1em;
            margin: 0;
            background: linear-gradient(
                90deg,
                rgba(0, 0, 0, 0) 0%,
                rgba(28, 26, 26, 0.25) 10%,
                rgba(54, 22, 56, 0.65) 50%,
                rgba(0, 0, 0, 0) 100%
            );
            height: 100%;
            display: flex;
            align-items: center;
            p {
                color: rgb(203, 182, 208);
                font-size: 13px;
                margin: 0;
            }
        }

        .stats {
            display: inline-flex;
            gap: 10px;
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
