<!-- SongDataGrid.svelte -->

<script lang="ts">
    import { onMount } from "svelte";
    import {
        Group,
        Layer,
        Path,
        Rect,
        Stage,
        Text,
        Label,
        Tag,
        type KonvaDragTransformEvent,
    } from "svelte-konva";

    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import { debounce } from "lodash-es";
    import type { Song } from "src/App";
    import { onDestroy } from "svelte";
    import { cubicInOut } from "svelte/easing";
    import { fade, fly } from "svelte/transition";
    import { db } from "../../data/db";

    import {
        arrowFocus,
        columnOrder,
        current,
        draggedAlbum,
        draggedSongs,
        forceRefreshLibrary,
        isDraggingFromQueue,
        isPlaying,
        isShuffleEnabled,
        isSidebarOpen,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        popupOpen,
        libraryScrollPos,
        os,
        queue,
        queriedSongs,
        query,
        rightClickedTrack,
        rightClickedTracks,
        shouldFocusFind,
        shuffledQueue,
        singleKeyShortcutsEnabled,
        smartQuery,
        smartQueryInitiator,
        uiView,
    } from "../../data/store";
    import { moveArrayElement } from "../../utils/ArrayUtils";
    import AudioPlayer from "../player/AudioPlayer";
    import SmartQueryBuilder from "../smart-query/SmartQueryBuilder.svelte";
    import SmartQueryMainHeader from "../smart-query/SmartQueryMainHeader.svelte";
    import SmartQueryResultsPlaceholder from "../smart-query/SmartQueryResultsPlaceholder.svelte";
    import ColumnPicker from "../library/ColumnPicker.svelte";
    import ImportPlaceholder from "../library/ImportPlaceholder.svelte";
    import { getQueryPart } from "../smart-query/QueryParts";
    import { UserQueryPart } from "../smart-query/UserQueryPart";
    import Konva from "konva";
    import audioPlayer from "../player/AudioPlayer";
    import TrackMenu from "../queue/TrackMenu.svelte";
    import { currentThemeObject } from "../../theming/store";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import {
        findQueueIndex,
        findQueueIndexes,
        setQueue,
        updateQueues,
    } from "../../data/storeHelper";
    import QueueMenu from "../queue/QueueMenu.svelte";

    export let allSongs = null;
    export let dim = false;
    export let isLoading = false;
    export let isInit = true;

    const WINDOW_CONTROLS_WIDTH = 70;

    $: songs = ($isShuffleEnabled ? $shuffledQueue : $queue).map((s, idx) => ({
        ...s,
        viewModel: {
            index: idx,
            viewId: idx.toString(),
        },
    }));

    const DEFAULT_FIELDS = [
        {
            name: "Title",
            value: "title",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
    ];

    export let fields = DEFAULT_FIELDS;

    let currentSongY = 0;
    let currentSongInView = false;
    let currentSongScrollIdx = null;

    let hoveredColumnIdx = null;
    let hoveredSongIdx = null; // Index is slice-specific
    let columnToInsertIdx = null;
    let columnToInsertXPos = 0;
    let hoveredField = null;
    let isDraggingOver = false;
    let isOver = false;

    $: isOrderChanged =
        JSON.stringify($columnOrder) !==
        JSON.stringify(DEFAULT_FIELDS.map((f) => f.value));

    let showColumnPicker = false;
    let columnPickerPos;

    $: displayFields = fields.filter((f) => f.show);
    $: numColumns = fields.filter((f) => f.show).length;

    let songsSlice: Song[];
    let songsStartSlice = 0;
    let songsEndSlice = 0;
    let canvas;

    let shouldRender = false;
    let ready = false; // When scroll position is restored (if available), to avoid jump
    let libraryContainer: HTMLDivElement;
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
    const HEADER_HEIGHT = 22;
    const ROW_HEIGHT = 26;
    const DROP_HINT_HEIGHT = 2;
    const BORDER_WIDTH = 1;
    const SCROLL_PADDING = 200;
    const DUMMY_COUNT = 5;
    const DUMMY_PADDING = DUMMY_COUNT * ROW_HEIGHT;

    // COLORS
    let BG_COLOR: string;
    let HEADER_BG_COLOR: string;
    let OFFSCREEN_BG_COLOR: string;
    let HEADER_BG_COLOR_HOVERED: string;
    let TEXT_COLOR: string;
    let HIGHLIGHT_BG_COLOR: string;
    let ROW_BG_COLOR: string;
    let ROW_BG_COLOR_HOVERED: string;
    let PLAYING_BG_COLOR: string;
    let PLAYING_TEXT_COLOR: string;
    let COLUMN_INSERT_HINT_COLOR: string;
    let DROP_HIGHLIGHT_BG_COLOR: string;
    let CLICKABLE_CELL_BG_COLOR: string;
    let CLICKABLE_CELL_BG_COLOR_HOVERED: string;
    let DRAGGING_SOURCE_COLOR: string;

    onMount(() => {
        init();

        // Check for retina screens
        var query =
            "(-webkit-min-device-pixel-ratio: 2), (min-device-pixel-ratio: 2), (min-resolution: 192dpi)";

        if (matchMedia(query).matches) {
            // Slightly reduce the ratio for better performance
            Konva.pixelRatio = 1.8;
        }

        const resizeObserver = new ResizeObserver((entries) => {
            // We're only watching one element
            const entry = entries.at(0);

            //Get the block size
            drawSongDataGrid();
        });

        resizeObserver.observe(libraryContainer);

        // This callback cleans up the observer
        return () => resizeObserver.unobserve(libraryContainer);
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

    let prevSongCount = 0;

    $: counts = liveQuery(() => {
        return db.transaction("r", db.songs, async () => {
            const artists = await (
                await db.songs.orderBy("artist").uniqueKeys()
            ).length;
            const albums = await (
                await db.songs.orderBy("album").uniqueKeys()
            ).length;
            const songs = await $allSongs.length;
            return { songs, artists, albums };
        });
    });

    $: noSongs = !songs || songs.length === 0;

    // Trigger: on songs updated
    $: if (songs !== undefined && libraryContainer !== undefined) {
        console.log("Queue::queue updated", songs.length);
        drawSongDataGrid();
        prevSongCount = songs.length;
    }

    $: if ($currentThemeObject) {
        // COLORS
        BG_COLOR = $currentThemeObject["panel-background"];
        HEADER_BG_COLOR = $currentThemeObject["library-header-bg"];
        OFFSCREEN_BG_COLOR = "#71658e3b";
        HEADER_BG_COLOR_HOVERED =
            $currentThemeObject["library-header-active-bg"];
        TEXT_COLOR = $currentThemeObject["library-text-color"];
        HIGHLIGHT_BG_COLOR = $currentThemeObject["library-highlight-bg"];
        ROW_BG_COLOR = "transparent";
        ROW_BG_COLOR_HOVERED = $currentThemeObject["library-hover-bg"];
        PLAYING_BG_COLOR = $currentThemeObject["library-playing-bg"];
        PLAYING_TEXT_COLOR = $currentThemeObject["library-playing-text"];
        COLUMN_INSERT_HINT_COLOR = "#b399ffca";
        DROP_HIGHLIGHT_BG_COLOR = "#b399ffca";
        CLICKABLE_CELL_BG_COLOR =
            $currentThemeObject["library-clickable-cell-bg"];
        CLICKABLE_CELL_BG_COLOR_HOVERED =
            $currentThemeObject["library-clickable-cell-hover-bg"];
        DRAGGING_SOURCE_COLOR = "#8a69683e";
    }

    // Restore scroll position if any
    $: if (
        $uiView.match(/(library|albums)/) &&
        (isInit || $forceRefreshLibrary === true) &&
        $libraryScrollPos !== null &&
        scrollContainer &&
        shouldRender
    ) {
        // console.log(
        //     "scrollpos",
        //     $libraryScrollPos,
        //     (songs?.length * ROW_HEIGHT - viewportHeight) * $libraryScrollPos
        // );
        setTimeout(() => {
            scrollContainer.scrollTo({
                top: 0,
                behavior: "instant",
            });
            ready = true;
        }, 50);
        isInit = false;
        $forceRefreshLibrary = false;
    } else if ($uiView.match(/^(playlists|map)/)) {
        scrollContainer?.scrollTo({
            top: 0,
        });
        ready = true;
    } else if ($uiView.match(/^(smart-query|favourites)/)) {
        scrollContainer?.scrollTo({
            top: 0,
        });
        ready = true;
    }

    $: if ($current.index !== null) {
        let idx = $current.index;
        currentSongY = idx * ROW_HEIGHT;
        currentSongInView = idx >= songsStartSlice && idx <= songsEndSlice;
    }

    function drawSongDataGrid() {
        calculateCanvasSize();
        calculateColumns();
        calculateSongSlice();
        shouldRender = true;
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
        // console.log('scrollContainer.clientWidth', scrollContainer?.offsetWidth);
        width = scrollContainer?.clientWidth ?? libraryContainer.clientWidth;
        // Set canvas size to fill the parent
        viewportHeight = libraryContainer.getBoundingClientRect().height;
        virtualViewportHeight = viewportHeight;

        if (area < 0) {
            area = 0;
            virtualViewportHeight = viewportHeight;
        }

        scrollableArea = area;
        isScrollable = scrollableArea > 0;
        // console.log("scrollableArea", scrollableArea);

        setTimeout(() => {
            width =
                scrollContainer?.clientWidth ?? libraryContainer.clientWidth;
            calculateColumns();
        }, 50);
    }

    function calculateColumns() {
        let runningX = 0;
        let previousWidth = 0;
        const sortedFields = $columnOrder
            .map((c) => fields.find((f) => f.value === c))
            .filter(Boolean);

        // Fields visible depending on window width
        const visibleFields = sortedFields.filter((f) => {
            switch (f?.value) {
                case "duration":
                    return width > 800;
                case "genre":
                    return width > 700;
                case "year":
                    return width > 650;
                case "trackNumber":
                    return width > 500;
                case "artist":
                    return width > 300;
                case "album":
                    return width > 450;
                case "albumArtist":
                    return width > 450;
                case "composer":
                    return width > 650;
                case "originCountry":
                    return width > 650;
                default:
                    return true;
            }
        });
        // console.log(visibleFields);

        // Calculate total width of fixed-width rectangles
        const fixedWidths = visibleFields
            .filter((f) => !f.viewProps.autoWidth)
            .map((f) => f.viewProps.width);
        const totalFixedWidth = fixedWidths.reduce(
            (total, width) => total + width,
            0,
        );
        // console.log("width", width, "totalFixedWidth", totalFixedWidth);
        // Calculate available width for 'auto' size rectangles
        const availableWidth = width - totalFixedWidth;
        // console.log("availableWidth", availableWidth);
        // Calculate the width for each 'auto' size rectangle
        const autoWidth =
            availableWidth / (visibleFields.length - fixedWidths.length);

        // Final display fields
        displayFields = [
            ...visibleFields.map((f) => {
                const rectWidth = f.viewProps.autoWidth
                    ? autoWidth
                    : f.viewProps.width;
                f.viewProps.x = runningX += previousWidth;
                f.viewProps.width = rectWidth;
                previousWidth = f.viewProps.width;
                return f;
            }),
        ];
        printInfo();
    }

    let prevRemainder = 0; // To fix choppiness when jumping from eg. 18 to 1.

    function calculateSongSlice() {
        if (songs.length === 0) {
            songsSlice = [];
            songsStartSlice = 0;
            songsEndSlice = 0;
        } else if (songs?.length) {
            // console.log(
            //     "scrollPos",
            //     scrollPos,
            //     "contentHeight",
            //     contentHeight,
            //     "viewport",
            //     viewportHeight
            // );

            // See how many rows fit in the current height
            const songsCountScrollable = Math.ceil(scrollableArea / ROW_HEIGHT);
            // console.log("pad", songsCountPadding);
            const songsCountViewport = Math.ceil(viewportHeight / ROW_HEIGHT);
            // Get px of current scroll position (content)
            let contentY = scrollableArea * scrollNormalized;
            sandwichTopHeight = contentY;
            sandwichBottomY = contentY + viewportHeight;
            sandwichBottomHeight =
                contentHeight - sandwichTopHeight - viewportHeight;
            // console.log(
            //     "sandwichTop",
            //     sandwichTopHeight,
            //     "sandwichBottom",
            //     sandwichBottomY,
            //     sandwichBottomHeight
            // );
            let remainder = contentY % ROW_HEIGHT;
            prevRemainder = remainder;
            // console.log("remainder", remainder);
            // scrollOffset = -contentY / 20;

            songsStartSlice = Math.floor(
                Math.max(
                    0,
                    Math.floor(scrollNormalized * songsCountScrollable),
                ),
            );
            songsEndSlice = Math.min(
                songs.length,
                Math.ceil(songsStartSlice + songsCountViewport),
            );
            // console.log("start", songsStartSlice, "end", songsEndSlice);

            const getTopOffscreenRows = (
                firstOnScreenIdx,
                backwardsCount,
                prefix,
            ) => {
                let offscreenRows = [];
                let firstSongIdx = Math.max(
                    0,
                    firstOnScreenIdx - backwardsCount,
                );
                let lastSongIdx = Math.max(0, firstOnScreenIdx);
                if (firstOnScreenIdx >= 0 && lastSongIdx >= 0) {
                    offscreenRows.unshift(
                        ...songs
                            .slice(firstSongIdx, lastSongIdx)
                            .map((s) => ({ ...s, dummy: true })),
                    );
                }
                let dummyCount =
                    firstOnScreenIdx - backwardsCount < 0
                        ? Math.abs(firstOnScreenIdx - backwardsCount)
                        : 0;
                if (dummyCount) {
                    offscreenRows.unshift(
                        ...getDummyRows(dummyCount, "dummy-top"),
                    );
                }
                return offscreenRows;
            };

            const getBottomOffscreenRows = (
                lastOnScreenIdx,
                forwardsCount,
                prefix,
            ) => {
                let offscreenRows = [];
                let firstSongIdx = Math.min(songs.length - 1, lastOnScreenIdx);
                let lastSongIdx = Math.min(
                    songs.length - 1,
                    lastOnScreenIdx + forwardsCount,
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
                            .map((s) => ({ ...s, dummy: true })),
                    );
                }
                let dummyRemainder =
                    songs.length - (lastOnScreenIdx + forwardsCount);
                let dummyCount =
                    dummyRemainder < 0 ? Math.abs(dummyRemainder) : 0;
                if (dummyCount) {
                    offscreenRows.push(
                        ...getDummyRows(dummyCount, "dummy-bottom"),
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
                        dummy: true,
                    }));
            };

            // Make sure the window is always filled with the right amount of rows
            songsSlice = songs.slice(songsStartSlice, songsEndSlice);
            // console.log("slice", songsStartSlice, songsEndSlice);
            let diff = songsCountViewport - (songsSlice.length - 1);
            // console.log("diff", diff);
            if (diff) {
                songsSlice = songsSlice.concat(
                    getDummyRows(diff, "dummy-middle"),
                );
            }
            // console.log("slice", songsSlice);
            // Top and bottom dummies
            songsSlice = getTopOffscreenRows(
                songsStartSlice,
                DUMMY_COUNT,
                "dummy-top",
            )
                .concat(...songsSlice)
                .concat(
                    ...getBottomOffscreenRows(
                        songsEndSlice,
                        DUMMY_COUNT,
                        "dummy-bottom",
                    ),
                );
            // console.log(songsSlice.length);
            prevScrollPos = scrollPos;
            // console.log(songsSlice);
        }
    }

    function rememberScrollPos() {
        $libraryScrollPos = scrollNormalized; // 0-1
    }

    function onScroll() {
        scrollPos = scrollContainer.scrollTop;
        scrollNormalized = scrollPos / (contentHeight - viewportHeight);

        if (scrollContainer && stage) {
            calculateSongSlice();
            let idx =
                currentSongScrollIdx !== null
                    ? currentSongScrollIdx
                    : $current.index;
            currentSongInView = idx >= songsStartSlice && idx <= songsEndSlice;
        }

        // Only save/restore scroll pos in main library view, not on playlists
        if ($uiView.match(/(library|albums)/)) {
            debounce(rememberScrollPos, 100)();
        }
    }

    function scrollToCurrentSong() {
        // console.log("y", currentSongY);

        let adjustedPos = currentSongY;
        if (currentSongY > viewportHeight / 2.3) {
            adjustedPos -= viewportHeight / 2.3;
        }
        if ($isPlaying && $current.song) {
            scrollContainer.scrollTo({
                top: adjustedPos,
                behavior: "smooth",
            });
        }
    }

    current.subscribe(async () => {
        if (isOver) {
            return;
        }

        if (scrollContainer && !currentSongInView) {
            scrollToCurrentSong();
        }
    });

    // LIBRARY FUNCTIONALITY

    let isMetaPressed = false;
    let isShiftPressed = false;
    let rangeStartSongIdx = null;
    let rangeEndSongIdx = null;
    let highlightedSongIdx = 0;
    let showQueueMenu = false;
    let showTrackMenu = false;
    let menuPos;
    let shouldProcessDrag = false;

    let songsHighlighted: Song[] = [];
    let onSongsHighlighted = null;

    $: if (
        songs?.length &&
        $query.query?.length &&
        $popupOpen !== "track-info"
    ) {
        highlightSong(songs[0], 0, false, true);
    }

    isShuffleEnabled.subscribe(() => {
        songsHighlighted = [];
    });

    function onDoubleClickSong(song, idx) {
        audioPlayer.playSong(song, 0, true, idx);
    }

    function onRightClick(e, song, idx) {
        if (!songsHighlighted.find((s) => s.viewModel.index === idx)) {
            highlightSong(song, idx, false);
        }

        showTrackMenu = true;
        menuPos = { x: e.clientX, y: e.clientY };
    }

    function isSongHighlighted(song: Song) {
        return songsHighlighted.map((s) => s?.id).includes(song?.id);
    }

    function isSongIdxHighlighted(songIdx: number) {
        return songsHighlighted.find((s) => s?.viewModel?.index === songIdx);
    }

    function onMouseDownSong(song, idx, isKeyboardArrows = false) {
        // Set arrow focus to the queue (library will ignore events)
        $arrowFocus = "queue";

        if (!song) song = songs[0];
        console.log("dragstart", idx);

        highlightedSongIdx = idx;
        if (isSongIdxHighlighted(idx)) {
            if (isMetaPressed) {
                unhighlightSong(song);
            } else if (isShiftPressed) {
                shouldProcessDrag = true;
            } else {
                songsHighlighted = [];
                highlightSong(song, idx, isKeyboardArrows);
            }
        } else {
            highlightSong(song, idx, isKeyboardArrows);
        }

        if (!isKeyboardArrows && shouldProcessDrag) {
            // console.log("songshighlighted", songsHighlighted);
            if (songsHighlighted.length > 1) {
                $draggedSongs = songsHighlighted;
                $isDraggingFromQueue = true;
            } else {
                $draggedSongs = [song];
                $isDraggingFromQueue = true;
            }
        }
    }

    function highlightSong(
        song: Song,
        idx,
        isKeyboardArrows: boolean,
        isDefault = false,
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
                    rangeEndSongIdx + 1,
                );
                rangeStartSongIdx = null;
                rangeEndSongIdx = null;
                $rightClickedTrack = null;
                shouldProcessDrag = false;
                // console.log("highlighted2", songsHighlighted);
            }
        } else if (
            (isKeyboardArrows && isShiftPressed) ||
            hotkeys.isPressed(91)
        ) {
            songsHighlighted.push(song);
            rangeStartSongIdx = idx;
            $rightClickedTrack = null;
            shouldProcessDrag = false;
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
            shouldProcessDrag = true;

            // Extra - if the Info overlay is shown, use the arrows to replace the track shown in the overlay
            if ($popupOpen === "track-info" && isKeyboardArrows) {
                $rightClickedTrack = song;
            }
        }
        // console.log("start", rangeStartSongIdx);

        console.log("highlighted2", songsHighlighted);
        onSongsHighlighted && onSongsHighlighted(songsHighlighted);
    }

    function unhighlightSong(song: Song) {
        songsHighlighted.splice(songsHighlighted.indexOf(song), 1);
        onSongsHighlighted && onSongsHighlighted(songsHighlighted);
    }

    // Something got released over the queue
    async function onMouseUpContainer(e) {
        if (hoveredSongIdx) {
            return;
        }

        if ($draggedSongs?.length) {
            if ($isShuffleEnabled) {
                updateQueues(
                    $draggedSongs,
                    $draggedSongs,
                    (queue, newSongs) => {
                        queue.push(...newSongs);
                    },
                );
            } else {
                updateQueues($draggedSongs, null, (queue, newSongs) => {
                    queue.push(...newSongs);
                });
            }
        }

        $draggedSongs = [];
        $draggedAlbum = null;
    }

    // Something got released over a song in the queue
    async function onMouseUpSong(song: Song, idx: number) {
        if (!$draggedSongs?.length) {
            return;
        }

        console.log("mouse up - song", $isDraggingFromQueue);

        const delta =
            song.viewModel?.index > $draggedSongs[0].viewModel?.index ? 1 : 0;

        if ($isDraggingFromQueue) {
            if ($draggedSongs.includes(song)) {
                return;
            }

            console.log("reorder song", $draggedSongs, idx);

            if ($isShuffleEnabled) {
                updateQueues(
                    [findQueueIndex($shuffledQueue[idx]), $draggedSongs, delta],
                    [idx, $draggedSongs, delta],
                    reorderSongs,
                );
            } else {
                updateQueues([idx, $draggedSongs, delta], null, reorderSongs);
            }

            songsHighlighted = [];
        } else {
            // Drop from library
            console.log("drop to queue", $draggedSongs, idx);

            if ($isShuffleEnabled) {
                updateQueues(
                    [findQueueIndex($shuffledQueue[idx]), $draggedSongs],
                    [idx, $draggedSongs],
                    (queue, [index, newSongs]) => {
                        queue.splice(index + delta, 0, ...newSongs);
                    },
                );
            } else {
                updateQueues(
                    [idx, $draggedSongs],
                    null,
                    (queue, [index, newSongs]) => {
                        queue.splice(index + delta, 0, ...newSongs);
                    },
                );
            }

            // Avoid layout shift - compensate for the scroll jump after adding new elements
            scrollContainer.scrollBy({
                top: ROW_HEIGHT * $draggedSongs.length,
            });
        }

        $draggedSongs = [];
        $draggedAlbum = null;
        $isDraggingFromQueue = false;
    }

    function reorderSongs(queue, [index, songs, delta]) {
        const id = queue[index].id;
        const indexes = songs.map((t) => t.viewModel.index);

        for (const index of indexes.sort((a, b) => b - a)) {
            queue.splice(index, 1);
        }

        const newIndex = queue.findIndex((song) => song.id === id);

        queue.splice(
            newIndex + delta,
            0,
            ...songs.sort((a, b) => a.viewModel.index - b.viewModel.index),
        );
    }

    function onStageClick(e) {
        if (e.detail.evt.button === 2) {
            e.preventDefault();
            showQueueMenu = true;
            menuPos = { x: e.detail.evt.clientX, y: e.detail.evt.clientY };
        }
    }

    // Shortcuts

    hotkeys("esc", function (event, handler) {
        if ($isSmartQueryBuilderOpen) {
            $isSmartQueryBuilderOpen = false;
        }
    });

    function onKeyDown(event) {
        if ($arrowFocus !== "queue") return;

        if (event.keyCode === 16) {
            isShiftPressed = true;
            console.log("shift pressed");
        } else if ($os !== "macos" && event.keyCode === 17) {
            isMetaPressed = true;
            console.log("ctrl pressed");
        } else if ($os === "macos" && event.keyCode === 91) {
            isMetaPressed = true;
            console.log("cmd pressed");
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
                onMouseDownSong(
                    queue[highlightedSongIdx - 1],
                    highlightedSongIdx - 1,
                    true,
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
                onMouseDownSong(
                    songs[highlightedSongIdx + 1],
                    highlightedSongIdx + 1,
                    true,
                );
            }
        } else if (
            event.keyCode === 73 &&
            $popupOpen !== "track-info" &&
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
            if ($popupOpen !== "track-info" && songsHighlighted.length) {
                console.log("opening info", songsHighlighted);
                if (songsHighlighted.length > 1) {
                    $rightClickedTracks = songsHighlighted;
                } else {
                    $rightClickedTrack = songsHighlighted[0];
                }
                $popupOpen = "track-info";
            }
        } else if (
            event.keyCode === 13 &&
            $popupOpen !== "track-info" &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // 'Enter' to play highlighted track
            event.preventDefault();
            if ($popupOpen !== "track-info") {
                setQueue($queriedSongs, highlightedSongIdx);
            }
        }
    }
    function onKeyUp(event) {
        if (event.keyCode === 16) {
            isShiftPressed = false;
            console.log("shift lifted");
        } else if (event.keyCode === 17) {
            isMetaPressed = false;
            console.log("ctrl lifted");
        } else if (event.keyCode === 91) {
            isMetaPressed = false;
            console.log("cmd lifted");
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

    // Re-order columns

    $: {
        displayFields && $columnOrder && calculateColumns();
    }

    // Sets back to default
    function resetColumnOrder() {
        fields = DEFAULT_FIELDS;
        $columnOrder = DEFAULT_FIELDS.map((f) => f.value);
    }

    // Favourite

    async function favouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: true,
        });

        song.isFavourite = true;
        songs = songs;
        if ($current.song?.id === song.id) {
            $current.song.isFavourite = true;
        }
    }

    async function unfavouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: false,
        });
        song.isFavourite = false;
        songs = songs;
        if ($current.song?.id === song.id) {
            $current.song.isFavourite = false;
        }
    }

    function filterByField(fieldName: string, fieldValue: any) {
        let queryPart;
        // console.log("filter", fieldName, fieldValue);
        switch (fieldName) {
            case "genre":
                queryPart = getQueryPart("CONTAINS_GENRE");
                $smartQueryInitiator = "library-cell";
                break;
            case "year":
                queryPart = getQueryPart("RELEASED_IN");
                $smartQueryInitiator = "library-cell";
                break;
            case "originCountry":
                queryPart = getQueryPart("FROM_COUNTRY");
                $smartQueryInitiator = "library-cell";
                break;
            default:
                return;
        }
        if ($uiView.match(/^(smart-query|favourites)/) === null) {
            $smartQuery.reset();
        }
        // console.log("built in query Part", queryPart);
        const userQueryPart = new UserQueryPart(queryPart);
        // console.log("built in user query Part", userQueryPart);
        userQueryPart.userInputs[fieldName].value = fieldValue;
        $smartQuery.addPart(userQueryPart);
        $smartQuery.parts = $smartQuery.parts;
        $isSmartQueryBuilderOpen = true;
        $isSmartQuerySaveUiOpen = false;
        $uiView = "smart-query";
    }

    function isInvalidValue(value) {
        return (
            value === "" ||
            value === -1 ||
            value === 0 ||
            value === null ||
            value === undefined ||
            value?.length === 0
        );
    }

    function validatedValue(value) {
        return isInvalidValue(value) ? "-" : value;
    }
