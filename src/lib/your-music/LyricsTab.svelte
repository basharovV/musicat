<script lang="ts">
    import { debounce } from "lodash-es";

    import { parseSong, renderSong } from "chord-mark/lib/chord-mark.js";
    import "../../../node_modules/chord-mark-themes/scss/themes/dark1.scss";

    import hotkeys from "hotkeys-js";
    import { onDestroy } from "svelte";
    import { clickOutside } from "../../utils/ClickOutside";
    import Icon from "../ui/Icon.svelte";
    import {
        isFullScreenLyrics,
        os,
        songbookFileSavedTime
    } from "../../data/store";
    import Divider from "../ui/Divider.svelte";
    import Dropdown from "../ui/Dropdown.svelte";
    import { writeChordMarkToSongProject } from "../../data/ArtistsToolkitData";

    export let lyrics;
    export let onLyricsUpdated;
    export let enabled = false;
    let isEditing = false;

    $: {
        console.log("lyrics", lyrics);
    }
    $: hasLyrics = enabled && typeof lyrics === "string" && lyrics.length > 0;

    let editor;

    function onLyricsChanged(e) {
        console.log("onLyricsChanged", e.target.value);
        onLyricsUpdated(e.target.value);
    }

    function onKeyDown(e) {
        // Handle escape key to exit editing mode
        if (e.key === "Escape") {
            e.preventDefault();
            isEditing = false;
        }
        // All other keys have default behavior
    }

    // Exit editing mode on ESC using hotkeys.js
    hotkeys("esc", (event, handler) => {
        console.log("esc pressed");
        isEditing = false;
    });

    onDestroy(() => {
        hotkeys.unbind("esc");
    });

    interface LyricSection {
        lines: any[];
        numBars: number;
        numLines: number; // chord / lyrics pair lines
        viewProps: {
            maxBarsPerLine: number;
            longestLine: number;
        };
    }

    let isCollapsedStructure = false;
    let parsedLyrics: LyricSection[];
    let parsingError;

    // This is used to center the bars around the lyrics.
    // So even if the bar starts on the second word, it will be centered
    let maxOffset = 0;
    // Used to line up all the last bars of a line
    let longestLine = 0;

    $: if (lyrics && transposeValue !== null && view !== null) {
        getChordMarkContent(lyrics);
    }

    function getChordMarkContent(lyrics) {
        let parsed;
        try {
            parsed = parseSong(lyrics);
            console.log("parsed", parsed);
            parsingError = null;
        } catch (error) {
            console.error(error);
            parsingError = error;
            return;
        }
        let rendered = renderSong(parsed, {
            accidentalsType: "flat",
            printChordsDuration: "uneven",
            wrapChordLyricLines: true,
            transposeValue,
            chartType: view.value === "structure" ? "all" : view.value,
            customRenderer: (
                allLines,
                allRenderedLines,
                { alignChordsWithLyrics, alignBars }
            ) => {
                console.log("allLines", allLines);
                console.log("allRenderedLines", allRenderedLines);
                console.log("alignChordsWithLyrics", alignChordsWithLyrics);
                console.log("alignBars", alignBars);

                let sections = [];
                let section = {
                    lines: [],
                    numBars: 0,
                    numLines: 0,
                    viewProps: {
                        maxBarsPerLine: 0,
                        longestLine: 0
                    }
                };

                maxOffset = Math.max(
                    ...allLines
                        .filter((line) => line.type === "chord")
                        .map((line) => line.model.offset | 0)
                );

                let sectionBarIdx = 0;
                let sectionLineIdx = 0; // count lines (chords and lyrics)

                for (let i = 0; i < allLines.length; i++) {
                    let line = allLines[i];
                    console.log("line", line);
                    if (line.type.match(/(timeSignature)/)) {
                        console.log("skipping line", line);
                        continue;
                    }

                    // When we get to a lyric or chord line, add it to the current section
                    if (
                        line.type.match(/(lyric|chord)/) &&
                        i < allLines.length - 1
                    ) {
                        if (line.type === "chord") {
                            line.model.idx = sectionLineIdx++;
                            section.numLines++;
                            line.model.allBars.forEach((bar) => {
                                bar.idx = sectionBarIdx++;
                            });
                        }
                        section.lines.push(line);
                        section.viewProps.longestLine = Math.max(
                            ...section.lines
                                .filter((line) => line.type === "chord")
                                .map(
                                    (line) =>
                                        getTotalSpaces(line) +
                                        Math.max(
                                            maxOffset,
                                            line.model.offset | 0
                                        )
                                )
                        );

                        section.viewProps.maxBarsPerLine = Math.max(
                            ...section.lines
                                .filter((line) => line.type === "chord")
                                .map((line) => {
                                    return line.model.allBars.length;
                                })
                        );
                        section.numBars += line.model?.allBars?.length || 0;
                    }
                    // When we get to a new section, or reached the end, add the current section
                    else {
                        if (!line.type.match(/(sectionLabel|emptyLine)/)) {
                            section.lines.push(line);
                        }

                        if (
                            i === allLines.length - 1 ||
                            (line.type === "sectionLabel" &&
                                i > 0 &&
                                section?.lines.length)
                        ) {
                            sections.push(section);
                            // Reset the section
                            section = {
                                lines:
                                    line.type === "sectionLabel" ? [line] : [],
                                numBars: 0,
                                numLines: 0,
                                viewProps: {
                                    longestLine: 0,
                                    maxBarsPerLine: 0
                                }
                            };
                            sectionBarIdx = 0;
                            sectionLineIdx = 0;
                        } else {
                            // First section label
                            section.lines.push(line);
                        }

                        continue;
                    }
                }

                console.log("sections", sections);
                parsedLyrics = sections;
                return "";
            }
        });
        console.log("rendered", rendered);
        return rendered;
    }

    /** Calculate total of all chords spacesAfter in all bars */
    function getTotalSpaces(line) {
        let totalSpaces = 0;
        line.model.allBars.forEach((bar, idx) => {
            bar.allChords.forEach((chord) => {
                totalSpaces += chord.spacesAfter + chord.symbol.length;
            });
        });
        return totalSpaces;
    }

    const fontSizes = [
        "font-12",
        "font-14",
        "font-16",
        "font-20",
        "font-24",
        "font-32",
        "font-48"
    ];

    let currentFontSizeIdx = 2;
    $: fontSize = fontSizes[currentFontSizeIdx];

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
    let isFullScreen = false;
    function toggleFullScreenLyrics() {
        $isFullScreenLyrics = !$isFullScreenLyrics;
    }

    // Shortcuts

    let modifier = $os === "macos" ? "cmd" : "ctrl";
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

    // Transposition
    let transposeValue = 0;

    function transposeUp() {
        transposeValue++;
    }

    function transposeDown() {
        transposeValue--;
    }

    // View options

    const viewOptions = [
        { label: "Lyrics with chords", value: "all" },
        { label: "Lyrics only", value: "lyrics" },
        { label: "Chords only", value: "chords" },
        { label: "Chords with first line", value: "chordsFirstLyricLine" },
        { label: "Structure", value: "structure" }
    ];
    let view = viewOptions[0];

    $: if (view.value === "structure") {
        isCollapsedStructure = true;
    } else {
        isCollapsedStructure = false;
    }
