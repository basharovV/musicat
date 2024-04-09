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
        type KonvaDragTransformEvent
    } from "svelte-konva";

    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import "iconify-icon";
    import { debounce } from "lodash-es";
    import type { Song } from "src/App";
    import { onDestroy } from "svelte";
    import { cubicInOut } from "svelte/easing";
    import { fade, fly } from "svelte/transition";
    import { db } from "../../data/db";

    import {
        bottomBarNotification,
        compressionSelected,
        currentSong,
        currentSongIdx,
        draggedColumnIdx,
        draggedSongs,
        emptyDropEvent,
        fileDropHandler,
        forceRefreshLibrary,
        importStatus,
        isPlaying,
        isQueueOpen,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isTrackInfoPopupOpen,
        libraryScrollPos,
        os,
        playlist,
        playlistIsAlbum,
        queriedSongs,
        query,
        rightClickedTrack,
        rightClickedTracks,
        shouldFocusFind,
        singleKeyShortcutsEnabled,
        smartQuery,
        smartQueryInitiator,
        uiView
    } from "../../data/store";
    import {
        moveArrayElement,
        swapArrayElements
    } from "../../utils/ArrayUtils";
    import AudioPlayer from "../player/AudioPlayer";
    import SmartQueryBuilder from "../smart-query/SmartQueryBuilder.svelte";
    import SmartQueryMainHeader from "../smart-query/SmartQueryMainHeader.svelte";
    import SmartQueryResultsPlaceholder from "../smart-query/SmartQueryResultsPlaceholder.svelte";
    import BottomBar from "./BottomBar.svelte";
    import ColumnPicker from "./ColumnPicker.svelte";
    import ImportPlaceholder from "./ImportPlaceholder.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    import { getQueryPart } from "../smart-query/QueryParts";
    import { UserQueryPart } from "../smart-query/UserQueryPart";

    export let allSongs = null;
    export let dim = false;
    export let isLoading = false;
    export let theme = "default";
    export let isInit = true;

    $: songs =
        $allSongs
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
                                songsArray[
                                    status.state.firstSongInPreviousAlbum
                                ];
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
                                songsArray[
                                    status.state.firstSongInPreviousArtist
                                ];
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
            )?.songs ?? [];

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
            show: false,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true
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
    let hoveredField = null;

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
    const HEADER_HEIGHT = 26;
    const ROW_HEIGHT = 26;
    const BORDER_WIDTH = 1;
    const SCROLL_PADDING = 200;
    const DUMMY_COUNT = 5;
    const DUMMY_PADDING = DUMMY_COUNT * ROW_HEIGHT;

    // COLORS
    const BG_COLOR = "rgba(36, 33, 34, 0.948)";
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
    const CLICKABLE_CELL_BG_COLOR = "#71658e1e";
    const CLICKABLE_CELL_BG_COLOR_HOVERED = "#8c7dae36";

    onMount(() => {
        init();

        // Check for retina screens
        var query =
            "(-webkit-min-device-pixel-ratio: 2), (min-device-pixel-ratio: 2), (min-resolution: 192dpi)";

        if (matchMedia(query).matches) {
            // Slightly reduce the ratio for better performance
            // Konva.pixelRatio = 1.8;
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

    $: noSongs =
        !songs ||
        songs.length === 0 ||
        ($uiView.match(/^(smart-query|favourites)/) &&
            $isSmartQueryBuilderOpen &&
            $smartQuery.isEmpty);

    // Trigger: on songs updated
    $: {
        if (songs !== undefined && libraryContainer) {
            console.log("Library::songs updated", songs.length);
            drawSongDataGrid();
            prevSongCount = songs.length;
        }
    }

    // Restore scroll position if any
    $: if (
        $uiView === "library" &&
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
                top: (contentHeight - viewportHeight) * $libraryScrollPos,
                behavior: "instant"
            });
            ready = true;
        }, 50);
        isInit = false;
        $forceRefreshLibrary = false;
    } else if ($uiView === "playlists") {
        scrollContainer?.scrollTo({
            top: 0
        });
        ready = true;
    } else if ($uiView.match(/^(smart-query|favourites)/)) {
        scrollContainer?.scrollTo({
            top: 0
        });
        ready = true;
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

        // Fields visible depending on window width
        const visibleFields = fields.filter((f) => {
            switch (f.value) {
                case "duration":
                    return f.show && width > 800;
                case "genre":
                    return f.show && width > 700;
                case "year":
                    return f.show && width > 650;
                case "trackNumber":
                    return f.show && width > 500;
                case "album":
                    return f.show && width > 450;
                default:
                    return f.show;
            }
        });
        // Calculate total width of fixed-width rectangles
        const fixedWidths = visibleFields
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
            })
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

    function rememberScrollPos() {
        $libraryScrollPos = scrollNormalized; // 0-1
    }

    function onScroll() {
        scrollPos = scrollContainer.scrollTop;
        scrollNormalized = scrollPos / (contentHeight - viewportHeight);

        if (scrollContainer && stage) {
            calculateSongSlice();
            currentSongInView =
                $currentSongIdx >= songsStartSlice &&
                $currentSongIdx <= songsEndSlice;
        }

        // Only save/restore scroll pos in main library view, not on playlists
        if ($uiView === "library") {
            debounce(rememberScrollPos, 100)();
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
        // console.log("y", currentSongY);
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
        AudioPlayer.shouldPlay = false;
        $currentSongIdx = idx;
        $playlist = $queriedSongs;
        $playlistIsAlbum = false;
        AudioPlayer.playSong(song);
    }

    function onRightClick(e, song, idx) {
        if (!songsHighlighted.includes(song)) {
            highlightSong(song, idx, false);
        }

        // console.log("songIdsHighlighted", songsHighlighted);
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
        // console.log("highlighted", song, idx);
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
                // console.log("highlighted2", songsHighlighted);
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
        // console.log("dragstart", song);
        // console.log("songshighlighted", songsHighlighted);
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
                AudioPlayer.shouldPlay = false;
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

    // SMART QUERY
    export let isSmartQueryEnabled = true; // Only for main view

    // Favourite

    async function favouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: true
        });

        if ($currentSong?.id === song.id) {
            $currentSong.isFavourite = true;
        }
    }

    async function unfavouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: false
        });
        if ($currentSong?.id === song.id) {
            $currentSong.isFavourite = false;
        }
    }

    $: if ($bottomBarNotification?.timeout) {
        setTimeout(() => {
            $bottomBarNotification = null;
        }, $bottomBarNotification.timeout);
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

<TrackMenu bind:showMenu={showTrackMenu} bind:pos={menuPos} />
<ColumnPicker
    bind:showMenu={showColumnPicker}
    bind:pos={columnPickerPos}
    bind:fields
    onResetOrder={resetColumnOrder}
    {isOrderChanged}
/>

<div class="library-container" bind:this={libraryContainer}>
    {#if isLoading}
        <div class="loading" out:fade={{ duration: 90, easing: cubicInOut }}>
            <p>💿 one sec...</p>
        </div>
    {:else if theme === "default" && (($importStatus.isImporting && $importStatus.backgroundImport === false) || (noSongs && $query.query.length === 0 && $uiView.match(/^(smart-query|favourites)/) === null))}
        <ImportPlaceholder />
    {:else}
        <div
            id="scroll-container"
            style="overflow-y: {isScrollable ? 'visible' : 'hidden'}"
            class:ready
            on:scroll={onScroll}
            bind:this={scrollContainer}
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
                <!-- svelte-ignore a11y-click-events-have-key-events -->
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                {#if $uiView === "library" && $isPlaying && $currentSong && !currentSongInView}
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

                {#if $uiView.match(/^(smart-query)/)}
                    <div
                        class="smart-query"
                        transition:fly={{
                            y: -10,
                            duration: 200,
                            easing: cubicInOut
                        }}
                    >
                        {#if $isSmartQueryBuilderOpen}
                            <div
                                in:fade={{
                                    duration: 150
                                }}
                                class="smart-query-builder"
                            >
                                <SmartQueryBuilder />
                            </div>
                        {:else}
                            <div
                                class="smart-query-main"
                                transition:fly={{
                                    y: -10,
                                    duration: 150,
                                    easing: cubicInOut
                                }}
                            >
                                <SmartQueryMainHeader />
                            </div>
                        {/if}
                    </div>
                {/if}
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
                                    y: sandwichTopHeight,
                                    width,
                                    height: viewportHeight,
                                    fill: BG_COLOR
                                }}
                            />

                            {#if songsSlice?.length}
                                {#each songsSlice as song, songIdx (song.viewModel?.viewId ?? song.id)}
                                    <Group
                                        on:dblclick={() =>
                                            onDoubleClickSong(song, songIdx)}
                                        on:click={(e) => {
                                            // console.log("e", e);
                                            if (e.detail.evt.button === 0) {
                                                toggleHighlight(
                                                    song,
                                                    song.viewModel.index
                                                );
                                            } else if (
                                                e.detail.evt.button === 2
                                            ) {
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
                                                listening: true,
                                                fill:
                                                    $currentSong?.id ===
                                                    song?.id
                                                        ? PLAYING_BG_COLOR
                                                        : songsHighlighted &&
                                                            isSongHighlighted(
                                                                song
                                                            )
                                                          ? HIGHLIGHT_BG_COLOR
                                                          : hoveredSongIdx ===
                                                              songIdx
                                                            ? ROW_BG_COLOR_HOVERED
                                                            : ROW_BG_COLOR
                                            }}
                                        />
                                        {#each displayFields as f, idx (f.value)}
                                            <!-- Smart query fields -->
                                            {#if f.value.match(/^(year|genre|originCountry)/) !== null && !isInvalidValue(song[f.value])}
                                                <Label
                                                    config={{
                                                        x: f.viewProps.x + 5,
                                                        y:
                                                            sandwichTopHeight +
                                                            HEADER_HEIGHT +
                                                            ROW_HEIGHT *
                                                                songIdx +
                                                            -DUMMY_PADDING +
                                                            scrollOffset +
                                                            2.5,
                                                        width:
                                                            f.viewProps.width -
                                                            10,
                                                        height: ROW_HEIGHT - 5
                                                    }}
                                                    on:mouseenter={() => {
                                                        hoveredField = f.value;
                                                    }}
                                                    on:mouseleave={() => {
                                                        hoveredField = null;
                                                    }}
                                                    on:click={() => {
                                                        filterByField(
                                                            f.value,
                                                            song[f.value]
                                                        );
                                                    }}
                                                >
                                                    <Tag
                                                        config={{
                                                            fill:
                                                                hoveredSongIdx ===
                                                                    songIdx &&
                                                                hoveredField ===
                                                                    f.value
                                                                    ? CLICKABLE_CELL_BG_COLOR_HOVERED
                                                                    : CLICKABLE_CELL_BG_COLOR,
                                                            padding: 10,
                                                            cornerRadius: 2
                                                        }}
                                                    />
                                                    <Text
                                                        config={{
                                                            text: validatedValue(
                                                                song[f.value]
                                                            ),
                                                            listening: false,
                                                            y: 0,
                                                            x: 0,
                                                            width:
                                                                f.viewProps
                                                                    .width - 10,
                                                            height:
                                                                ROW_HEIGHT - 5,
                                                            align: "center",
                                                            fontSize: 13.5,
                                                            verticalAlign:
                                                                "middle",
                                                            fill:
                                                                $currentSong?.id ===
                                                                song.id
                                                                    ? PLAYING_TEXT_COLOR
                                                                    : TEXT_COLOR,
                                                            ellipsis:
                                                                f.value.match(
                                                                    /^(title|artist|album|genre)/
                                                                ) !== null
                                                        }}
                                                    />
                                                </Label>
                                            {:else}
                                                <Text
                                                    config={{
                                                        x:
                                                            f.value.match(
                                                                /^(title|artist|album|track)/
                                                            ) !== null
                                                                ? f.viewProps
                                                                      .x + 10
                                                                : f.viewProps.x,
                                                        y:
                                                            sandwichTopHeight +
                                                            HEADER_HEIGHT +
                                                            ROW_HEIGHT *
                                                                songIdx +
                                                            -DUMMY_PADDING +
                                                            scrollOffset,
                                                        text: validatedValue(
                                                            song[f.value]
                                                        ),
                                                        listening: false,
                                                        align:
                                                            f.value.match(
                                                                /^(title|artist|album|track)/
                                                            ) !== null
                                                                ? "left"
                                                                : "center",
                                                        width:
                                                            idx ===
                                                            displayFields.length -
                                                                1
                                                                ? f.viewProps
                                                                      .width -
                                                                  10
                                                                : f.value.match(
                                                                        /^(title|artist|album|track)/
                                                                    ) !== null
                                                                  ? f.value ===
                                                                        "title" &&
                                                                    $currentSong?.id ===
                                                                        song.id
                                                                      ? f
                                                                            .viewProps
                                                                            .width -
                                                                        38
                                                                      : f
                                                                            .viewProps
                                                                            .width -
                                                                        12
                                                                  : f.viewProps
                                                                        .width,
                                                        padding:
                                                            f.value.match(
                                                                /^(genre)/
                                                            ) !== null
                                                                ? 10
                                                                : 2,
                                                        height: HEADER_HEIGHT,
                                                        fontSize: 13.5,
                                                        verticalAlign: "middle",
                                                        fill:
                                                            $currentSong?.id ===
                                                            song.id
                                                                ? PLAYING_TEXT_COLOR
                                                                : TEXT_COLOR,
                                                        ellipsis:
                                                            f.value.match(
                                                                /^(title|artist|album|genre)/
                                                            ) !== null
                                                    }}
                                                />
                                            {/if}

                                            {#if f.value === "title"}
                                                <!-- Now playing icon -->
                                                {#if $currentSong?.id === song.id}
                                                    <Path
                                                        config={{
                                                            x:
                                                                f.viewProps
                                                                    .width - 40,
                                                            y:
                                                                sandwichTopHeight +
                                                                HEADER_HEIGHT +
                                                                ROW_HEIGHT *
                                                                    songIdx +
                                                                -DUMMY_PADDING +
                                                                scrollOffset +
                                                                7,
                                                            listening: false,
                                                            scaleX: 0.65,
                                                            scaleY: 0.65,
                                                            data: "M9.383 3.076A1 1 0 0 1 10 4v12a1 1 0 0 1-1.707.707L4.586 13H2a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1h2.586l3.707-3.707a1 1 0 0 1 1.09-.217m5.274-.147a1 1 0 0 1 1.414 0A9.972 9.972 0 0 1 19 10a9.972 9.972 0 0 1-2.929 7.071a1 1 0 0 1-1.414-1.414A7.971 7.971 0 0 0 17 10a7.97 7.97 0 0 0-2.343-5.657a1 1 0 0 1 0-1.414m-2.829 2.828a1 1 0 0 1 1.415 0A5.983 5.983 0 0 1 15 10a5.984 5.984 0 0 1-1.757 4.243a1 1 0 0 1-1.415-1.415A3.984 3.984 0 0 0 13 10a3.983 3.983 0 0 0-1.172-2.828a1 1 0 0 1 0-1.415",
                                                            fill: "#00ddff"
                                                        }}
                                                    />
                                                {/if}

                                                <!-- Favourite icon button -->
                                                {#if song.isFavourite}
                                                    <Path
                                                        on:click={() =>
                                                            unfavouriteSong(
                                                                song
                                                            )}
                                                        config={{
                                                            x:
                                                                f.viewProps
                                                                    .width - 20,
                                                            y:
                                                                sandwichTopHeight +
                                                                HEADER_HEIGHT +
                                                                ROW_HEIGHT *
                                                                    songIdx +
                                                                -DUMMY_PADDING +
                                                                scrollOffset +
                                                                6,
                                                            scaleX: 0.36,
                                                            scaleY: 0.36,
                                                            data: "M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87",
                                                            fill:
                                                                $currentSong.id ===
                                                                song.id
                                                                    ? "#00ddff"
                                                                    : "#5123dd"
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
                                                            y:
                                                                sandwichTopHeight +
                                                                HEADER_HEIGHT +
                                                                ROW_HEIGHT *
                                                                    songIdx +
                                                                -DUMMY_PADDING +
                                                                scrollOffset +
                                                                6,
                                                            scaleX: 0.36,
                                                            scaleY: 0.36,
                                                            data: "M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87",
                                                            fill: "transparent",
                                                            stroke: "#784bff"
                                                        }}
                                                    />
                                                {/if}
                                            {/if}
                                        {/each}
                                    </Group>
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
                                            listening: false
                                        }}
                                    />
                                {/if}
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
                                        listening: false
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
                                        listening: false
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
                                                x: ev.detail.evt.clientX,
                                                y: 15
                                            };
                                            showColumnPicker =
                                                !showColumnPicker;
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
                                            listening: true,
                                            fill:
                                                columnToInsertIdx === null &&
                                                dropColumnIdx === idx &&
                                                dropColumnIdx !==
                                                    $draggedColumnIdx
                                                    ? DROP_HIGHLIGHT_BG_COLOR
                                                    : hoveredColumnIdx ===
                                                            idx ||
                                                        $query.orderBy ===
                                                            f.value
                                                      ? HEADER_BG_COLOR_HOVERED
                                                      : HEADER_BG_COLOR
                                        }}
                                    />
                                    {#if hoveredColumnIdx === idx}
                                        <Path
                                            config={{
                                                x: -2,
                                                y: 6,
                                                listening: false,
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
                                                    listening: false,
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
                                                    listening: false,
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
                                                f.value.match(
                                                    /^(track|duration)/
                                                ) !== null
                                                    ? 8
                                                    : 10,
                                            height: HEADER_HEIGHT,
                                            fontSize: 14,
                                            letterSpacing:
                                                f.value.match(/^(duration)/) !==
                                                null
                                                    ? -1
                                                    : 0,
                                            fontStyle: "bold",
                                            verticalAlign: "middle",
                                            fontFamily:
                                                "-apple-system, Avenir, Helvetica, Arial, sans-serif",
                                            fill: TEXT_COLOR,
                                            listening: false
                                        }}
                                    />
                                </Group>
                                {#if idx > 0}
                                    <Rect
                                        config={{
                                            x: f.viewProps.x - 1,
                                            y:
                                                sandwichTopHeight +
                                                HEADER_HEIGHT,
                                            height: viewportHeight,
                                            width: 0.5,
                                            fill: "rgba(242, 242, 242, 0.144)",
                                            listening: false
                                        }}
                                    />
                                {/if}
                            {/each}
                        </Layer>
                    </Stage>
                {/if}
                {#if $isSmartQueryBuilderOpen && noSongs}
                    <SmartQueryResultsPlaceholder />
                {/if}
            </div>
        </div>
    {/if}
</div>

<style lang="scss">
    .library-container {
        position: relative;
        height: 100%;
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        border-bottom-left-radius: 5px;
        border-bottom-right-radius: 5px;
        border-left: 0.7px solid #ffffff2a;
        border-bottom: 0.7px solid #ffffff2a;
        overflow: hidden;
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

    .scroll-now-playing {
        position: absolute;
        bottom: 0.5em;
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

    .smart-query {
        display: grid;
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
        background-color: #4d347c;
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