</script>

<!-- <svelte:window on:resize={debounce(onResize, 5)} /> -->

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-click-events-have-key-events -->

<QueueMenu bind:showMenu={showQueueMenu} bind:pos={menuPos} />
<TrackMenu
    bind:showMenu={showTrackMenu}
    bind:pos={menuPos}
    bind:songs={songsHighlighted}
/>
<ColumnPicker
    bind:showMenu={showColumnPicker}
    bind:pos={columnPickerPos}
    bind:fields
    bind:columnOrder={$columnOrder}
    onResetOrder={resetColumnOrder}
    {isOrderChanged}
/>

<div
    class="library-container"
    class:dragover={isDraggingOver}
    bind:this={libraryContainer}
>
    {#if isLoading}
        <div class="loading" out:fade={{ duration: 90, easing: cubicInOut }}>
            <p>💿 one sec...</p>
        </div>
    {:else}
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
            id="scroll-container"
            style="overflow-y: {isScrollable ? 'visible' : 'hidden'}"
            class:ready
            on:scroll={onScroll}
            bind:this={scrollContainer}
            on:mouseup={onMouseUpContainer}
            on:mouseenter={() => {
                isOver = true;
                isDraggingOver = $draggedSongs?.length > 0;
            }}
            on:mouseleave={() => {
                isOver = false;
                isDraggingOver = false;
            }}
        >
            <div
                id="large-container"
                style="height: {scrollableArea}px;max-height: {scrollableArea}px;"
            />

            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="container"
                bind:this={container}
                on:contextmenu|preventDefault
            >
                {#if dim}
                    <div class="dimmer" />
                {/if}

                {#if shouldRender}
                    <Stage
                        config={{
                            width,
                            height: virtualViewportHeight,
                            y: -sandwichTopHeight,
                        }}
                        bind:handle={stage}
                        on:click={onStageClick}
                    >
                        <Layer>
                            <Rect
                                config={{
                                    x: 0,
                                    y: sandwichTopHeight,
                                    width,
                                    height: viewportHeight,
                                    fill: BG_COLOR,
                                }}
                            />

                            {#if songsSlice?.length}
                                {#each songsSlice as song, songIdx (song.viewModel?.viewId ?? songIdx)}
                                    <Group
                                        on:dblclick={() =>
                                            onDoubleClickSong(
                                                song,
                                                song?.viewModel?.index,
                                            )}
                                        on:click={(e) => {
                                            if (e.detail.evt.button === 2) {
                                                e.preventDefault();

                                                onRightClick(
                                                    e.detail.evt,
                                                    song,
                                                    song.viewModel.index,
                                                );
                                            }
                                        }}
                                        on:mouseenter={() => {
                                            hoveredSongIdx = songIdx;
                                            if (
                                                $draggedSongs?.length &&
                                                songIdx > songsSlice.length - 15
                                            ) {
                                                scrollContainer?.scrollBy({
                                                    top: ROW_HEIGHT,
                                                });
                                            } else if (
                                                $draggedSongs?.length &&
                                                songIdx < 10
                                            ) {
                                                scrollContainer?.scrollBy({
                                                    top: -ROW_HEIGHT,
                                                });
                                            }
                                        }}
                                        on:mouseleave={() => {
                                            hoveredSongIdx = null;
                                        }}
                                        on:mousedown={(e) =>
                                            e.detail.evt.button === 0 &&
                                            onMouseDownSong(
                                                song,
                                                song.viewModel?.index,
                                            )}
                                        config={{
                                            x: 0,
                                            y:
                                                sandwichTopHeight +
                                                HEADER_HEIGHT +
                                                ROW_HEIGHT * songIdx +
                                                -DUMMY_PADDING +
                                                scrollOffset,
                                            visible: !song.dummy,
                                        }}
                                        on:mouseup={(e) => {
                                            onMouseUpSong(
                                                song,
                                                song.viewModel.index,
                                            );
                                        }}
                                    >
                                        <Rect
                                            config={{
                                                width: width,
                                                height: ROW_HEIGHT,
                                                listening: true,
                                                fill: $draggedSongs.includes(
                                                    song,
                                                )
                                                    ? DRAGGING_SOURCE_COLOR
                                                    : $current.index ===
                                                            song?.viewModel
                                                                ?.index &&
                                                        song.id ===
                                                            $current.song?.id
                                                      ? PLAYING_BG_COLOR
                                                      : songsHighlighted &&
                                                          isSongIdxHighlighted(
                                                              song?.viewModel
                                                                  ?.index,
                                                          )
                                                        ? HIGHLIGHT_BG_COLOR
                                                        : hoveredSongIdx ===
                                                            songIdx
                                                          ? ROW_BG_COLOR_HOVERED
                                                          : ROW_BG_COLOR,
                                            }}
                                        />
                                        {#each displayFields as f, idx (f.value)}
                                            <Text
                                                config={{
                                                    x:
                                                        f.value.match(
                                                            /^(title|artist|album|track)/,
                                                        ) !== null
                                                            ? f.viewProps.x + 10
                                                            : f.viewProps.x,
                                                    y: 1,
                                                    text: validatedValue(
                                                        song[f.value],
                                                    ),
                                                    listening: false,
                                                    align:
                                                        f.value.match(
                                                            /^(title|artist|album|track)/,
                                                        ) !== null
                                                            ? "left"
                                                            : "center",
                                                    width:
                                                        idx ===
                                                        displayFields.length - 1
                                                            ? f.viewProps
                                                                  .width - 10
                                                            : f.value.match(
                                                                    /^(title|artist|album|track)/,
                                                                ) !== null
                                                              ? f.value ===
                                                                    "title" &&
                                                                $current.song
                                                                    ?.id ===
                                                                    song.id
                                                                  ? f.viewProps
                                                                        .width -
                                                                    38
                                                                  : f.viewProps
                                                                        .width -
                                                                    12
                                                              : f.viewProps
                                                                    .width,
                                                    padding:
                                                        f.value.match(
                                                            /^(genre)/,
                                                        ) !== null
                                                            ? 10
                                                            : 2,
                                                    height: HEADER_HEIGHT,
                                                    fontSize: 13.5,
                                                    verticalAlign: "middle",
                                                    fill:
                                                        $current.index ===
                                                            song?.viewModel
                                                                ?.index &&
                                                        song.id ===
                                                            $current.song?.id
                                                            ? PLAYING_TEXT_COLOR
                                                            : TEXT_COLOR,
                                                    ellipsis:
                                                        f.value.match(
                                                            /^(title|artist|album|genre)/,
                                                        ) !== null,
                                                }}
                                            />

                                            {#if hoveredSongIdx === songIdx && $draggedSongs?.length === 0}
                                                <Path
                                                    config={{
                                                        x: -2,
                                                        y: 6,
                                                        listening: false,
                                                        scaleX: 0.9,
                                                        scaleY: 0.9,
                                                        data: "M7.375 3.67c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m0 8.66c0-.646-.56-1.17-1.25-1.17s-1.25.524-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17m-1.25-5.5c.69 0 1.25.525 1.25 1.17c0 .645-.56 1.17-1.25 1.17S4.875 8.645 4.875 8c0-.645.56-1.17 1.25-1.17m5-3.16c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m-1.25 7.49c.69 0 1.25.524 1.25 1.17c0 .645-.56 1.17-1.25 1.17s-1.25-.525-1.25-1.17c0-.646.56-1.17 1.25-1.17M11.125 8c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17",
                                                        fill: "rgba(255, 255, 255, 0.5)",
                                                    }}
                                                />
                                            {/if}

                                            {#if f.value === "title"}
                                                <!-- Now playing icon -->
                                                {#if $current.index === song?.viewModel?.index && song.id === $current.song?.id}
                                                    <Path
                                                        config={{
                                                            x:
                                                                f.viewProps
                                                                    .width - 40,
                                                            y: 7,
                                                            listening: false,
                                                            scaleX: 0.65,
                                                            scaleY: 0.65,
                                                            data: "M9.383 3.076A1 1 0 0 1 10 4v12a1 1 0 0 1-1.707.707L4.586 13H2a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1h2.586l3.707-3.707a1 1 0 0 1 1.09-.217m5.274-.147a1 1 0 0 1 1.414 0A9.972 9.972 0 0 1 19 10a9.972 9.972 0 0 1-2.929 7.071a1 1 0 0 1-1.414-1.414A7.971 7.971 0 0 0 17 10a7.97 7.97 0 0 0-2.343-5.657a1 1 0 0 1 0-1.414m-2.829 2.828a1 1 0 0 1 1.415 0A5.983 5.983 0 0 1 15 10a5.984 5.984 0 0 1-1.757 4.243a1 1 0 0 1-1.415-1.415A3.984 3.984 0 0 0 13 10a3.983 3.983 0 0 0-1.172-2.828a1 1 0 0 1 0-1.415",
                                                            fill: $currentThemeObject[
                                                                "library-playing-icon"
                                                            ],
                                                        }}
                                                    />
                                                {/if}

                                                <!-- Favourite icon button -->
                                                {#if song.isFavourite}
                                                    <Path
                                                        on:click={() =>
                                                            unfavouriteSong(
                                                                song,
                                                            )}
                                                        config={{
                                                            x:
                                                                f.viewProps
                                                                    .width - 20,
                                                            y: 6,
                                                            scaleX: 0.36,
                                                            scaleY: 0.36,
                                                            data: "M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87",
                                                            fill:
                                                                $current.song
                                                                    ?.id ===
                                                                song.id
                                                                    ? $currentThemeObject[
                                                                          "library-playing-icon"
                                                                      ]
                                                                    : $currentThemeObject[
                                                                          "library-favourite-icon"
                                                                      ],
                                                        }}
                                                    />
                                                {:else if hoveredSongIdx === songIdx}
                                                    <Path
                                                        on:click={() =>
                                                            favouriteSong(song)}
                                                        config={{
                                                            x:
                                                                f.viewProps
                                                                    .width - 20,
                                                            y: 6,
                                                            scaleX: 0.36,
                                                            scaleY: 0.36,
                                                            data: "M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87",
                                                            fill: "transparent",
                                                            stroke:
                                                                $current.song
                                                                    ?.id ===
                                                                song.id
                                                                    ? $currentThemeObject[
                                                                          "library-playing-icon"
                                                                      ]
                                                                    : $currentThemeObject[
                                                                          "library-favourite-hover-icon"
                                                                      ],
                                                        }}
                                                    />
                                                {/if}
                                            {/if}
                                        {/each}
                                    </Group>

                                    {#if hoveredSongIdx === songIdx && $draggedSongs?.length}
                                        <Rect
                                            on:mouseup={(e) => {
                                                onMouseUpSong(
                                                    song,
                                                    song.viewModel.index,
                                                );
                                            }}
                                            config={{
                                                x: 0,
                                                y:
                                                    sandwichTopHeight +
                                                    HEADER_HEIGHT +
                                                    ROW_HEIGHT * songIdx +
                                                    -DUMMY_PADDING +
                                                    scrollOffset +
                                                    (song.viewModel?.index >
                                                    $draggedSongs[0].viewModel
                                                        ?.index
                                                        ? ROW_HEIGHT
                                                        : 0),
                                                width: width,
                                                height: DROP_HINT_HEIGHT,
                                                fill: PLAYING_BG_COLOR,
                                                listening: true,
                                            }}
                                        />
                                    {/if}
                                {/each}
                                {#if isScrollable}
                                    <Rect
                                        config={{
                                            x: 0,
                                            y: sandwichBottomY,
                                            width,
                                            height:
                                                sandwichBottomHeight +
                                                ROW_HEIGHT,
                                            fill: OFFSCREEN_BG_COLOR,
                                            listening: false,
                                        }}
                                    />
                                {/if}
                            {:else}
                                <Text
                                    config={{
                                        x: 10,
                                        y: HEADER_HEIGHT,
                                        text: "Nothing here yet",
                                        listening: false,
                                        align: "left",
                                        width: width,
                                        padding: 2,
                                        height: HEADER_HEIGHT,
                                        fontSize: 13.5,
                                        verticalAlign: "middle",
                                        fill: TEXT_COLOR,
                                    }}
                                />
                            {/if}
                        </Layer>
                        <Layer>
                            {#if isScrollable}
                                <Rect
                                    config={{
                                        x: 0,
                                        y: 0,
                                        width,
                                        height: sandwichTopHeight,
                                        fill: OFFSCREEN_BG_COLOR,
                                        listening: false,
                                    }}
                                />
                            {/if}

                            {#if columnToInsertIdx !== null}
                                <Rect
                                    config={{
                                        x: columnToInsertXPos,
                                        y: sandwichTopHeight + HEADER_HEIGHT,
                                        height: viewportHeight,
                                        width: 2,
                                        fill: COLUMN_INSERT_HINT_COLOR,
                                        listening: false,
                                    }}
                                />
                            {/if}
                            <!-- HEADER -->
                            <Group
                                config={{
                                    x: 0,
                                    y: sandwichTopHeight,
                                    width: width,
                                }}
                            >
                                <Rect
                                    config={{
                                        width: width - 0.5,
                                        height: HEADER_HEIGHT,
                                        listening: true,
                                        fill: HEADER_BG_COLOR,
                                    }}
                                />
                                <Rect
                                    config={{
                                        x: 0,
                                        y: HEADER_HEIGHT,
                                        width: width,
                                        height: 0.5,
                                        fill: "#544e55",
                                    }}
                                />
                                <Text
                                    config={{
                                        x:
                                            !$isSidebarOpen && $os === "macos"
                                                ? WINDOW_CONTROLS_WIDTH
                                                : null,
                                        text: "Queue",
                                        align: "left",
                                        padding: 10,
                                        height: HEADER_HEIGHT,
                                        fontSize: 14,
                                        letterSpacing: 0,
                                        fontStyle: "bold",
                                        verticalAlign: "middle",
                                        fontFamily:
                                            "-apple-system, Avenir, Helvetica, Arial, sans-serif",
                                        fill: TEXT_COLOR,
                                        listening: false,
                                    }}
                                />
                            </Group>
                        </Layer>
                    </Stage>
                {/if}
                {#if $isSmartQueryBuilderOpen && noSongs}
                    <SmartQueryResultsPlaceholder />
                {/if}
            </div>
        </div>
    {/if}

    <ShadowGradient type="bottom" />
</div>

<style lang="scss">
    .library-container {
        position: relative;
        max-width: 290px;
        width: 290px;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 5px;
        border-left: 0.7px solid var(--panel-secondary-border-main);
        border-bottom: 0.7px solid var(--panel-secondary-border-main);
        border-right: 0.7px solid var(--panel-secondary-border-main);
        overflow: hidden;
        margin: 4px 0 0 0;
        &.dragover {
            border: 1.5px solid var(--accent-secondary);
        }
    }
    .container {
        width: 100%;
        height: 100%;
        pointer-events: all;
        display: flex;
        flex-direction: column;
        position: sticky;
        top: 0;
        bottom: 0;
        user-select: none;
    }

    .dimmer {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        width: 100%;
        height: 100%;
        z-index: 12;
        pointer-events: none;
        background-color: #1b1b1c61;
        backdrop-filter: brightness(0.7);
    }

    #large-container {
        width: 100%;
        overflow: hidden;
    }

    #scroll-container {
        width: 100%;
        height: 100%;
        position: absolute;
        overflow-y: auto;
        overflow-x: hidden;
        z-index: 10;
        pointer-events: auto;
        visibility: hidden;
        &.ready {
            visibility: visible;
        }
    }

    .smart-query {
        display: grid;
        background-color: #4d347c;
        /* min-height: 51px; */
        .smart-query-builder {
            grid-column-start: 1;
            grid-column-end: 2;
            grid-row-start: 1;
            grid-row-end: 2;
        }

        .smart-query-main {
            grid-column-start: 1;
            grid-column-end: 2;
            grid-row-start: 1;
            grid-row-end: 2;
        }
    }

    .bottom-bar {
        position: sticky;
        bottom: 0;
        left: 0;
        right: 0;
        z-index: 15;
    }

    .big-seekbar {
        position: sticky;
        bottom: 29.5px;
        left: 0;
        right: 0;
        z-index: 15;
    }
</style>
