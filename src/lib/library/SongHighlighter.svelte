<script lang="ts">
    import type { Song } from "src/App";

    import {
        expandedSongWithStems,
        isFindFocused,
        os,
        popupOpen,
        rightClickedTrack,
        rightClickedTracks,
        shouldFocusFind,
    } from "../../data/store";
    import { isInputFocused } from "../../utils/ActiveElementUtils";
    import { window as tauriWindow } from "@tauri-apps/api";

    /**
     * This class is used to highlight songs in the library
     * and support keyboard navigation (using arrow keys),
     * ranged selection (using shift + arrow keys), or
     * multiple selection (using ctrl/cmd + arrow keys).
     *
     * It's just to keep track of the state of the highlighter,
     * the click and key events are handled by the library component
     */

    export let songs: Song[] = [];
    export let songsHighlighted: Song[] = [];
    export let keyboardEnabled = false;
    export let onSongsHighlighted: (songs: Song[]) => void = null;

    let songIdxsHighlighted = new Set<number>();

    $: {
        songsHighlighted = [...songIdxsHighlighted.values()].map(
            (idx) => songs[idx],
        );
        if (songsHighlighted.length > 0) {
            $rightClickedTracks = songsHighlighted;
            $rightClickedTrack = null;
        } else {
            $rightClickedTracks = [];
            $rightClickedTrack = null;
        }
    }

    let isShiftPressed = false;
    let isMetaPressed = false;
    let currentWindow = tauriWindow.getCurrentWindow();

    currentWindow.listen("tauri://blur", (evt) => {
        isMetaPressed = false;
        isShiftPressed = false;
    });

    let previousKeyboardDirection: "up" | "down" | null = null;

    let rangeStartSongIdx: number | null = null;
    let rangeEndSongIdx: number | null = null;

    let lastIdx: number | null = null;

    function isMultiSelect() {
        return rangeStartSongIdx !== null && rangeEndSongIdx !== null;
    }

    export function isSongIdxHighlighted(songIdx: number) {
        return songsHighlighted.find((s) => s?.viewModel?.index === songIdx);
    }

    export function toggleHighlight(
        song: Song,
        idx,
        isKeyboardArrows: boolean,
        isDefault = false,
        isQueryChanged = false,
    ) {
        // Compensate for expanded stems

        console.log("song view model index", idx);
        // console.log("highlighted", song, idx, isKeyboardArrows, isDefault);
        if (isKeyboardArrows) {
            console.log("isShiftPressed", isShiftPressed);
            console.log("isMultiSelect", isMultiSelect());
            console.log("Range: ", rangeStartSongIdx, idx);
            if (isShiftPressed) {
                if (isMultiSelect() && idx > rangeEndSongIdx) {
                    // Going down
                    if (previousKeyboardDirection === "up") {
                        // Reversing direction - delete the current highlight as well as the next
                        idx -= 1;
                    }
                    console.log("Going down");
                    previousKeyboardDirection = "down";
                    if (songIdxsHighlighted.has(idx)) {
                        songIdxsHighlighted.delete(idx);
                    } else {
                        songIdxsHighlighted.add(idx);
                    }
                    songIdxsHighlighted = songIdxsHighlighted;
                    rangeEndSongIdx = idx;
                } else if (isMultiSelect() && idx < rangeEndSongIdx) {
                    if (previousKeyboardDirection === "down") {
                        // Reversing direction - delete the current highlight as well as the next
                        idx += 1;
                    }
                    previousKeyboardDirection = "up";
                    if (songIdxsHighlighted.has(idx)) {
                        songIdxsHighlighted.delete(idx);
                    } else {
                        songIdxsHighlighted.add(idx);
                    }
                    songIdxsHighlighted = songIdxsHighlighted;
                    rangeEndSongIdx = idx;
                } else {
                    // Not multi-select yet, entering multi-select mode
                    previousKeyboardDirection =
                        idx > rangeStartSongIdx ? "down" : "up";
                    songIdxsHighlighted.add(idx);
                    songIdxsHighlighted = songIdxsHighlighted;
                    rangeEndSongIdx = idx;
                }
            } else {
                console.log("ARROWS");
                songIdxsHighlighted.clear();
                songIdxsHighlighted.add(idx);
                songIdxsHighlighted = songIdxsHighlighted;
                rangeStartSongIdx = idx;
                rangeEndSongIdx = null;
            }

            $rightClickedTrack = null;
            $rightClickedTracks = songsHighlighted;
        } else if (isQueryChanged) {
            songIdxsHighlighted.clear();
            songIdxsHighlighted.add(idx);
            songIdxsHighlighted = songIdxsHighlighted;

            rangeStartSongIdx = idx;
            rangeEndSongIdx = null;
        } else if (
            isDefault &&
            $popupOpen !== "track-info" &&
            $rightClickedTracks?.length
        ) {
            songsHighlighted = $rightClickedTracks;
        } else if (
            isDefault &&
            $popupOpen !== "track-info" &&
            $rightClickedTrack
        ) {
            songsHighlighted = [$rightClickedTrack];
        } else {
            if (!isDefault) {
                $shouldFocusFind = { target: "search", action: "unfocus" };
            }

            // Cmd / ctrl + click
            if (isMetaPressed) {
                if (songIdxsHighlighted.has(idx)) {
                    songIdxsHighlighted.delete(idx);
                } else {
                    songIdxsHighlighted.add(idx);
                    rangeStartSongIdx = idx;
                }
                songIdxsHighlighted = songIdxsHighlighted;
            } // Shift + click
            else if (isShiftPressed) {
                // No previous selection
                if (rangeStartSongIdx === null) {
                    rangeStartSongIdx = idx;
                    rangeEndSongIdx = null;
                } else {
                    console.log("Creating range from ", rangeStartSongIdx, idx);
                    // Create range - Highlight all the songs in between
                    rangeEndSongIdx = idx;
                    // Shift-click on song higher up
                    if (rangeEndSongIdx < rangeStartSongIdx) {
                        let startIdx = rangeStartSongIdx;
                        rangeStartSongIdx = rangeEndSongIdx;
                        rangeEndSongIdx = startIdx;
                    }
                    for (let i = rangeStartSongIdx; i <= rangeEndSongIdx; i++) {
                        songIdxsHighlighted.add(i);
                        songIdxsHighlighted = songIdxsHighlighted;
                    }
                    rangeStartSongIdx = rangeEndSongIdx;
                    rangeEndSongIdx = null;
                    $rightClickedTrack = null;

                    // console.log("highlighted2", songsHighlighted);
                }
            } else {
                console.log("Single click");
                // Highlight single song, via a good old click
                songIdxsHighlighted.clear();
                songIdxsHighlighted.add(idx);
                songIdxsHighlighted = songIdxsHighlighted;

                rangeStartSongIdx = idx;
                rangeEndSongIdx = null;
            }
        }
        lastIdx = idx;
    }

    export function onQueryChanged(newSongs: Song[]) {
        songs = newSongs;
        toggleHighlight(songs[0], 0, false, false, true);
    }

    export function highlightAll() {
        songIdxsHighlighted = new Set(songs.map((s, i) => i));
        rangeStartSongIdx = 0;
        rangeEndSongIdx = songs.length - 1;
    }
    function unhighlightSong(idx: number) {
        songIdxsHighlighted.delete(idx);
        songIdxsHighlighted = songIdxsHighlighted;
    }

    /**
     * Keyboard events still work even if Track info popup is open (except Escape, which closes the popup)
     * This allows to change the selection with arrow keys,
     * and even add multiple selections when holding shift without closing the popup
     * @param event
     */
    export function onKeyDown(event: KeyboardEvent) {
        // Ignore keyboard events if any input/textarea (except search input) is focused
        if (isInputFocused() && !$isFindFocused) {
            return;
        }

        if (
            !isInputFocused() &&
            event.code === "KeyA" &&
            (($os === "macos" && event.metaKey) || event.ctrlKey)
        ) {
            event.preventDefault();

            songsHighlighted = [...songs];
        }

        if (event.key === "Shift") {
            isShiftPressed = true;
        } else if ($os !== "macos" && event.key === "Control") {
            isMetaPressed = true;
        } else if ($os === "macos" && event.key === "Meta") {
            isMetaPressed = true;
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            if (lastIdx > 0) {
                toggleHighlight(songs[lastIdx - 1], lastIdx - 1, true);
            }
        } else if (event.key === "ArrowDown") {
            event.preventDefault();
            if (lastIdx < songs.length) {
                toggleHighlight(songs[lastIdx + 1], lastIdx + 1, true);
            }
        } else if (event.key === "Escape" && $popupOpen !== "track-info") {
            console.log("restting, popup open", $popupOpen);
            reset();
        }
    }

    export function onKeyUp(event) {
        if (event.keyCode === 16) {
            isShiftPressed = false;
        } else if (event.key === "Meta" || event.key === "Control") {
            isMetaPressed = false;
        }
    }

    export function reset() {
        songIdxsHighlighted.clear();
        songIdxsHighlighted = songIdxsHighlighted;
        rangeStartSongIdx = null;
        rangeEndSongIdx = null;
    }
</script>
