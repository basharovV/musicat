<!-- SongDataGrid.svelte -->

<script lang="ts">
    import { onMount } from "svelte";
    import {
        Group,
        Layer,
        Rect,
        Stage,
        Text,
        Line,
        Path,
        type KonvaDragTransformEvent
    } from "svelte-konva";

    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import "iconify-icon";
    import { debounce } from "lodash-es";
    import type { Song } from "src/App";
    import { onDestroy } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { flip } from "svelte/animate";
    import { cubicInOut, quadOut } from "svelte/easing";
    import { get } from "svelte/store";
    import { fade, fly } from "svelte/transition";
    import { openTauriImportDialog, runScan } from "../../data/LibraryImporter";
    import BuiltInQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";

    import {
        bottomBarNotification,
        currentSong,
        currentSongIdx,
        draggedColumnIdx,
        draggedSongs,
        emptyDropEvent,
        fileDropHandler,
        importStatus,
        isFolderWatchUpdate,
        isPlaying,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isTrackInfoPopupOpen,
        libraryScrollPos,
        nextUpSong,
        os,
        playlist,
        playlistIsAlbum,
        queriedSongs,
        query,
        rightClickedTrack,
        rightClickedTracks,
        shouldShowToast,
        shouldFocusFind,
        singleKeyShortcutsEnabled,
        smartQuery,
        smartQueryInitiator,
        songsJustAdded,
        uiView,
        compressionSelected
    } from "../../data/store";
    import {
        moveArrayElement,
        swapArrayElements
    } from "../../utils/ArrayUtils";
    import { getFlagEmoji } from "../../utils/EmojiUtils";
    import AudioPlayer from "../player/AudioPlayer";
    import ColumnPicker from "./ColumnPicker.svelte";
    import ImportPlaceholder from "./ImportPlaceholder.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    import { codes } from "../data/CountryCodes";
    import { getQueryPart } from "../smart-query/QueryParts";
    import SmartQueryBuilder from "../smart-query/SmartQueryBuilder.svelte";
    import SmartQueryMainHeader from "../smart-query/SmartQueryMainHeader.svelte";
    import SmartQueryResultsPlaceholder from "../smart-query/SmartQueryResultsPlaceholder.svelte";
    import { UserQueryPart } from "../smart-query/UserQueryPart";
    import { optionalTippy } from "../ui/TippyAction";
    import BottomBar from "./BottomBar.svelte";

    export let allSongs = null;
    export let dim = false;

    export let isLoading = false;

    $: songs = $allSongs
        ?.filter((song: Song) => {
            if ($compressionSelected === "lossless") {
                return song?.fileInfo?.lossless;
            } else if ($compressionSelected === "lossy") {
                return song?.fileInfo?.lossless === false;
            } else return song !== undefined;
        })
        .reduce(
            (status, s, idx, songsArray) => {
                if (s.viewModel) {
                    s.viewModel.index = idx;
                } else {
                    s.viewModel = {
                        index: idx
                    };
                }
                if (s?.album !== status.state.previousAlbum) {
                    if (status.state.firstSongInPreviousAlbum) {
                        // Set the view model property here
                        const song =
                            songsArray[status.state.firstSongInPreviousAlbum];
                        if (song.viewModel) {
                            song.viewModel.isFirstAlbum = true;
                        } else {
                            song.viewModel = {
                                isFirstAlbum: true,
                                isFirstArtist: false
                            };
                        }
                    }
                    status.state.firstSongInPreviousAlbum = idx;
                } else {
                    status.state.tracksInAlbum++;
                }
                if (s?.artist !== status.state.previousArtist) {
                    if (status.state.firstSongInPreviousArtist) {
                        // Set the view model property here
                        const song =
                            songsArray[status.state.firstSongInPreviousArtist];
                        if (song.viewModel) {
                            song.viewModel.isFirstArtist = true;
                        } else {
                            song.viewModel = {
                                isFirstAlbum: false,
                                isFirstArtist: true
                            };
                        }
                    }
                    status.state.firstSongInPreviousArtist = idx;
                } else {
                    status.state.albumsInArtist++;
                }
                status.state.previousAlbum = s?.album;
                status.state.previousArtist = s?.artist;
                return {
                    songs: songsArray,
                    state: status.state
                };
            },
            {
                state: {
                    previousAlbum: null,
                    previousArtist: null,
                    firstSongInPreviousAlbum: null,
                    firstSongInPreviousArtist: null
                }
            }
        ).songs;

    const DEFAULT_FIELDS = [
        {
            name: "Title",
            value: "title",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true
            }
        },
        {
            name: "Artist",
            value: "artist",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true
            }
        },
        {
            name: "Composer",
            value: "composer",
            show: false,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true
            }
        },
        {
            name: "Album",
            value: "album",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true
            }
        },
        {
            name: "Track",
            value: "trackNumber",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Year",
            value: "year",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Genre",
            value: "genre",
            show: true,
            viewProps: {
                width: 100,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Origin",
            value: "originCountry",
            show: true,
            viewProps: {
                width: 120,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Duration",
            value: "duration",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false
            }
        }
    ];

    export let fields = DEFAULT_FIELDS;

    let hoveredColumnIdx = null;
    let hoveredSongIdx = null; // Index is slice-specific
    let columnToInsertIdx = null;
    let columnToInsertXPos = 0;

    $: isOrderChanged =
        JSON.stringify(
            displayFields.filter((f) => f.show).map((f) => f.value)
        ) !==
        JSON.stringify(
            DEFAULT_FIELDS.filter((f) => f.show).map((f) => f.value)
        );

    let showColumnPicker = false;
    let columnPickerPos;

    $: displayFields = fields.filter((f) => f.show);
    $: numColumns = fields.filter((f) => f.show).length;

    let songsSlice: Song[];
    let songsStartSlice = 0;
    let songsEndSlice = 0;
    let canvas;

    let shouldRender = false;

    let scrollContainer: HTMLDivElement;
    let container: HTMLElement;
    let stage;
    let isScrollable = false;
    let prevScrollPos = 0;
    let scrollPos = 0;
    let scrollOffset = 0;
    let scrollNormalized = 0;

    // Sandwich
    let sandwichTopHeight = 0;
    let sandwichBottomY = 0;
    let sandwichBottomHeight = 0;

    let width = 0;
    let viewportHeight = 0;
    let virtualViewportHeight = 0; // With padding
    let ctx: CanvasRenderingContext2D;
    let dpr;

    // CONSTANTS
    const HEADER_HEIGHT = 26;
    const ROW_HEIGHT = 26;
    const BORDER_WIDTH = 1;
    const SCROLL_PADDING = 200;
    const DUMMY_COUNT = 5;
    const DUMMY_PADDING = DUMMY_COUNT * ROW_HEIGHT;

    // COLORS
    const BG_COLOR = "rgba(36, 33, 34, 0.688)";
    const HEADER_BG_COLOR = "#71658e7e";
    const OFFSCREEN_BG_COLOR = "#71658e3b";
    const HEADER_BG_COLOR_HOVERED = "#604d8d";
    const TEXT_COLOR = "rgb(211, 211, 211)";
    const HIGHLIGHT_BG_COLOR = "#2e3357";
    const ROW_BG_COLOR = "transparent";
    const ROW_BG_COLOR_HOVERED = "#1f1f1f";
    const PLAYING_BG_COLOR = "#5123dd";
    const PLAYING_TEXT_COLOR = "#00ddff";
    const COLUMN_INSERT_HINT_COLOR = "#b399ffca";
    const DROP_HIGHLIGHT_BG_COLOR = "#b399ffca";

    onMount(() => {
        init();
    });

    function init() {
        drawSongDataGrid();
    }

    function onResize(e: MouseEvent) {
        e.stopPropagation();
        e.preventDefault();
        drawSongDataGrid();
    }

    let contentHeight = HEADER_HEIGHT;
    let scrollableArea = contentHeight;

    // Trigger: on songs updated
    $: if (songs?.length) {
        console.log("SONGS UPDATED");
        drawSongDataGrid();
        scrollContainer?.scrollTo({
            top: 0
        });
        if (stage) {
            stage.on("click", function (e) {
                // e.target is a clicked Konva.Shape or current stage if you clicked on empty space
                console.log("clicked on", e.target);
                console.log(
                    "usual click on " +
                        JSON.stringify(stage.getPointerPosition())
                );
            });
        }
    }

    function drawSongDataGrid() {
        calculateCanvasSize();
        calculateColumns();
        calculateSongSlice();
        shouldRender = true;
        console.log("fields", fields);
        // drawHeaders();
        // drawRows();
    }

    function printInfo() {
        // console.log("fields", fields);
        // console.log("canvas size", width, virtualViewportHeight);
        // console.log(
        //     "widths add up to",
        //     displayFields.reduce((total, f) => (total += f.viewProps.width), 0)
        // );
    }

    function calculateCanvasSize() {
        contentHeight = HEADER_HEIGHT + songs?.length * ROW_HEIGHT;
        let area = contentHeight - viewportHeight;
        width = scrollContainer.clientWidth;
        // Set canvas size to fill the parent
        console.log("CANVS WIDTH", width);
        viewportHeight = container.getBoundingClientRect().height;
        virtualViewportHeight = viewportHeight;

        if (area < 0) {
            area = 0;
            virtualViewportHeight = viewportHeight;
        }

        scrollableArea = area;
        isScrollable = scrollableArea > 0;
        console.log("scrollable", isScrollable);

        setTimeout(() => {
            width = scrollContainer.clientWidth;
            calculateColumns();
        }, 50);
    }

    function calculateColumns() {
        let runningX = 0;
        let previousWidth = 0;

        // Calculate total width of fixed-width rectangles
        const fixedWidths = fields
            .filter((f) => !f.viewProps.autoWidth)
            .map((f) => f.viewProps.width);
        const totalFixedWidth = fixedWidths.reduce(
            (total, width) => total + width,
            0
        );
        // console.log("width", width, "totalFixedWidth", totalFixedWidth);
        // Calculate available width for 'auto' size rectangles
        const availableWidth = width - totalFixedWidth;
        // console.log("availableWidth", availableWidth);
        // Calculate the width for each 'auto' size rectangle
        const autoWidth =
            availableWidth / (displayFields.length - fixedWidths.length);

        displayFields = [
            ...displayFields.map((f) => {
                const rectWidth = f.viewProps.autoWidth
                    ? autoWidth
                    : f.viewProps.width;
                f.viewProps.x = runningX += previousWidth;
                f.viewProps.width = rectWidth;
                previousWidth = f.viewProps.width;
                return f;
            })
        ];
        printInfo();
    }

    let prevRemainder = 0; // To fix choppiness when jumping from eg. 18 to 1.

    function calculateSongSlice() {
        if (songs?.length) {
            // console.log(
            //     "scrollPos",
            //     scrollPos,
            //     "contentHeight",
            //     contentHeight,
            //     "viewport",
            //     viewportHeight
            // );

            // See how many rows fit in the current height
            let contentToViewportRatio = viewportHeight / contentHeight; // Multiply by this to get content sizes, divide to get viewport sizes
            const songsCountScrollable = Math.ceil(scrollableArea / ROW_HEIGHT);
            const songsCountPadding = Math.ceil(SCROLL_PADDING / ROW_HEIGHT);
            // console.log("pad", songsCountPadding);
            const songsCountViewport = Math.ceil(viewportHeight / ROW_HEIGHT);
            // Get px of current scroll position (content)
            let contentY = scrollableArea * scrollNormalized;
            sandwichTopHeight = contentY;
            sandwichBottomY = contentY + viewportHeight;
            sandwichBottomHeight =
                contentHeight - sandwichTopHeight - viewportHeight;

            // How much space to leave at the top and bottom
            let paddingTop = Math.min(contentY, SCROLL_PADDING / 2);
            // console.log('b', (contentHeight - contentY - viewportHeight));
            let paddingBottom = Math.min(
                contentHeight - contentY - viewportHeight,
                SCROLL_PADDING / 2
            );

            // console.log(
            //     "sandwich top",
            //     sandwichTopHeight,
            //     "bottom",
            //     sandwichBottomHeight
            // );

            let songsToPadTop = Math.floor(paddingTop / ROW_HEIGHT);
            let songsToPadBottom = Math.floor(paddingBottom / ROW_HEIGHT);
            let songsScrolled = contentY / ROW_HEIGHT;
            // console.log("songs at top", songsToPadTop);
            // console.log("songs at bottom", songsToPadBottom);
            let remainder = contentY % ROW_HEIGHT;
            prevRemainder = remainder;
            // console.log("remainder", remainder);
            // scrollOffset = -contentY / 20;

            // console.log("scrollNormalized", scrollNormalized);
            // console.log(
            //     "startSlice",
            //     scrollNormalized * (songsCountScrollable - songsToPadTop)
            // );
            songsStartSlice = Math.floor(
                Math.max(0, Math.floor(scrollNormalized * songsCountScrollable))
            );
            songsEndSlice = Math.min(
                songs.length,
                Math.ceil(songsStartSlice + songsCountViewport)
            );
            // console.log("start", songsStartSlice, "end", songsEndSlice);

            const getTopOffscreenRows = (
                firstOnScreenIdx,
                backwardsCount,
                prefix
            ) => {
                let offscreenRows = [];
                let firstSongIdx = Math.max(
                    0,
                    firstOnScreenIdx - backwardsCount
                );
                let lastSongIdx = Math.max(0, firstOnScreenIdx);
                if (firstOnScreenIdx >= 0 && lastSongIdx >= 0) {
                    offscreenRows.unshift(
                        ...songs
                            .slice(firstSongIdx, lastSongIdx)
                            .map((s) => ({ ...s, dummy: true }))
                    );
                }
                let dummyCount =
                    firstOnScreenIdx - backwardsCount < 0
                        ? Math.abs(firstOnScreenIdx - backwardsCount)
                        : 0;
                if (dummyCount) {
                    offscreenRows.unshift(
                        ...getDummyRows(dummyCount, "dummy-top")
                    );
                }
                return offscreenRows;
            };

            const getBottomOffscreenRows = (
                lastOnScreenIdx,
                forwardsCount,
                prefix
            ) => {
                let offscreenRows = [];
                let firstSongIdx = Math.min(songs.length - 1, lastOnScreenIdx);
                let lastSongIdx = Math.min(
                    songs.length - 1,
                    lastOnScreenIdx + forwardsCount
                );
                // console.log("first", firstSongIdx, "lastSongIdx", lastSongIdx);

                // console.log("songs length", songs.length);
                if (
                    firstSongIdx <= songs.length - 1 &&
                    lastSongIdx <= songs.length - 1
                ) {
                    offscreenRows.push(
                        ...songs
                            .slice(firstSongIdx, lastSongIdx)
                            .map((s) => ({ ...s, dummy: true }))
                    );
                }
                let dummyRemainder =
                    songs.length - (lastOnScreenIdx + forwardsCount);
                let dummyCount =
                    dummyRemainder < 0 ? Math.abs(dummyRemainder) : 0;
                if (dummyCount) {
                    offscreenRows.push(
                        ...getDummyRows(dummyCount, "dummy-bottom")
                    );
                }
                // console.log("bottom", offscreenRows);

                return offscreenRows;
            };

            const getDummyRows = (count, prefix) => {
                return new Array(count)
                    .fill({ title: "dummy" })
                    .map((s, idx) => ({
                        ...s,
                        id: `${prefix}-${idx}`,
                        dummy: true
                    }));
            };

            // Make sure the window is always filled with the right amount of rows
            songsSlice = songs.slice(songsStartSlice, songsEndSlice);
            // console.log("slice", songsStartSlice, songsEndSlice);
            let diff = songsCountViewport - (songsSlice.length - 1);
            // console.log("diff", diff);
            if (diff) {
                songsSlice = songsSlice.concat(
                    getDummyRows(diff, "dummy-middle")
                );
            }
            // console.log("slice", songsSlice);
            // Top and bottom dummies
            songsSlice = getTopOffscreenRows(
                songsStartSlice,
                DUMMY_COUNT,
                "dummy-top"
            )
                .concat(...songsSlice)
                .concat(
                    ...getBottomOffscreenRows(
                        songsEndSlice,
                        DUMMY_COUNT,
                        "dummy-bottom"
                    )
                );
            // console.log(songsSlice.length);
            prevScrollPos = scrollPos;
        }
    }

    function onScroll() {
        scrollPos = scrollContainer.scrollTop;
        scrollNormalized = scrollPos / (contentHeight - viewportHeight);

        if (scrollContainer && stage) {
            calculateSongSlice();

            currentSongInView =
                $currentSongIdx >= songsStartSlice &&
                $currentSongIdx <= songsEndSlice;

            var dy = scrollPos;
            // stage.container().style.transform = "translateY(" + -dy + "px)";
            // requestAnimationFrame(() => stage.y(-dy));
        }
    }

    let currentSongY = 0;
    $: if ($currentSongIdx) {
        currentSongY = $currentSongIdx * ROW_HEIGHT;

        currentSongInView =
            $currentSongIdx >= songsStartSlice &&
            $currentSongIdx <= songsEndSlice;

        // console.log("currentSongY", currentSongY);
    }

    function scrollToCurrentSong() {
        console.log("y", currentSongY);
        let adjustedPos = currentSongY;
        if (currentSongY > viewportHeight / 2.3) {
            adjustedPos -= viewportHeight / 2.3;
        }
        if ($isPlaying && $currentSong) {
            scrollContainer.scrollTo({
                top: adjustedPos,
                behavior: "smooth"
            });
        }
    }

    // LIBRARY FUNCTIONALITY

    let isCmdOrCtrlPressed = false;
    let isShiftPressed = false;
    let rangeStartSongIdx = null;
    let rangeEndSongIdx = null;
    let highlightedSongIdx = 0;
    let showTrackMenu = false;
    let menuPos;
    let currentSongInView = false;

    function onDoubleClickSong(song, idx) {
        $currentSongIdx = idx;
        $playlist = $queriedSongs;
        $playlistIsAlbum = false;
        AudioPlayer.playSong(song);
    }

    function onRightClick(e, song, idx) {
        if (!songsHighlighted.includes(song)) {
            highlightSong(song, idx, false);
        }

        console.log("songIdsHighlighted", songsHighlighted);
        if (songsHighlighted.length > 1) {
            $rightClickedTracks = songsHighlighted;
            $rightClickedTrack = null;
        } else {
            $rightClickedTrack = song;
        }
        showTrackMenu = true;
        menuPos = { x: e.clientX, y: e.clientY };
    }

    $: {
        if (songs?.length && $query.query?.length && !$isTrackInfoPopupOpen) {
            highlightSong(songs[0], 0, false, true);
        }
    }
    export let songsHighlighted: Song[] = [];
    export let onSongsHighlighted = null;

    function isSongHighlighted(song: Song) {
        return songsHighlighted.map((s) => s?.id).includes(song?.id);
    }

    function toggleHighlight(song, idx, isKeyboardArrows = false) {
        if (!song) song = songs[0];
        highlightedSongIdx = idx;
        if (isSongHighlighted(song)) {
            if (songsHighlighted.length) {
                songsHighlighted = [];
                highlightSong(song, idx, isKeyboardArrows);
            } else {
                unhighlightSong(song);
            }
        } else {
            highlightSong(song, idx, isKeyboardArrows);
        }
    }

    function highlightSong(
        song: Song,
        idx,
        isKeyboardArrows: boolean,
        isDefault = false
    ) {
        console.log("highlighted", song, idx);
        if (!isKeyboardArrows && isShiftPressed) {
            if (rangeStartSongIdx === null) {
                rangeStartSongIdx = idx;
            } else {
                rangeEndSongIdx = idx;
                // Highlight all the songs in between

                if (rangeEndSongIdx < rangeStartSongIdx) {
                    let startIdx = rangeStartSongIdx;
                    rangeStartSongIdx = rangeEndSongIdx;
                    rangeEndSongIdx = startIdx;
                }
                songsHighlighted = songs.slice(
                    rangeStartSongIdx,
                    rangeEndSongIdx + 1
                );
                rangeStartSongIdx = null;
                rangeEndSongIdx = null;
                $rightClickedTrack = null;
                console.log("highlighted2", songsHighlighted);
            }
        } else if (
            (isKeyboardArrows && isShiftPressed) ||
            isCmdOrCtrlPressed ||
            hotkeys.isPressed(91)
        ) {
            songsHighlighted.push(song);
            rangeStartSongIdx = idx;
            $rightClickedTrack = null;
        } else {
            // Highlight single song, via a good old click
            if (!isDefault) {
                $shouldFocusFind = { target: "search", action: "unfocus" };
            }
            songsHighlighted = [song];
            highlightedSongIdx = idx;
            $rightClickedTracks = [];
            $rightClickedTracks = $rightClickedTracks;
            rangeStartSongIdx = idx;

            // Extra - if the Info overlay is shown, use the arrows to replace the track shown in the overlay
            if ($isTrackInfoPopupOpen && isKeyboardArrows) {
                $rightClickedTrack = song;
            }
        }
        // console.log("start", rangeStartSongIdx);

        songsHighlighted = songsHighlighted;
        onSongsHighlighted && onSongsHighlighted(songsHighlighted);
    }

    function unhighlightSong(song: Song) {
        songsHighlighted.splice(songsHighlighted.indexOf(song), 1);
        songsHighlighted = songsHighlighted;
        onSongsHighlighted && onSongsHighlighted(songsHighlighted);
    }

    function onSongDragStart(song: Song) {
        console.log("dragstart", song);
        console.log("songshighlighted", songsHighlighted);
        if (songsHighlighted.length > 1) {
            $draggedSongs = songsHighlighted;
        } else {
            $draggedSongs = [song];
        }
    }

    // Shortcuts

    if ($os === "Darwin") {
        hotkeys("cmd", function (event, handler) {
            isCmdOrCtrlPressed = true;
        });
    } else if ($os === "Windows_NT" || $os === "Linux") {
        hotkeys("ctrl", function (event, handler) {
            isCmdOrCtrlPressed = true;
        });
    }

    hotkeys("esc", function (event, handler) {
        if ($isSmartQueryBuilderOpen) {
            $isSmartQueryBuilderOpen = false;
        }
    });

    function onKeyDown(event) {
        if (event.keyCode === 16) {
            isShiftPressed = true;
            console.log("shift pressed");
        } else if (
            event.keyCode === 38 &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            event.preventDefault();
            // up
            if (highlightedSongIdx > 0) {
                toggleHighlight(
                    songs[highlightedSongIdx - 1],
                    highlightedSongIdx - 1,
                    true
                );
            }
        } else if (
            event.keyCode === 40 &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // down
            event.preventDefault();
            if (highlightedSongIdx < songs.length) {
                toggleHighlight(
                    songs[highlightedSongIdx + 1],
                    highlightedSongIdx + 1,
                    true
                );
            }
        } else if (
            event.keyCode === 73 &&
            !$isTrackInfoPopupOpen &&
            $singleKeyShortcutsEnabled &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // 'i' for info popup
            event.preventDefault();
            console.log("active element", document.activeElement.tagName);
            // Check if there an input in focus currently
            if (!$isTrackInfoPopupOpen && songsHighlighted.length) {
                console.log("opening info", songsHighlighted);
                if (songsHighlighted.length > 1) {
                    $rightClickedTracks = songsHighlighted;
                } else {
                    $rightClickedTrack = songsHighlighted[0];
                }
                $isTrackInfoPopupOpen = true;
            }
        } else if (
            event.keyCode === 13 &&
            !$isTrackInfoPopupOpen &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // 'Enter' to play highlighted track
            event.preventDefault();
            if (!$isTrackInfoPopupOpen) {
                $playlist = $queriedSongs;
                $playlistIsAlbum = false;
                $currentSongIdx = highlightedSongIdx;
                AudioPlayer.playSong(songsHighlighted[0]);
            }
        }
    }
    function onKeyUp(event) {
        if (event.keyCode === 16) {
            isShiftPressed = false;
            console.log("shift lifted");
        }
    }

    addEventListener("keydown", onKeyDown);
    addEventListener("keyup", onKeyUp);

    onDestroy(() => {
        hotkeys.unbind("ctrl");
        hotkeys.unbind("cmd");
        hotkeys.unbind("esc");
        removeEventListener("keydown", onKeyDown);
        removeEventListener("keyup", onKeyUp);
    });

    // COLUMNS

    function updateOrderBy(newOrderBy) {
        if (newOrderBy === "trackNumber") return; // Not supported
        if ($query.orderBy === newOrderBy) {
            $query.reverse = !$query.reverse;
        }
        $query.orderBy = newOrderBy;
        $query = $query;
    }

    // Re-order columns

    $: {
        displayFields && calculateColumns();
    }

    let dropColumnIdx = null;

    function handleColumnDrag(
        pos: { x: number; y: number },
        index
    ): { x: number; y: number } {
        console.log("over", pos);
        // const headerColumn = document.querySelector(`[data-index='${index}']`);
        // const elementRect = headerColumn.getBoundingClientRect();
        // const elementWidth = elementRect.width;
        // const dragZoneWidth = 6; // 5% of the element's width

        return { x: pos.x, y: 0 };
    }

    function onDragStart(event: KonvaDragTransformEvent, index) {
        event.detail.target.moveToBottom();
        $draggedColumnIdx = index;
    }

    function onDragEnd(event: KonvaDragTransformEvent, index) {
        console.log("event", event);
        if (columnToInsertIdx !== null) {
            insertColumn($draggedColumnIdx, columnToInsertIdx);
        } else {
            swapColumns($draggedColumnIdx, dropColumnIdx);
        }
        $draggedColumnIdx = null;
        dropColumnIdx = null;
        event.detail.target.position({ x: 0, y: 0 });
    }

    function onDragMove(event: KonvaDragTransformEvent) {
        let x = event.detail.evt.offsetX;
        const index = displayFields.findIndex(
            (f) => x >= f.viewProps.x && x <= f.viewProps.x + f.viewProps.width
        );

        dropColumnIdx = index;
        const column = displayFields[index];
        const elementWidth = column.viewProps.width;
        let offsetX = x - column.viewProps.x;
        const dragZoneWidth = 6; // 5% of the element's width
        if (
            index !== $draggedColumnIdx &&
            index !== $draggedColumnIdx + 1 &&
            offsetX < dragZoneWidth
        ) {
            // User is dragging over the sides (5% of width) of the element
            columnToInsertIdx = index;
            columnToInsertXPos = column.viewProps.x;
            // Add your logic here
        } else if (
            index !== $draggedColumnIdx - 1 &&
            index !== $draggedColumnIdx &&
            index < displayFields.length - 1 &&
            offsetX > elementWidth - dragZoneWidth
        ) {
            columnToInsertIdx = index + 1;
            columnToInsertXPos = column.viewProps.x + column.viewProps.width;
        } else {
            columnToInsertIdx = null;
            columnToInsertXPos = 0;
        }
        console.log("columnToInsertIdx", columnToInsertIdx);
        console.log("dropidx", dropColumnIdx);
    }

    function resetColumnOrderUi() {
        $fileDropHandler = null;
        $emptyDropEvent = null;
        dropColumnIdx = null;
        $draggedColumnIdx = null;
        columnToInsertIdx = null;
        columnToInsertXPos = 0;
    }
    function insertColumn(oldIndex, newIndex) {
        displayFields = moveArrayElement(displayFields, oldIndex, newIndex);
        resetColumnOrderUi();
    }

    function swapColumns(oldIndex, newIndex) {
        console.log(oldIndex, newIndex);
        displayFields = swapArrayElements(displayFields, oldIndex, newIndex);
        resetColumnOrderUi();
    }

    // Sets back to default
    function resetColumnOrder() {
        fields = DEFAULT_FIELDS;
    }
