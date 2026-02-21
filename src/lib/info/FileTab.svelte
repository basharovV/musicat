<script lang="ts">
    import { openPath } from "@tauri-apps/plugin-opener";
    import { onMount } from "svelte";
    import { fade, fly } from "svelte/transition";
    import { rightClickedTracks } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import Icon from "../ui/Icon.svelte";

    function getDurationText(durationInSeconds: number) {
        return `${(~~(durationInSeconds / 60))
            .toString()
            .padStart(2, "0")}:${(~~(durationInSeconds % 60))
            .toString()
            .padStart(2, "0")}`;
    }

    // File(s) table
    let tableOuterContainer;
    let tableInnerScrollArea;
    let showTableTopScrollShadow = false;
    let showTableBottomScrollShadow = false;

    function onTableResize() {
        console.log("scrollTop");
        // Check scroll area size, add shadows if necessary
        if (tableInnerScrollArea) {
            showTableTopScrollShadow =
                tableInnerScrollArea.scrollTop > 0 &&
                tableInnerScrollArea.scrollHeight >
                    tableInnerScrollArea.clientHeight;
            showTableBottomScrollShadow =
                tableInnerScrollArea.scrollHeight >
                tableInnerScrollArea.clientHeight;
        }
    }

    function onTableScroll(e) {
        const tableHeight = tableOuterContainer.clientHeight;
        const scrollPercentage =
            tableInnerScrollArea.scrollTop /
            (tableInnerScrollArea.scrollHeight -
                tableInnerScrollArea.clientHeight);
        // Show top shadow
        showTableTopScrollShadow =
            tableInnerScrollArea.scrollTop > 0 &&
            tableInnerScrollArea.scrollHeight >
                tableInnerScrollArea.clientHeight;
    }

    onMount(() => {
        onTableResize();
    });
</script>