</script>

<div class="container" class:full-screen={$isFullScreenLyrics}>
    <div class="lyrics-options">
        <div class="view-selector">
            <Dropdown options={viewOptions} bind:selected={view} />
        </div>
        <Divider />
        <Icon icon="transpose-up" onClick={transposeUp} />
        <Icon icon="transpose-down" onClick={transposeDown} />
        <Divider />
        <Icon icon="mdi:format-font-size-decrease" onClick={decreaseFontSize} />
        <Icon icon="mdi:format-font-size-increase" onClick={increaseFontSize} />
        <Icon
            icon="icon-park-outline:full-screen-one"
            onClick={toggleFullScreenLyrics}
        />
    </div>

    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="lyrics-container {fontSize}" on:click={() => (isEditing = true)}>
        {#if parsingError || isEditing || !hasLyrics}
            <textarea
                placeholder="Start typing..."
                bind:this={editor}
                on:input={debounce(onLyricsChanged, 300)}
                on:keydown={onKeyDown}
                bind:value={lyrics}
                use:clickOutside={() => (isEditing = false)}
            />
        {:else if parsedLyrics && !isEditing}
            <div
                class="lyrics-viewer {fontSize}"
                on:focus={() => (isEditing = true)}
                on:blur={() => (isEditing = false)}
            >
                <div class="lyrics" on:click={() => (isEditing = true)}>
                    {#each parsedLyrics as section, sectionIdx}
                        <section class:collapsed={isCollapsedStructure}>
                            {#each section.lines as line, lineIdx}
                                {#if lineIdx > 0}
                                    <br />
                                {/if}
                                {#if line.type === "sectionLabel"}
                                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                                    <p
                                        class="label"
                                        on:click|stopPropagation={() =>
                                            (isCollapsedStructure =
                                                !isCollapsedStructure)}
                                    >
                                        {line.model.rendered.label}
                                        <span class="bar-count">
                                            ({section.numBars} bars)
                                        </span>
                                    </p>
                                {:else if line.type === "chord"}
                                    {@const chordLineOffset = Math.max(
                                        line.model.offset | 0,
                                        maxOffset
                                    )}
                                    {@const remainingSpaces = Math.max(
                                        0,
                                        section.viewProps.longestLine -
                                            chordLineOffset -
                                            getTotalSpaces(line) +
                                            (section.viewProps.maxBarsPerLine -
                                                line.model.allBars.length) *
                                                2
                                    )}

                                    <span class="chords">
                                        <span class="offset">
                                            {#each [...Array(chordLineOffset).keys()] as space}
                                                &nbsp;
                                            {/each}
                                        </span>
                                        {#each line.model.allBars as bar, barIdx}
                                            <span
                                                class="bar"
                                                class:last={bar.idx ===
                                                    section.numBars - 1 &&
                                                    line.model.idx ===
                                                        section.numLines - 1}
                                            >
                                                {#each bar.allChords as chord, chordIdx}
                                                    {bar.shouldPrintChordsDuration
                                                        ? chord.symbol +
                                                          ".".repeat(
                                                              chord.duration
                                                          )
                                                        : chord.symbol}{#each [...Array(Math.max(0, chord.spacesAfter)).keys()] as space}&nbsp;{/each}
                                                {/each}
                                            </span>
                                        {/each}
                                    </span>
                                {:else if line.type === "lyric"}
                                    <span class="lyric-line">
                                        <span class="offset">
                                            {#each [...Array(Math.max(0, maxOffset - (line.model.chordPositions?.at(0) | 0))).keys()] as space}
                                                &nbsp;
                                            {/each}
                                        </span>
                                        <span class="lyric">
                                            {line.model.lyrics}
                                        </span></span
                                    >
                                {/if}

                                {#if lineIdx === section.lines.length - 1}
                                    <br />
                                {/if}
                            {/each}
                        </section>
                        <br />
                    {/each}
                </div>
            </div>
        {:else}
            <br />
            <p>Create a project to add lyrics</p>
        {/if}
    </div>
</div>

<style lang="scss">
    .container {
        width: 100%;
        height: 100%;
        position: relative;
        overflow: hidden;
        &.full-screen {
            position: fixed;
            top: 0;
            bottom: 0;
            right: 0;
            left: 0;
            width: 100%;
            height: 100%;
            z-index: 20;
            background-color: rgb(from var(--panel-background) r g b / 1);
        }
    }

    .lyrics-options {
        position: absolute;
        top: 0.5em;
        right: 2em;
        margin: 0;
        height: auto;
        width: auto;
        display: flex;
        justify-content: flex-end;
        z-index: 23;
        gap: 0.5em;
        background: color-mix(
            in srgb,
            var(--panel-background) 90%,
            transparent
        );
        backdrop-filter: blur(8px);
        border-radius: 4px;
        padding: 0.5em;
        border: 1px solid
            color-mix(in srgb, var(--type-bw-inverse) 11%, transparent);

        .view-selector {
            display: flex;
            gap: 0.5em;
            align-items: center;
        }
    }

    .lyrics-container {
        width: 100%;
        height: 100%;
        position: relative;
        overflow: hidden;
        font-family: monospace;
        &.font-12 {
            font-size: 12px;
        }

        &.font-14 {
            font-size: 14px;
        }

        &.font-16 {
            font-size: 16px;
        }

        &.font-20 {
            font-size: 20px;
        }

        &.font-24 {
            font-size: 24px;
        }

        &.font-32 {
            font-size: 32px;
        }

        &.font-48 {
            font-size: 48px;
        }
    }

    textarea {
        appearance: none;
        width: 100%;
        height: 100%;
        padding: 4rem 2em;
        outline: none;
        border: none;
        background: none;
        font-size: inherit;
        font-family: monospace;
        line-height: 1.8em;
        resize: none;
        color: var(--text);
    }

    .lyrics-viewer {
        width: 100%;
        height: 100%;
        position: relative;
        overflow: auto;

        .lyrics {
            padding: 4rem 2em;
            overflow-y: auto;
            line-height: 1.8em;
            font-size: inherit;
            p {
                margin: 0;
            }

            section {
                display: inline-block;
                &:hover {
                    .bar-count {
                        opacity: 1;
                    }
                    cursor: pointer;
                }

                &.collapsed {
                    width: 100%;
                    height: fit-content;
                    span:not(.bar-count),
                    br {
                        display: none;
                    }
                    .bar-count {
                        opacity: 1 !important;
                    }
                }
            }

            .label {
                display: inline;
                margin-bottom: 1em;
                position: relative;
                color: var(--text-secondary);

                &:not(:first-of-type) {
                    margin-top: 1em;
                }

                &:hover {
                    background-color: color-mix(
                        in srgb,
                        var(--accent) 40%,
                        transparent
                    );
                }

                .bar-count {
                    opacity: 0;
                    transition: opacity 0.1s ease-in;
                    font-size: 14px;
                    position: relative;
                    &::before {
                        content: "";
                    }
                }
            }

            .chords {
                color: color-mix(in srgb, var(--accent) 70%, transparent);
                white-space: nowrap;
            }

            .bar {
                position: relative;
                &:not(:last-child) {
                    &::after {
                        right: 5px;
                        display: block;
                    }
                }
                &.last {
                    &::after {
                        display: block;
                    }
                }
                &::before,
                &::after {
                    background-color: color-mix(
                        in srgb,
                        var(--accent) 40%,
                        transparent
                    );
                }
                &::before {
                    content: "";
                    position: absolute;
                    top: -0.3em;
                    left: -12px;
                    height: 1.8em;
                    width: 2px;
                    opacity: 0.5;
                }

                &::after {
                    content: "";
                    position: absolute;
                    top: -0.3em;
                    height: 1.8em;
                    right: -12px;
                    width: 2px;
                    opacity: 0.5;
                    display: none;
                }
            }

            .lyric-line {
                white-space: nowrap;
            }
            .lyric {
                white-space: nowrap;
                transition: all 0.2s cubic-bezier(0.075, 0.82, 0.165, 1);
                &:hover {
                    background-color: color-mix(
                        in srgb,
                        var(--accent) 40%,
                        transparent
                    );
                    padding: 2px 5px;
                    border-radius: 3px;
                    cursor: pointer;
                }
            }
        }
    }
</style>