</script>

<svelte:window on:resize={debounce(onResize, 5)} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-click-events-have-key-events -->

<TrackMenu bind:showMenu={showTrackMenu} bind:pos={menuPos} />
<ColumnPicker
    bind:showMenu={showColumnPicker}
    bind:pos={columnPickerPos}
    bind:fields
    onResetOrder={resetColumnOrder}
    {isOrderChanged}
/>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if $isPlaying && $currentSong && !currentSongInView}
    <div
        in:fly={{ duration: 150, y: 30 }}
        out:fly={{ duration: 150, y: 30 }}
        class="scroll-now-playing"
        on:click={scrollToCurrentSong}
    >
        <div class="eq">
            <span class="eq1" />
            <span class="eq2" />
            <span class="eq3" />
        </div>
        <p>Scroll to Now playing</p>
    </div>
{/if}

<div
    id="scroll-container"
    style="overflow-y: {isScrollable ? 'visible' : 'hidden'}"
    on:scroll={onScroll}
    bind:this={scrollContainer}
>
    <div
        id="large-container"
        style="height: {scrollableArea}px;max-height: {scrollableArea}px;"
    />

    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="container" bind:this={container} on:contextmenu|preventDefault>
        {#if shouldRender}
            <Stage
                config={{
                    width,
                    height: virtualViewportHeight,
                    y: -sandwichTopHeight
                }}
                bind:handle={stage}
            >
                <Layer>
                    <Rect
                        config={{
                            x: 0,
                            y: 0,
                            width,
                            height: sandwichTopHeight,
                            fill: OFFSCREEN_BG_COLOR
                        }}
                    />
                    {#if columnToInsertIdx !== null}
                        <Rect
                            config={{
                                x: columnToInsertXPos,
                                y: sandwichTopHeight + HEADER_HEIGHT,
                                height: viewportHeight,
                                width: 2,
                                fill: COLUMN_INSERT_HINT_COLOR
                            }}
                        />
                    {/if}
                    {#each displayFields as f, idx (f.value)}
                        <Group
                            config={{
                                x: f.viewProps.x,
                                y: sandwichTopHeight,
                                width: f.viewProps.width,
                                draggable: true,
                                dragBoundFunc(pos) {
                                    return handleColumnDrag(pos, idx);
                                }
                            }}
                            on:click={(ev) => {
                                if (ev.detail.evt.button === 0)
                                    updateOrderBy(f.value);
                                else if (ev.detail.evt.button === 2) {
                                    console.log("ev", ev);
                                    columnPickerPos = {
                                        x: ev.detail.evt.offsetX,
                                        y: 15
                                    };
                                    showColumnPicker = !showColumnPicker;
                                }
                            }}
                            on:mouseenter={() => {
                                hoveredColumnIdx = idx;
                            }}
                            on:mouseleave={() => {
                                hoveredColumnIdx = null;
                            }}
                            on:dragmove={(ev) => {
                                onDragMove(ev);
                            }}
                            on:dragstart={(ev) => {
                                onDragStart(ev, idx);
                            }}
                            on:dragend={(ev) => {
                                onDragEnd(ev, idx);
                            }}
                        >
                            <Rect
                                config={{
                                    width:
                                        idx === displayFields.length - 1
                                            ? f.viewProps.width
                                            : f.viewProps.width - 0.5,
                                    height: HEADER_HEIGHT,
                                    fill:
                                        columnToInsertIdx === null &&
                                        dropColumnIdx === idx &&
                                        dropColumnIdx !== $draggedColumnIdx
                                            ? DROP_HIGHLIGHT_BG_COLOR
                                            : hoveredColumnIdx === idx ||
                                                $query.orderBy === f.value
                                              ? HEADER_BG_COLOR_HOVERED
                                              : HEADER_BG_COLOR
                                }}
                            />
                            {#if hoveredColumnIdx === idx}
                                <Path
                                    config={{
                                        x: -2,
                                        y: 6,
                                        scaleX: 0.9,
                                        scaleY: 0.9,
                                        data: "M7.375 3.67c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m0 8.66c0-.646-.56-1.17-1.25-1.17s-1.25.524-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17m-1.25-5.5c.69 0 1.25.525 1.25 1.17c0 .645-.56 1.17-1.25 1.17S4.875 8.645 4.875 8c0-.645.56-1.17 1.25-1.17m5-3.16c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m-1.25 7.49c.69 0 1.25.524 1.25 1.17c0 .645-.56 1.17-1.25 1.17s-1.25-.525-1.25-1.17c0-.646.56-1.17 1.25-1.17M11.125 8c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17",
                                        fill: "rgba(255, 255, 255, 0.5)"
                                    }}
                                />
                            {/if}
                            {#if $query.orderBy === f.value}
                                {#if $query.reverse}
                                    <Path
                                        config={{
                                            x: f.viewProps.width - 16,
                                            y: 6,
                                            scaleX: 0.6,
                                            scaleY: 0.6,
                                            data: "m7.293 8.293l3.995-4a1 1 0 0 1 1.32-.084l.094.083l4.006 4a1 1 0 0 1-1.32 1.499l-.094-.083l-2.293-2.291v11.584a1 1 0 0 1-.883.993L12 20a1 1 0 0 1-.993-.884L11 19.001V7.41L8.707 9.707a1 1 0 0 1-1.32.084l-.094-.084a1 1 0 0 1-.084-1.32zl3.995-4z",
                                            fill: "rgba(255, 255, 255, 0.8)"
                                        }}
                                    />
                                {:else}
                                    <Path
                                        config={{
                                            x: f.viewProps.width - 16,
                                            y: 6,
                                            scaleX: 0.6,
                                            scaleY: 0.6,
                                            data: "M11.883 4.01L12 4.005a1 1 0 0 1 .993.883l.007.117v11.584l2.293-2.294a1 1 0 0 1 1.32-.084l.094.083a1 1 0 0 1 .084 1.32l-.084.095l-3.996 4a1 1 0 0 1-1.32.083l-.094-.083l-4.004-4a1 1 0 0 1 1.32-1.498l.094.083L11 16.583V5.004a1 1 0 0 1 .883-.992L12 4.004z",
                                            fill: "rgba(255, 255, 255, 0.8)"
                                        }}
                                    />
                                {/if}
                            {/if}
                            <Text
                                config={{
                                    text: f.name,
                                    align: "left",
                                    padding:
                                        f.value.match(/^(track|duration)/) !==
                                        null
                                            ? 8
                                            : 10,
                                    height: HEADER_HEIGHT,
                                    fontSize: 14,
                                    letterSpacing:
                                        f.value.match(/^(duration)/) !== null
                                            ? -1
                                            : 0,
                                    fontStyle: "bold",
                                    verticalAlign: "middle",
                                    fontFamily:
                                        "-apple-system, Avenir, Helvetica, Arial, sans-serif",
                                    fill: TEXT_COLOR
                                }}
                            />
                        </Group>
                        {#if idx > 0}
                            <Rect
                                config={{
                                    x: f.viewProps.x - 1,
                                    y: sandwichTopHeight + HEADER_HEIGHT,
                                    height: viewportHeight,
                                    width: 0.5,
                                    fill: "rgba(242, 242, 242, 0.144)"
                                }}
                            />
                        {/if}
                    {/each}
                    {#if songsSlice?.length}
                        {#each songsSlice as song, songIdx (song.viewModel?.viewId ?? song.id)}
                            <Group
                                on:dblclick={() =>
                                    onDoubleClickSong(song, songIdx)}
                                on:click={(e) => {
                                    console.log("e", e);
                                    if (e.detail.evt.button === 0) {
                                        toggleHighlight(
                                            song,
                                            song.viewModel.index
                                        );
                                    } else if (e.detail.evt.button === 2) {
                                        onRightClick(
                                            e.detail.evt,
                                            song,
                                            song.viewModel.index
                                        );
                                    }
                                }}
                                on:mouseenter={() => {
                                    hoveredSongIdx = songIdx;
                                }}
                                on:mouseleave={() => {
                                    hoveredSongIdx = null;
                                }}
                                on:mousedown={(e) =>
                                    e.detail.evt.button === 0 &&
                                    onSongDragStart(song)}
                                config={{
                                    visible: !song.dummy
                                }}
                            >
                                <Rect
                                    config={{
                                        x: 0,
                                        y:
                                            sandwichTopHeight +
                                            HEADER_HEIGHT +
                                            ROW_HEIGHT * songIdx +
                                            -DUMMY_PADDING +
                                            scrollOffset,
                                        width: width,
                                        height: ROW_HEIGHT,
                                        fill:
                                            $currentSong?.id === song?.id
                                                ? PLAYING_BG_COLOR
                                                : songsHighlighted &&
                                                    isSongHighlighted(song)
                                                  ? HIGHLIGHT_BG_COLOR
                                                  : hoveredSongIdx === songIdx
                                                    ? ROW_BG_COLOR_HOVERED
                                                    : ROW_BG_COLOR
                                    }}
                                />
                                {#each displayFields as f, idx (f.value)}
                                    <Text
                                        config={{
                                            x:
                                                f.value.match(
                                                    /^(title|artist|album|track)/
                                                ) !== null
                                                    ? f.viewProps.x + 10
                                                    : f.viewProps.x,
                                            y:
                                                sandwichTopHeight +
                                                HEADER_HEIGHT +
                                                ROW_HEIGHT * songIdx +
                                                -DUMMY_PADDING +
                                                scrollOffset,
                                            text: song[f.value],
                                            align:
                                                f.value.match(
                                                    /^(title|artist|album|track)/
                                                ) !== null
                                                    ? "left"
                                                    : "center",
                                            width:
                                                idx === displayFields.length - 1
                                                    ? f.viewProps.width - 10
                                                    : f.value.match(
                                                            /^(title|artist|album|track)/
                                                        ) !== null
                                                      ? f.viewProps.width - 12
                                                      : f.viewProps.width,
                                            padding:
                                                f.value.match(/^(genre)/) !==
                                                null
                                                    ? 10
                                                    : 2,
                                            height: HEADER_HEIGHT,
                                            fontSize: 13.5,
                                            verticalAlign: "middle",
                                            fill:
                                                $currentSong?.id === song.id
                                                    ? PLAYING_TEXT_COLOR
                                                    : TEXT_COLOR,
                                            ellipsis:
                                                f.value.match(
                                                    /^(title|artist|album|genre)/
                                                ) !== null
                                        }}
                                    />
                                {/each}
                            </Group>
                        {/each}
                        <Rect
                            config={{
                                x: 0,
                                y: sandwichBottomY,
                                width,
                                height: sandwichBottomHeight,
                                fill: OFFSCREEN_BG_COLOR
                            }}
                        />
                    {/if}
                </Layer>
            </Stage>
        {/if}
    </div>
</div>

<style lang="scss">
    .container {
        width: 100%;
        height: 100vh;
        pointer-events: all;
        display: flex;
        position: sticky;
        top: 0;
        bottom: 0;
        user-select: none;
        background-color: rgba(36, 33, 34, 0.688);
    }

    #large-container {
        width: 100%;
        overflow: hidden;
    }

    #scroll-container {
        width: 100%;
        height: 100vh;
        position: absolute;
        overflow-y: auto;
        overflow-x: hidden;
        z-index: 10;
        pointer-events: auto;
    }

    .scroll-now-playing {
        position: absolute;
        bottom: 3em;
        left: 0;
        right: 0;
        padding: 0.5em 1em;
        border-radius: 10px;
        background-color: #1b1b1c;
        border: 1px solid rgb(58, 56, 56);
        box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.834);
        color: white;
        margin: auto;
        width: fit-content;
        z-index: 11;
        display: inline-flex;
        align-items: center;
        gap: 10px;
        cursor: default;
        user-select: none;

        @media only screen and (max-width: 522px) {
            display: none;
        }
        &:hover {
            background-color: #1f1f21;
            border: 1px solid rgb(101, 98, 98);
            box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.934);
        }
        &:active {
            background-color: #2a2a2d;
            border: 2px solid rgb(101, 98, 98);
            box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.934);
        }

        .eq {
            width: 15px;
            padding: 0.5em;
            position: relative;

            span {
                display: inline-block;
                width: 3px;
                background-color: #ddd;
                position: absolute;
                bottom: 0;
            }

            .eq1 {
                height: 13px;
                left: 0;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0s;
            }

            .eq2 {
                height: 15px;
                left: 6px;
                animation-name: talleq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.17s;
            }

            .eq3 {
                height: 13px;
                left: 12px;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.34s;
            }
        }
        p {
            margin: 0;
        }
    }

    @keyframes shorteq {
        0% {
            height: 10px;
        }
        50% {
            height: 5px;
        }
        100% {
            height: 10px;
        }
    }
    @keyframes talleq {
        0% {
            height: 15px;
        }
        50% {
            height: 8px;
        }
        100% {
            height: 15px;
        }
    }
</style>