<section class="file-section boxed" bind:this={tableOuterContainer}>
    {#if $rightClickedTracks.length}
        {#if showTableTopScrollShadow}
            <div in:fade={{ duration: 150 }} class="top-shadow" />
        {/if}
        {#if showTableBottomScrollShadow}
            <div in:fly={{ duration: 150, y: 20 }} class="bottom-shadow" />
        {/if}
        <div
            class="file-info-container"
            bind:this={tableInnerScrollArea}
            on:scroll={onTableScroll}
        >
            <table class="file-info">
                <thead>
                    <tr>
                        <td>{$LL.trackInfo.file()}</td>
                        <td>{$LL.trackInfo.duration()}</td>
                        <td>{$LL.trackInfo.codec()}</td>
                    </tr>
                </thead>
                {#each $rightClickedTracks as track}
                    <tr>
                        <td class="file-path">
                            <div
                                on:click={() =>
                                    openPath(
                                        track.path.replace(track.file, ""),
                                    )}
                            >
                                <span>
                                    <Icon
                                        icon="bi:file-earmark-play"
                                        size={12}
                                    />
                                </span>
                                {track.file}
                            </div>
                        </td>

                        <td>
                            <p>
                                {getDurationText(track.fileInfo.duration)}
                            </p>
                        </td>
                        <td class="file-audio-info">
                            <p>
                                {track.fileInfo.codec} |
                                {track.fileInfo.tagType} |
                                {track.fileInfo.sampleRate} hz |
                                {#if track.fileInfo.bitDepth}
                                    {track.fileInfo.bitDepth}
                                    {$LL.trackInfo.bit()}
                                {/if}
                            </p>
                        </td>
                    </tr>
                {/each}
            </table>
        </div>
    {:else}
        <p>{$LL.trackInfo.noMetadata()}</p>
    {/if}
</section>

<style lang="scss">
    .file-section {
        padding: 0 1em;

        .section-title {
            top: -10px;
            margin: 0;
        }

        .top-shadow {
            font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
            pointer-events: none;
            background: linear-gradient(
                to bottom,
                hsl(320, 4.92%, 11.96%) 0%,
                hsla(320, 4.92%, 11.96%, 0.988) 2.6%,
                hsla(320, 4.92%, 11.96%, 0.952) 5.8%,
                hsla(320, 4.92%, 11.96%, 0.898) 9.7%,
                hsla(320, 4.92%, 11.96%, 0.828) 14.3%,
                hsla(320, 4.92%, 11.96%, 0.745) 19.5%,
                hsla(320, 4.92%, 11.96%, 0.654) 25.3%,
                hsla(320, 4.92%, 11.96%, 0.557) 31.6%,
                hsla(320, 4.92%, 11.96%, 0.458) 38.5%,
                hsla(320, 4.92%, 11.96%, 0.361) 45.9%,
                hsla(320, 4.92%, 11.96%, 0.268) 53.9%,
                hsla(320, 4.92%, 11.96%, 0.184) 62.2%,
                hsla(320, 4.92%, 11.96%, 0.112) 71.1%,
                hsla(320, 4.92%, 11.96%, 0.055) 80.3%,
                hsla(320, 4.92%, 11.96%, 0.016) 90%,
                hsla(320, 4.92%, 11.96%, 0) 100%
            );
            height: 40px;
            width: 100%;
            position: absolute;
            top: 0;
            right: 0;
            left: 0;
            z-index: 4;
            opacity: 0.5;
        }
        .bottom-shadow {
            font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
            pointer-events: none;
            background: linear-gradient(
                to top,
                hsl(320, 4.92%, 11.96%) 0%,
                hsla(320, 4.92%, 11.96%, 0.988) 2.6%,
                hsla(320, 4.92%, 11.96%, 0.952) 5.8%,
                hsla(320, 4.92%, 11.96%, 0.898) 9.7%,
                hsla(320, 4.92%, 11.96%, 0.828) 14.3%,
                hsla(320, 4.92%, 11.96%, 0.745) 19.5%,
                hsla(320, 4.92%, 11.96%, 0.654) 25.3%,
                hsla(320, 4.92%, 11.96%, 0.557) 31.6%,
                hsla(320, 4.92%, 11.96%, 0.458) 38.5%,
                hsla(320, 4.92%, 11.96%, 0.361) 45.9%,
                hsla(320, 4.92%, 11.96%, 0.268) 53.9%,
                hsla(320, 4.92%, 11.96%, 0.184) 62.2%,
                hsla(320, 4.92%, 11.96%, 0.112) 71.1%,
                hsla(320, 4.92%, 11.96%, 0.055) 80.3%,
                hsla(320, 4.92%, 11.96%, 0.016) 90%,
                hsla(320, 4.92%, 11.96%, 0) 100%
            );
            height: 40px;
            width: 100%;
            position: absolute;
            bottom: 0;
            right: 0;
            left: 0;
            z-index: 4;
            opacity: 0.5;
        }
        .file-info-container {
            max-height: 200px;
            overflow-y: auto;
            position: relative;
            padding: 0.5em;
        }

        .file-info {
            table-layout: auto;
            display: table;
            justify-content: center;
            flex-wrap: wrap;
            gap: 1em;
            text-align: left;
            width: 100%;
            font-size: 13px;
            -webkit-border-horizontal-spacing: 0px;
            -webkit-border-vertical-spacing: 0px;
            border-collapse: collapse;
            /* border: 1px solid rgb(97, 92, 92); */
            > thead {
                > tr {
                }
                td {
                    opacity: 0.5;
                }
            }
            td:not(:first-child) {
                padding-left: 1em;
            }
            tr {
                > td {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    border-right: 5px solid transparent;

                    p {
                        padding: 0.2em 0.5em;
                        width: fit-content;
                        border-radius: 4px;
                        color: var(--text);
                        user-select: none;
                        cursor: default;
                        margin: 0;
                    }
                }

                .file-path {
                    margin: auto;
                    font-family: -apple-system, Avenir, Helvetica, Arial,
                        sans-serif;
                    cursor: default;
                    min-width: 170px;
                    max-width: 208px;

                    span {
                        vertical-align: middle;
                        position: relative;
                        display: flex;
                    }

                    & > div {
                        overflow: hidden;
                        text-overflow: ellipsis;
                        white-space: nowrap;
                        font-size: 13px;
                        width: 100%;
                        vertical-align: middle;
                        display: flex;
                        gap: 4px;
                    }

                    &:hover {
                        text-decoration: underline;
                    }
                }

                .file-audio-info {
                    p {
                        color: var(--text-secondary);
                    }
                }
            }
        }
    }
</style>
