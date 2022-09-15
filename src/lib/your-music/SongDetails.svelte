<script lang="ts">
    import type { Song, SongProject } from "src/App";
    import { debounce } from "lodash-es";
    import { autoWidth } from "../../utils/AutoWidth";
    import { db } from "../../data/db";
    import MusicTab from "./MusicTab.svelte";
    import LyricsTab from "./LyricsTab.svelte";
    import OtherTab from "./OtherTab.svelte";
    import { cloneDeep, isEqual, uniqBy } from "lodash-es";
    import { os } from "../../data/store";
    import hotkeys from "hotkeys-js";
    import { onDestroy } from "svelte";

    export let songProject: SongProject;
    let songProjectClone = cloneDeep(songProject);
    $: isProject = songProject?.id;

    let bpmTicker = 1;
    let bpmInterval;

    // Eg. 127 beats per minute = 1 beat every 0.4724409449 seconds
    function startBPMTicker(bpm: number) {
        if (bpmTicker === 1 && !bpmInterval) {
            bpmInterval = setInterval(() => {
                if (songProject.bpm !== undefined) {
                    bpmTicker++;
                } else {
                    stopBpmTicker();
                }
            }, 60000 / bpm / 2);
        }
    }

    function stopBpmTicker() {
        clearInterval(bpmInterval);
        bpmInterval = null;
        bpmTicker = 1;
    }

    function onBpmUpdated(e) {
        if (e.target.value) {
            stopBpmTicker();
            startBPMTicker(e.target.value);
        } else {
            stopBpmTicker();
        }
    }

    function onBpmFocus() {
        startBPMTicker(songProject?.bpm);
    }

    $: {
        // Will only have ID once it's a song project
        // So normal Songs won't be saved automatically,
        // needs to be converted into a project first

        if (
            songProject?.id !== undefined &&
            songProject.id !== songProjectClone?.id
        ) {
            songProjectClone = cloneDeep(songProject);
        } else if (
            songProject?.id !== undefined &&
            songProject.id === songProjectClone?.id &&
            !isEqual(songProject, songProjectClone)
        ) {
            db.songProjects.put(songProjectClone);
        } else {
            songProjectClone = cloneDeep(songProject);
        }
    }

    const tabs = ["music", "lyrics", "other"];
    let selectedTab = tabs[0];

    export let onDeleteSongProject;

    const fontSizes = [
        "font-12",
        "font-14",
        "font-20",
        "font-24",
        "font-32",
        "font-48"
    ];

    let currentFontSizeIdx = 1;
    $: currentFontSize = fontSizes[currentFontSizeIdx];

    function increaseFontSize() {
        currentFontSizeIdx =
            currentFontSizeIdx <= fontSizes.length - 2
                ? currentFontSizeIdx + 1
                : fontSizes.length - 1;
    }
    function decreaseFontSize() {
        currentFontSizeIdx =
            currentFontSizeIdx > 0 ? currentFontSizeIdx - 1 : 0;
    }
    let fullScreenLyrics = false;
    function toggleFullScreenLyrics() {
        fullScreenLyrics = !fullScreenLyrics;
    }

    // Shortcuts

    let modifier = $os === "Darwin" ? "cmd" : "ctrl";
    hotkeys(`${modifier}+=`, function (event, handler) {
        increaseFontSize();
    });
    hotkeys(`${modifier}+-`, function (event, handler) {
        decreaseFontSize();
    });

    onDestroy(() => {
        hotkeys.unbind(`${modifier}+=`);
        hotkeys.unbind(`${modifier}+-`);
    });
</script>

<container class:full-screen={fullScreenLyrics}>
    <header>
        {#if songProjectClone}
            <div class="info">
                <span>
                    <p>"</p>
                    <input
                        use:autoWidth
                        bind:value={songProjectClone.title}
                        placeholder="?"
                    />
                    <p>"</p></span
                >
            </div>
            {#if isProject}
                <iconify-icon
                    class="delete-icon"
                    icon="ant-design:delete-outlined"
                    on:click={() => {
                        onDeleteSongProject(songProject);
                    }}
                />
            {/if}
        {:else}
            <p>Select a song on the left</p>
        {/if}
    </header>

    {#if songProjectClone}
        <div class="details">
            <div>
                <p>in album:</p>
                <input
                    bind:value={songProjectClone.album}
                    placeholder="add an album"
                />
            </div>
            <div>
                <p>music written by:</p>
                <input
                    bind:value={songProjectClone.musicComposedBy}
                    placeholder="add a composer"
                />
            </div>
            <div>
                <p>lyrics written by:</p>
                <input
                    bind:value={songProjectClone.lyricsWrittenBy}
                    placeholder="add a lyricist"
                />
            </div>
        </div>

        <div class="music-info">
            <div>
                <p class:bpm={bpmTicker % 2 === 0}>BPM:</p>
                <input
                    bind:value={songProjectClone.bpm}
                    on:input={debounce(onBpmUpdated, 300)}
                    on:focus={onBpmFocus}
                    on:blur={stopBpmTicker}
                    placeholder="bpm"
                    max="300"
                    min="20"
                />
            </div>
            <div>
                <p>key:</p>
                <input bind:value={songProjectClone.key} placeholder="key" />
            </div>
        </div>
        <div class="content-container">
            <div class="content-header">
                <div class="content-tabs">
                    {#each tabs as tab}
                        <div
                            class="tab"
                            class:selected={selectedTab === tab}
                            on:click={() => {
                                selectedTab = tab;
                            }}
                        >
                            <p>{tab}</p>
                        </div>
                    {/each}
                </div>
                <div class="lyrics-options">
                    <iconify-icon
                        icon="mdi:format-font-size-decrease"
                        on:click={decreaseFontSize}
                    />
                    <iconify-icon
                        icon="mdi:format-font-size-increase"
                        on:click={increaseFontSize}
                    />
                    <iconify-icon
                        icon="icon-park-outline:full-screen-one"
                        on:click={toggleFullScreenLyrics}
                    />
                </div>

                <div class="bg-gradient" />
            </div>
            <content class={selectedTab}>
                {#if selectedTab === "music"}
                    <MusicTab
                        recordings={songProjectClone.recordings}
                        {songProject}
                    />
                {:else if selectedTab === "lyrics"}
                    <LyricsTab
                        fontSize={currentFontSize}
                        isFullScreen={fullScreenLyrics}
                    />
                {:else if selectedTab === "other"}
                    <OtherTab />
                {/if}
            </content>
        </div>
    {/if}
</container>

<style lang="scss">
    container {
        display: grid;
        grid-template-rows: auto auto auto 1fr;
        height: 100%;

        &.full-screen {
            position: fixed;
            top: 0;
            bottom: 0;
            right: 0;
            left: 0;
            width: 100%;
            height: 100%;
            z-index: 20;
            background-color: #252126;

            grid-template-rows: 1fr;
            .details,
            .music-info {
                display: none;
            }
            header {
                padding: 2em;
                position: absolute;
            }

            .lyrics-options {
                position: absolute;
                top: 5px;
                right: 0;
            }
            .delete-icon {
                display: none;
            }
            .content-tabs {
                margin-top: 6em;
            }
            content {
                top: 7em;
                &.lyrics {
                    top: 0;
                }
            }
            .bg-gradient {
                display: none;
            }
        }

        header {
            display: grid;
            grid-template-columns: 1fr auto;
            align-items: center;
            padding: 2em;
            gap: 1em;

            h2 {
                font-family: "Snake";
                font-size: 3em;
                margin: 0;
            }

            .info {
                display: flex;
                flex-direction: column;
                span {
                    font-family: "Snake";
                    display: inline-flex;
                    p {
                        font-size: 3em;
                        margin: 0;
                    }
                    input {
                        font-family: "Snake";
                        font-size: 3em;
                        padding: 0;
                        min-width: 20px;
                    }
                }
            }
        }

        .details {
            padding: 0 2em 1em 2em;
            > div {
                display: flex;
                flex-direction: row;
                align-items: center;
                width: 100%;
                p {
                    opacity: 0.7;
                    margin: 0;
                    white-space: nowrap;
                }
            }
        }

        .music-info {
            padding: 0 2em 0 2em;
            display: flex;
            flex-direction: row;
            justify-content: flex-start;
            border-top: 1px solid rgba(255, 255, 255, 0.093);
            border-bottom: 1px solid rgba(255, 255, 255, 0.093);

            > div {
                padding: 0.3em 0;
                display: inline-flex;
                align-items: center;
                p {
                    opacity: 0.7;
                    margin: 0;
                    position: relative;

                    &.bpm::before {
                        content: "•";
                        position: absolute;
                        color: cyan;
                        left: -20px;
                        top: -2px;
                        font-size: 30px;
                    }
                }
                input {
                    max-width: 60px;
                }
            }
        }

        .content-container {
            position: relative;
            overflow: overlay;
            height: 100%;
            overflow: hidden;

            .content-header {
                position: sticky;
                top: 0;
                width: 100%;
                display: flex;
                align-items: center;
                justify-content: space-between;
                z-index: 3;
            }

            .bg-gradient {
                height: 80px;
                width: 100%;
                top: 0;
                position: absolute;
                z-index: -1;
                background: linear-gradient(
                    to bottom,
                    #252126 0%,
                    #242126c4 60%,
                    transparent 100%
                );
            }
        }

        content {
            position: absolute;
            top: 2em;
            left: 0;
            right: 0;
            bottom: 0;
            z-index: 0;
        }

        .content-tabs {
            display: flex;
            justify-content: center;
            gap: 1em;
            padding: 0.3em 2em;
            > .tab {
                padding: 0.5em 0;
                cursor: default;
                opacity: 0.6;
                p {
                    margin: 0;
                    text-transform: capitalize;
                }
                &:hover {
                    opacity: 0.5;
                    /* border-bottom: 2px solid rgba(255, 255, 255, 0.487); */
                }
                &:active {
                    opacity: 0.4;
                }

                &.selected {
                    opacity: 1;
                    border-bottom: 2px solid white;
                }
            }
        }

        .lyrics-options {
            display: flex;
            flex-direction: row;
            gap: 3px;
            align-items: center;
            margin-right: 20px;

            iconify-icon {
                padding: 6px;
                font-size: 20px;
                border-radius: 4px;
                &:hover {
                    background-color: rgba(0, 0, 0, 0.457);
                }
            }
        }
    }
    p {
        cursor: default;
    }

    input {
        width: 100%;
        align-items: center;
        padding: 0.2em 0.5em;
        font-size: 14px;
        outline: none;
        background: none;
        border: none;

        &::placeholder {
            color: rgb(105, 105, 105);
        }
    }
</style>
