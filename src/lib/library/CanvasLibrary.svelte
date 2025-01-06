<!-- SongDataGrid.svelte -->

<script lang="ts">
    import { liveQuery, type Observable } from "dexie";
    import hotkeys from "hotkeys-js";
    import { Layer as Lyr } from "konva/lib/Layer";
    import { Stage as Stg } from "konva/lib/Stage";
    import { Rect as Rct } from "konva/lib/shapes/Rect";
    import { debounce } from "lodash-es";
    import type { Song } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import {
        Group,
        type KonvaWheelEvent,
        Label,
        Layer,
        Path,
        Rect,
        Stage,
        Tag,
        Text,
        type KonvaDragTransformEvent,
    } from "svelte-konva";
    import { fly } from "svelte/transition";
    import { db } from "../../data/db";

    import Konva from "konva";
    import { Context } from "konva/lib/Context";
    import toast from "svelte-french-toast";
    import { reorderSongsInPlaylist } from "../../data/M3UUtils";
    import {
        arrowFocus,
        current,
        compressionSelected,
        draggedColumnIdx,
        draggedSongs,
        emptyDropEvent,
        fileDropHandler,
        forceRefreshLibrary,
        importStatus,
        isPlaying,
        isQueueOpen,
        isShuffleEnabled,
        isSidebarOpen,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isTagCloudOpen,
        libraryScrollPos,
        os,
        popupOpen,
        queriedSongs,
        query,
        queueMirrorsSearch,
        rightClickedTrack,
        rightClickedTracks,
        scrollToSong,
        selectedPlaylistFile,
        selectedTags,
        shouldFocusFind,
        singleKeyShortcutsEnabled,
        smartQuery,
        smartQueryInitiator,
        smartQueryResults,
        uiView,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { currentThemeObject } from "../../theming/store";
    import {
        moveArrayElement,
        swapArrayElements,
    } from "../../utils/ArrayUtils";
    import { timeSince } from "../../utils/DateUtils";
    import AudioPlayer from "../player/AudioPlayer";
    import { getQueryPart } from "../smart-query/QueryParts";
    import SmartQueryResultsPlaceholder from "../smart-query/SmartQueryResultsPlaceholder.svelte";
    import { UserQueryPart } from "../smart-query/UserQueryPart";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import ColumnPicker from "./ColumnPicker.svelte";
    import ImportPlaceholder from "./ImportPlaceholder.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    import Scrollbar from "../ui/Scrollbar.svelte";
    import { setQueue } from "../../data/storeHelper";
    import QueryResultsPlaceholder from "./QueryResultsPlaceholder.svelte";

    export let allSongs: Observable<Song[]> = null;
    export let columnOrder;
    export let dim = false;
    export let isInit = true;
    export let isLoading = false;
    export let theme = "default";

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
                    let timeSinceAdded = s.dateAdded
                        ? timeSince(s.dateAdded)
                        : "";
                    if (s.viewModel) {
                        s.viewModel.index = idx;
                        s.viewModel.timeSinceAdded = timeSinceAdded;
                    } else {
                        s.viewModel = {
                            index: idx,
                            timeSinceAdded,
                        };
                    }

                    if (s.tags) {
                        // console.log("songs.tags", s.tags);
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
                                    ...song.viewModel,
                                    isFirstAlbum: true,
                                    isFirstArtist: false,
                                };
                            }
                        }
                        status.state.firstSongInPreviousAlbum = idx;
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
                                    ...song.viewModel,
                                    isFirstAlbum: false,
                                    isFirstArtist: true,
                                };
                            }
                        }
                        status.state.firstSongInPreviousArtist = idx;
                    }
                    status.state.previousAlbum = s?.album;
                    status.state.previousArtist = s?.artist;

                    // If currently in album/shuffle/custom queue mode, then the current song index doesn't match the library index,
                    // and we need to find it
                    if (s.id === $current.song?.id) {
                        console.log("found song:", idx);
                        currentSongY = idx * ROW_HEIGHT;
                        currentSongScrollIdx = idx;
                    } else {
                        currentSongScrollIdx = null;
                    }

                    // Highlighted songs indexes might need to be updated
                    if (idx === songsArray.length - 1) {
                        if (songsHighlighted.length > 0) {
                            songsHighlighted = songsHighlighted.map((s) => {
                                s.viewModel.index = songsArray?.find(
                                    (song) => song.id === s.id,
                                )?.viewModel?.index;
                                highlightedSongIdx = s.viewModel.index;
                                return s;
                            });
                        }
                    }

                    return {
                        songs: songsArray,
                        state: status.state,
                    };
                },
                {
                    state: {
                        previousAlbum: null,
                        previousArtist: null,
                        firstSongInPreviousAlbum: null,
                        firstSongInPreviousArtist: null,
                    },
                    songs: [],
                },
            )?.songs ?? [];

    const DEFAULT_FIELDS = [
        {
            name: $LL.library.fields.title(),
            value: "title",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
        {
            name: $LL.library.fields.artist(),
            value: "artist",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
        {
            name: $LL.library.fields.composer(),
            value: "composer",
            show: false,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
        {
            name: $LL.library.fields.album(),
            value: "album",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
        {
            name: $LL.library.fields.albumArtist(),
            value: "albumArtist",
            show: true,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
        {
            name: $LL.library.fields.track(),
            value: "trackNumber",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false,
            },
        },
        {
            name: $LL.library.fields.dateAdded(),
            value: "dateAdded",
            displayValue: "viewModel.timeSinceAdded",
            show: true,
            viewProps: {
                width: 100,
                x: 0,
                autoWidth: false,
            },
        },
        {
            name: $LL.library.fields.compilation(),
            value: "compilation",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false,
            },
        },
        {
            name: $LL.library.fields.year(),
            value: "year",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false,
            },
        },
        {
            name: $LL.library.fields.genre(),
            value: "genre",
            show: true,
            viewProps: {
                width: 100,
                x: 0,
                autoWidth: false,
            },
        },
        {
            name: $LL.library.fields.origin(),
            value: "originCountry",
            show: false,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
        {
            name: $LL.library.fields.duration(),
            value: "duration",
            show: true,
            viewProps: {
                width: 63,
                x: 0,
                autoWidth: false,
            },
        },
        {
            name: $LL.library.fields.tags(),
            value: "tags",
            show: false,
            viewProps: {
                width: 0,
                x: 0,
                autoWidth: true,
            },
        },
    ];

    export let fields = DEFAULT_FIELDS;

    let hoveredColumnIdx = null;
    let hoveredSongIdx = null; // Index is slice-specific
    let columnToInsertIdx = null;
    let columnToInsertXPos = 0;
    let hoveredField = null;
    let hoveredTag = null;
    $: isOrderChanged =
        JSON.stringify(columnOrder) !==
        JSON.stringify(DEFAULT_FIELDS.map((f) => f.value));

    let showColumnPicker = false;
    let columnPickerPos;

    $: displayFields = fields.filter((f) => f.show);
    $: numColumns = fields.filter((f) => f.show).length;

    let songsSlice: Song[];
    let songsIdxSlice: number[];
    let songsStartSlice = 0;
    let songsEndSlice = 0;
    let canvas;

    let shouldRender = false;
    let ready = false; // When scroll position is restored (if available), to avoid jump
    let libraryContainer: HTMLDivElement;
    let scrollContainer: HTMLDivElement;
    let container: HTMLElement;
    let stage: Stg;
    let layer: Lyr;

    $: if (layer) {
        context = layer.getContext();
        context.font = "13.5px sans-serif";
        context.textAlign = "center";
    }

    let context: Context;
    let isScrollable = false;
    let prevScrollPos = 0;
    let scrollPos = 0;
    let scrollOffset = 0;
    let scrollNormalized = 0;

    let scrollbar: Rct;
    let scrollbarY = 0;
    let scrollbarHeight = 200;

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
    const TAG_PADDING = 10;
    const TAG_MARGIN = 5;

    // PLATFORM SPECIFIC
    const WINDOW_CONTROLS_WIDTH = 70;

    // COLORS
    let BG_COLOR: string;
    let HEADER_BG_COLOR: string;
    let HEADER_BG_COLOR_ACCENT: string;
    let HEADER_TEXT_COLOR: string;
    let OFFSCREEN_BG_COLOR: string;
    let HEADER_BG_COLOR_HOVERED: string;
    let TEXT_COLOR: string;
    let TEXT_COLOR_SECONDARY: string;
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

    const SCROLLING_PIXEL_RATIO = 1.3;
    const IDLE_PIXEL_RATIO = 1.8;

    onMount(() => {
        init();

        // Check for retina screens
        var query =
            "(-webkit-min-device-pixel-ratio: 2), (min-device-pixel-ratio: 2), (min-resolution: 192dpi)";

        if (matchMedia(query).matches) {
            // Slightly reduce the ratio for better performance
            Konva.pixelRatio = IDLE_PIXEL_RATIO;
        }

        const resizeObserver = new ResizeObserver((entries) => {
            // We're only watching one element
            const entry = entries.at(0);

            //Get the block size
            drawSongDataGrid(true);
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
    $: if (songs !== undefined && libraryContainer !== undefined) {
        console.log("Library::songs updated", songs.length);
        drawSongDataGrid();
        prevSongCount = songs.length;
    }

    $: if ($currentThemeObject) {
        // COLORS
        BG_COLOR = $currentThemeObject["panel-background"];
        HEADER_BG_COLOR = $currentThemeObject["library-header-bg"];
        HEADER_BG_COLOR_ACCENT = $currentThemeObject["accent-secondary"];
        HEADER_TEXT_COLOR = $currentThemeObject["library-header-text"];
        OFFSCREEN_BG_COLOR = "#71658e3b";
        HEADER_BG_COLOR_HOVERED =
            $currentThemeObject["library-header-active-bg"];
        TEXT_COLOR = $currentThemeObject["library-text-color"];
        TEXT_COLOR_SECONDARY = $currentThemeObject["text-secondary"];
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

    let isRestoringScrollPos = false;

    // Restore scroll position if any
    $: if (
        $uiView === "library" &&
        (isInit || $forceRefreshLibrary === true) &&
        ($libraryScrollPos !== null || $scrollToSong !== null) &&
        scrollContainer &&
        shouldRender
    ) {
        // console.log(
        //     "scrollpos",
        //     $libraryScrollPos,
        //     (songs?.length * ROW_HEIGHT - viewportHeight) * $libraryScrollPos
        // );
        isRestoringScrollPos = true;
        setTimeout(() => {
            if ($scrollToSong) {
                focusSong($scrollToSong);
                $scrollToSong = null;
            } else {
                onScroll(null, $libraryScrollPos, null, false, true);
            }
            ready = true;
        }, 50);
        isInit = false;
        $forceRefreshLibrary = false;
    } else if ($uiView.match(/^(playlists|to-delete)/)) {
        onScroll(null, 0, null, false, true);
        ready = true;
    } else if ($uiView.match(/^(smart-query|favourites)/)) {
        onScroll(null, 0, null, false, true);
        ready = true;
    } else if ($uiView.match(/^(albums)/)) {
        onScroll(null, 0, null, false, true);
        ready = true;
    }

    function drawSongDataGrid(isResize = false) {
        calculateCanvasSize();
        calculateColumns();
        if (isResize) {
            onScroll(null, scrollNormalized, null, true);
        } else {
            calculateSongSlice();
        }
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
        viewportHeight = libraryContainer.getBoundingClientRect().height;
        // console.log("contentHeight", contentHeight, "viewportHeight", viewportHeight);
        let area = contentHeight - viewportHeight;
        // console.log('scrollContainer.clientWidth', scrollContainer?.offsetWidth);
        width = scrollContainer?.clientWidth ?? libraryContainer.clientWidth;
        // Set canvas size to fill the parent
        virtualViewportHeight = viewportHeight;
        if (area < 0) {
            area = 0;
            virtualViewportHeight = viewportHeight;
        }

        scrollableArea = area;
        isScrollable = contentHeight > viewportHeight;
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

        const sortedFields = columnOrder.map((c) =>
            fields.find((f) => f.value === c),
        );

        /* Playlists show an additional file order column */
        if ($uiView === "playlists") {
            sortedFields.unshift({
                name: "none",
                value: "none",
                displayValue: "viewModel.index",
                operation: "+1",
                show: true,
                viewProps: {
                    width: 50,
                    x: 0,
                    autoWidth: false,
                },
            });
        }

        // Fields visible depending on window width
        const visibleFields = sortedFields.filter((f) => {
            switch (f.value) {
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
                case "dateAdded":
                    return width > 650;
                case "tags":
                    return width > 650;
                default:
                    return true;
            }
        });

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

    async function calculateSongSlice() {
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
            // let remainder = contentY % ROW_HEIGHT;
            // prevRemainder = remainder;
            // console.log("remainder", remainder);
            // scrollOffset = -contentY / 20;
            // console.log("scrollNormalized", scrollNormalized, "scrollableArea", scrollableArea);
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
            // console.log("slice indexes", songsStartSlice, songsEndSlice);
            // songsSlice = songs.slice(songsStartSlice, songsEndSlice);
            // console.log("songSlice", songsSlice);
            // Make sure the window is always filled with the right amount of rows
            // console.log("songIdxSlice", songsEndSlice - songsStartSlice);
            let newLength = songsEndSlice - songsStartSlice;
            if (songsIdxSlice?.length !== newLength) {
                songsIdxSlice = Array(newLength)
                    .fill(0)
                    .map((_, idx) => songsStartSlice + idx);
            } else {
                for (let i = 0; i < newLength; i++) {
                    songsIdxSlice[i] = songsStartSlice + i;
                }
            }

            // console.log("slice", songsIdxSlice);
            // console.log(songsSlice.length);
            prevScrollPos = scrollPos;
        }
    }

    const rememberScrollPos = debounce(async () => {
        // console.log("remembering scroll pos", scrollNormalized);
        $libraryScrollPos = scrollNormalized; // 0-1
    }, 100);

    let lastScrollTime = 0;
    let lastScrollTop = 0;
    let timeout;
    let isScrolling = false;
    let scrollingTimeout;

    // Change pixel ratio on scroll
    $: if (isScrolling) {
        // console.log("scrolling");
        Konva.pixelRatio = SCROLLING_PIXEL_RATIO;
        if (stage) {
            stage.getLayers().forEach((l) => {
                l.canvas.setPixelRatio(SCROLLING_PIXEL_RATIO);
            });
        }
    } else {
        // console.log("idle");
        Konva.pixelRatio = IDLE_PIXEL_RATIO;
        if (stage) {
            stage.getLayers().forEach((l) => {
                l.canvas.setPixelRatio(IDLE_PIXEL_RATIO);
                l.draw();
            });
        }
    }

    async function onScroll(
        e?: KonvaWheelEvent, // From mouse wheel
        scrollProgress?: number, // From scrollbar
        scrollPosY?: number, // From programmatic scroll (eg. restore pos)
        isResize = false,
        force = false,
    ) {
        if (!isScrollable) return;
        // console.log("onscroll", e, scrollProgress, scrollPosY);
        if (
            !force &&
            Date.now() - lastScrollTime < 50 &&
            Math.abs(lastScrollTop - scrollPos) > 500
        ) {
            clearTimeout(timeout);
            // console.log("throttled");
            timeout = setTimeout(() => {
                onScroll(e);
            }, 20);
            return;
        }

        let newScrollPos = 0;
        // From scrollbar
        if (scrollProgress != null) {
            scrollNormalized = scrollProgress;
            newScrollPos = Math.round(
                scrollNormalized *
                    (contentHeight - viewportHeight - HEADER_HEIGHT),
            );
            // console.log(
            //     "NEW SCROLL POS",
            //     newScrollPos,
            //     contentHeight - viewportHeight - HEADER_HEIGHT
            // );
        } else if (e) {
            // From mouse wheel
            newScrollPos = scrollPos + e.detail.evt.deltaY;
        } else if (scrollPosY != null) {
            // From programmatic scroll
            newScrollPos = scrollPosY;
        }
        // Handle boundaries
        if (newScrollPos < HEADER_HEIGHT) {
            newScrollPos = HEADER_HEIGHT;
        } else if (
            newScrollPos >
            contentHeight - viewportHeight - HEADER_HEIGHT
        ) {
            newScrollPos = contentHeight - viewportHeight - HEADER_HEIGHT;
        }

        if (newScrollPos === scrollPos) {
            return;
        }
        scrollPos = newScrollPos;
        if (e || (scrollPosY && force)) {
            scrollNormalized =
                scrollPos / (contentHeight - viewportHeight - HEADER_HEIGHT);
        }

        lastScrollTop = scrollPos;
        lastScrollTime = Date.now();

        if (e || force || isResize) {
            scrollbarY = scrollNormalized;
        }

        // console.log("scrollbarY", scrollbarY);

        if (scrollContainer && stage) {
            // let startTime = performance.now();
            calculateSongSlice();
            // console.log("took: ", performance.now() - startTime);
            let idx =
                currentSongScrollIdx !== null
                    ? currentSongScrollIdx
                    : $current.index;
            currentSongInView = idx >= songsStartSlice && idx <= songsEndSlice;
        }

        // Only save/restore scroll pos in main library view, not on playlists
        if ($uiView === "library" && !$isTagCloudOpen) {
            rememberScrollPos();
        }

        // Set isScrolling to true on scroll, then set it to false after 500ms
        if (!isRestoringScrollPos) {
            if (!isScrolling) {
                isScrolling = true;
            } else {
                scrollingTimeout && clearTimeout(scrollingTimeout);
            }
            scrollingTimeout = setTimeout(() => {
                isScrolling = false;
            }, 140);
        } else {
            isRestoringScrollPos = false;
        }
    }

    function currentSongIdxMatches() {
        return (
            $current.index < $allSongs?.length &&
            $allSongs[$current.index]?.id === $current.song?.id
        );
    }

    let currentSongY = 0;
    $: if (!$isShuffleEnabled) {
        let idx = currentSongIdxMatches()
            ? $current.index
            : $allSongs?.findIndex((s) => s.id === $current.song?.id);
        if (idx !== undefined) {
            currentSongY = idx * ROW_HEIGHT;
            currentSongInView = idx >= songsStartSlice && idx <= songsEndSlice;
        }
    }

    function scrollToCurrentSong() {
        console.log("y", currentSongY);

        let adjustedPos = currentSongY;
        if (currentSongY > viewportHeight / 2.3) {
            adjustedPos -= viewportHeight / 2.3;
        }
        if ($isPlaying && $current.song) {
            onScroll(null, null, adjustedPos, false, true);
        }
    }

    /**
     * Scrolls to and highlights song
     * @param song
     */
    function focusSong(song: Song) {
        let found;
        let idx = $allSongs?.findIndex((s) => {
            if (s.id === song.id) {
                found = s;
                return true;
            }
            return false;
        });
        if (idx !== undefined) {
            let y = idx * ROW_HEIGHT;

            if (y > viewportHeight / 2.3) {
                y -= viewportHeight / 2.3;
            }
            onScroll(null, null, y, false, true);
            highlightSong(found, found.viewModel.index, false);
        }
    }

    // LIBRARY FUNCTIONALITY

    let isShiftPressed = false;
    let rangeStartSongIdx = null;
    let rangeEndSongIdx = null;
    let highlightedSongIdx = 0;
    let showTrackMenu = false;
    let menuPos;
    let currentSongInView = false;
    let currentSongScrollIdx = null;

    async function onDoubleClickSong(song, idx) {
        if ($uiView.match(/^(albums)/)) {
            const albums = await db.albums
                .where("displayTitle")
                .equals(song.album)
                .filter(({ tracksIds }) => tracksIds.includes(song.id))
                .toArray();

            if (albums.length !== 1) {
                return;
            }

            const album = albums[0];

            let tracks = await db.songs
                .where("id")
                .anyOf(album.tracksIds)
                .toArray();

            tracks.sort((a, b) => {
                return a.trackNumber - b.trackNumber;
            });

            setQueue(tracks, song.viewModel.index);
        } else if ($uiView === "smart-query") {
            setQueue($smartQueryResults, song.viewModel.index);
        } else {
            setQueue($queriedSongs, song.viewModel.index);
        }

        if ($query.query.length) {
            $queueMirrorsSearch = true;
        }
    }

    function onRightClick(e, song, idx) {
        if (!songsHighlighted.includes(song)) {
            highlightSong(song, idx, false, false);
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

    /**
     * Highlight behaviour on query input:
     * - if a single song is highlighted, it will stay highlighted as you type (if still in results)
     * - if multiple songs are highlighted, they will be unhighlighted and the first song will be highlighted as you type
     */
    $: {
        if ($query.query?.length && $popupOpen !== "track-info") {
            if (songs?.length === 0) {
                resetHighlight();
            } else {
                highlightFirst();
            }
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
        isDefault = false,
    ) {
        // console.log("highlighted", song, idx, isKeyboardArrows, isDefault);
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
                // console.log("highlighted2", songsHighlighted);
            }
        } else if (
            (isKeyboardArrows && isShiftPressed) ||
            hotkeys.isPressed(91)
        ) {
            songsHighlighted.push(song);
            rangeStartSongIdx = idx;
            $rightClickedTrack = null;
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
            if ($popupOpen === "track-info" && isKeyboardArrows) {
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

    function resetHighlight() {
        songsHighlighted = [];
    }

    function highlightFirst() {
        // Only highlight first if previous highlight no longer exists after query update
        if (
            songsHighlighted.length === 0 ||
            (songsHighlighted.length === 1 &&
                !songs?.find((s) => s?.id === songsHighlighted[0]?.id)) ||
            songsHighlighted.length > 1
        ) {
            highlightSong(songs[0], 0, false, true);
        }
    }

    let draggingSongIdx = null;

    function onSongDragStart(song: Song, idx: number) {
        console.log("dragstart", idx);
        $arrowFocus = "library";
        // console.log("songshighlighted", songsHighlighted);
        if (songsHighlighted.length > 1) {
            $draggedSongs = songsHighlighted;
        } else {
            $draggedSongs = [song];
            if (idx !== undefined && $uiView === "playlists") {
                draggingSongIdx = idx;
            }
        }
    }

    async function onReorderSong(song: Song, idx: number) {
        if (draggingSongIdx !== null && $selectedPlaylistFile) {
            console.log("reorder song", idx);
            if (idx === draggingSongIdx) {
                draggingSongIdx = null;
                return;
            }

            if ($query.orderBy !== "none") {
                toast.error($LL.library.orderDisabledHint());
            }

            await reorderSongsInPlaylist(
                $selectedPlaylistFile,
                draggingSongIdx,
                idx,
            );
            $selectedPlaylistFile = $selectedPlaylistFile; // Trigger re-render
            draggingSongIdx = null;
        }
    }

    // Shortcuts

    hotkeys("esc", function (event, handler) {
        if (
            $popupOpen !== "track-info" &&
            $singleKeyShortcutsEnabled &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            if (showTrackMenu) {
                showTrackMenu = false;
            } else {
                songsHighlighted = [];
            }
        }
    });

    function onKeyDown(event) {
        if ($arrowFocus !== "library") return;

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
                toggleHighlight(
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
            event.keyCode === 84 &&
            $popupOpen !== "track-info" &&
            $singleKeyShortcutsEnabled &&
            (document.activeElement.id === "search" ||
                (document.activeElement.id !== "search" &&
                    document.activeElement.tagName.toLowerCase() !==
                        "input")) &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // 't' for tags/right-click menu
            event.preventDefault();
            console.log("active element", document.activeElement.tagName);
            // Check if there an input in focus currently
            if (!showTrackMenu && songsHighlighted.length) {
                console.log("opening info", songsHighlighted);
                if (songsHighlighted.length > 1) {
                    $rightClickedTracks = songsHighlighted;
                } else {
                    $rightClickedTrack = songsHighlighted[0];
                }

                const topTrack = songsHighlighted[0];
                // Get the y position of the top track by calculating the offset using the index in the slice
                const topTrackY =
                    stage
                        .findOne(
                            `#${topTrack.viewModel?.viewId ?? topTrack.id}`,
                        )
                        .getAbsolutePosition().y +
                    ROW_HEIGHT +
                    HEADER_HEIGHT +
                    10;
                console.log("top track y", topTrackY);
                menuPos = { x: 250, y: topTrackY };
                showTrackMenu = true;
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
            AudioPlayer.shouldPlay = true;
            setQueue($queriedSongs, highlightedSongIdx);

            if ($query.query.length) {
                $queueMirrorsSearch = true;
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
        if ($uiView === "library") {
            // This is used to restore the orderBy
            // when going back to the library from a playlist (custom order)
            $query.libraryOrderBy = newOrderBy;
        }
        $query = $query;
    }

    // Re-order columns

    $: {
        displayFields && columnOrder && calculateColumns();
    }

    let dropColumnIdx = null;

    function handleColumnDrag(
        pos: { x: number; y: number },
        index,
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
            (f) => x >= f.viewProps.x && x <= f.viewProps.x + f.viewProps.width,
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
            columnToInsertIdx = index > $draggedColumnIdx ? index - 1 : index;
            columnToInsertXPos = column.viewProps.x;
            // Add your logic here
        } else if (
            index !== $draggedColumnIdx - 1 &&
            index !== $draggedColumnIdx &&
            index < displayFields.length - 1 &&
            offsetX > elementWidth - dragZoneWidth
        ) {
            columnToInsertIdx = index > $draggedColumnIdx ? index : index + 1;
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
        console.log("insert column", oldIndex, newIndex);

        const oldIdxField = displayFields[oldIndex];
        const newIdxField = displayFields[newIndex];
        const columnOrderOldIdx = columnOrder.findIndex(
            (c) => c === oldIdxField.value,
        );
        const columnOrderNewIdx = columnOrder.findIndex(
            (c) => c === newIdxField.value,
        );
        columnOrder = moveArrayElement(
            columnOrder,
            columnOrderOldIdx,
            columnOrderNewIdx,
        );
        console.log("column order", columnOrder);
        // displayFields = moveArrayElement(displayFields, oldIndex, newIndex);
        resetColumnOrderUi();
    }

    function swapColumns(oldIndex, newIndex) {
        const oldIdxField = displayFields[oldIndex];
        const newIdxField = displayFields[newIndex];
        const columnOrderOldIdx = columnOrder.findIndex(
            (c) => c === oldIdxField.value,
        );
        const columnOrderNewIdx = columnOrder.findIndex(
            (c) => c === newIdxField.value,
        );
        console.log("swap column", columnOrderOldIdx, columnOrderNewIdx);
        columnOrder = swapArrayElements(
            columnOrder,
            columnOrderOldIdx,
            columnOrderNewIdx,
        );
        // displayFields = swapArrayElements(displayFields, oldIndex, newIndex);
        console.log("column order", columnOrder);
        resetColumnOrderUi();
    }

    // Sets back to default
    function resetColumnOrder() {
        fields = DEFAULT_FIELDS;
        columnOrder = DEFAULT_FIELDS.map((f) => f.value);
    }

    // SMART QUERY
    export let isSmartQueryEnabled = true; // Only for main view

    // Favourite

    async function favouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: true,
        });

        if ($current.song?.id === song.id) {
            $current.song.isFavourite = true;
        }
    }

    async function unfavouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: false,
        });
        if ($current.song?.id === song.id) {
            $current.song.isFavourite = false;
        }
    }

    function filterByField(fieldName: string, fieldValue: any) {
        let queryPart;
        console.log("filter", fieldName, fieldValue);
        $isTagCloudOpen = false;
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
            case "tags":
                queryPart = getQueryPart("CONTAINS_TAG");
                $smartQueryInitiator = "library-cell";
                break;
            default:
                return;
        }
        if ($uiView.match(/^(smart-query|favourites)/) === null) {
            $smartQuery.reset();
        }
        console.log("built in query Part", queryPart);
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

    /**
     * Supports nested props
     * @param song
     * @param field
     */
    function getValue(song, field) {
        if (field.displayValue) {
            let value = field.displayValue
                .split(".")
                .reduce((acc, key) => acc && acc[key], song);
            if (field.operation) {
                eval(`value = value ${field.operation}`);
            }
            return value;
        }
        return song[field.value];
    }

    function getTagOffset(tags, idx) {
        if (idx === 0) {
            return 0;
        }
        let offset = 0;
        for (let i = 0; i < idx; i++) {
            offset += (context?.measureText(tags[i])?.width ?? 0) + 30;
        }
        return offset;
    }

    interface Tag {
        name: string;
        viewProps: {
            x: number;
        };
    }

    interface SongTags {
        visible: Tag[];
        hidden: string[];
        hiddenLabelOffset: number;
    }

    async function getSongTags(
        song: Song,
        fieldWidth: number,
    ): Promise<SongTags> {
        let offset = TAG_MARGIN;
        let hiddenLabelWidth = 0;

        return song.tags.reduce(
            (acc, tag, idx) => {
                const tagWidth =
                    context?.measureText(tag)?.width + (1.5 * tag.length + 1) ??
                    0;
                // console.log("tagWidth", tagWidth);
                const newOffset =
                    offset + tagWidth + TAG_PADDING * 2 + TAG_MARGIN;
                let newHiddenCount = acc.hidden.length + 1;
                hiddenLabelWidth =
                    context.measureText(`+${newHiddenCount}`)?.width +
                        TAG_PADDING * 2 +
                        TAG_MARGIN ?? 0;

                if (newOffset + hiddenLabelWidth > fieldWidth) {
                    acc.hidden.push(tag);
                } else {
                    acc.visible.push({ name: tag, viewProps: { x: offset } });
                    offset = newOffset;
                    // console.log("offset", tag, offset);
                    acc.hiddenLabelOffset = offset;
                }
                return acc;
            },
            {
                visible: [],
                hidden: [],
                hiddenLabelOffset: 0,
            },
        );
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
    bind:columnOrder
    onResetOrder={resetColumnOrder}
    {isOrderChanged}
/>

<div class="library-container" bind:this={libraryContainer}>
    {#if isLoading}
        <!-- <div class="loading" out:fade={{ duration: 90, easing: cubicInOut }}>
            <p>💿 one sec...</p>
        </div> -->
    {:else if theme === "default" && (($importStatus.isImporting && $importStatus.backgroundImport === false) || (noSongs && $query.query.length === 0 && $uiView.match(/^(smart-query|favourites|to-delete)/) === null && $isTagCloudOpen === false))}
        <ImportPlaceholder />
    {:else}
        <div
            id="scroll-container"
            style="overflow-y: {false ? 'visible' : 'hidden'}"
            class:ready
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

                {#if shouldRender}
                    <Stage
                        config={{
                            width,
                            height: viewportHeight,
                            y: -sandwichTopHeight,
                        }}
                        bind:handle={stage}
                        on:wheel={onScroll}
                    >
                        <!-- SONGS LIST -->
                        <Layer
                            bind:handle={layer}
                            config={{
                                x: 0,
                                y: HEADER_HEIGHT,
                            }}
                        >
                            <Rect
                                config={{
                                    x: 0,
                                    y: sandwichTopHeight,
                                    width,
                                    height: viewportHeight,
                                    fill: BG_COLOR,
                                }}
                            />

                            {#if songsIdxSlice?.length && songs?.length}
                                {#each songsIdxSlice as idx, songIdx (songs[idx]?.viewModel?.viewId ?? songs[idx]?.id)}
                                    {@const song = songs[idx]}
                                    <!-- {@debug idx} -->
                                    <!-- {@debug song} -->
                                    <Group
                                        on:dblclick={() =>
                                            onDoubleClickSong(song, songIdx)}
                                        on:click={(e) => {
                                            // console.log("e", e);
                                            if (e.detail.evt.button === 0) {
                                                toggleHighlight(
                                                    song,
                                                    song.viewModel.index,
                                                );
                                            } else if (
                                                e.detail.evt.button === 2
                                            ) {
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
                                                draggingSongIdx !== null &&
                                                songIdx > songsSlice.length - 15
                                            ) {
                                                scrollContainer?.scrollBy({
                                                    top: ROW_HEIGHT,
                                                });
                                            } else if (
                                                draggingSongIdx !== null &&
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
                                            onSongDragStart(
                                                song,
                                                song.viewModel?.index,
                                            )}
                                        config={{
                                            visible: !song.dummy,
                                        }}
                                        on:mouseup={(e) => {
                                            onReorderSong(
                                                song,
                                                song.viewModel.index,
                                            );
                                        }}
                                    >
                                        <Rect
                                            config={{
                                                id:
                                                    song.viewModel?.viewId ??
                                                    song?.id,
                                                x: 0,
                                                y:
                                                    sandwichTopHeight +
                                                    ROW_HEIGHT * songIdx +
                                                    scrollOffset,
                                                width: width,
                                                height: ROW_HEIGHT,
                                                listening: true,
                                                fill:
                                                    draggingSongIdx ===
                                                    song.viewModel?.index
                                                        ? DRAGGING_SOURCE_COLOR
                                                        : $current.song?.id ===
                                                            song?.id
                                                          ? PLAYING_BG_COLOR
                                                          : songsHighlighted &&
                                                              isSongHighlighted(
                                                                  song,
                                                              )
                                                            ? HIGHLIGHT_BG_COLOR
                                                            : hoveredSongIdx ===
                                                                songIdx
                                                              ? ROW_BG_COLOR_HOVERED
                                                              : ROW_BG_COLOR,
                                            }}
                                        />
                                        {#each displayFields as f, idx (f.value)}
                                            <!-- Smart query fields -->
                                            {#if f.value.match(/^(year|genre|originCountry)/) !== null && !isInvalidValue(getValue(song, f))}
                                                <Label
                                                    config={{
                                                        x: f.viewProps.x + 5,
                                                        y:
                                                            sandwichTopHeight +
                                                            ROW_HEIGHT *
                                                                songIdx +
                                                            2.5,
                                                        width:
                                                            f.viewProps.width -
                                                            10,
                                                        height: ROW_HEIGHT - 5,
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
                                                            getValue(song, f),
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
                                                            cornerRadius: 2,
                                                        }}
                                                    />
                                                    <Text
                                                        config={{
                                                            text: validatedValue(
                                                                getValue(
                                                                    song,
                                                                    f,
                                                                ),
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
                                                                $current.song
                                                                    ?.id ===
                                                                song.id
                                                                    ? PLAYING_TEXT_COLOR
                                                                    : TEXT_COLOR,
                                                            ellipsis:
                                                                f.value.match(
                                                                    /^(title|artist|album|genre)/,
                                                                ) !== null,
                                                        }}
                                                    />
                                                </Label>
                                            {:else if f.value.match(/^(tags)/) !== null && !isInvalidValue(getValue(song, f))}
                                                {#if song.tags}
                                                    {#await getSongTags(song, f.viewProps.width) then tags}
                                                        {#each tags.visible as tag, idx (tag)}
                                                            <Label
                                                                config={{
                                                                    x:
                                                                        f
                                                                            .viewProps
                                                                            .x +
                                                                        tag
                                                                            .viewProps
                                                                            .x,
                                                                    y:
                                                                        sandwichTopHeight +
                                                                        ROW_HEIGHT *
                                                                            songIdx +
                                                                        scrollOffset +
                                                                        2.5,
                                                                    height:
                                                                        ROW_HEIGHT -
                                                                        5,
                                                                }}
                                                                on:mouseenter={() => {
                                                                    hoveredField =
                                                                        f.value;
                                                                    hoveredTag =
                                                                        tag;
                                                                }}
                                                                on:mouseleave={() => {
                                                                    hoveredField =
                                                                        null;
                                                                    hoveredTag =
                                                                        null;
                                                                }}
                                                                on:click={() => {
                                                                    $isTagCloudOpen = true;
                                                                    $isSmartQueryBuilderOpen = false;
                                                                    $uiView =
                                                                        "library";
                                                                    $selectedTags.add(
                                                                        tag.name,
                                                                    );
                                                                    $selectedTags =
                                                                        $selectedTags;
                                                                }}
                                                            >
                                                                <Tag
                                                                    config={{
                                                                        fill:
                                                                            hoveredSongIdx ===
                                                                                songIdx &&
                                                                            hoveredField ===
                                                                                f.value &&
                                                                            hoveredTag ===
                                                                                tag
                                                                                ? CLICKABLE_CELL_BG_COLOR_HOVERED
                                                                                : CLICKABLE_CELL_BG_COLOR,
                                                                        cornerRadius: 10,
                                                                    }}
                                                                />
                                                                <Text
                                                                    config={{
                                                                        text: validatedValue(
                                                                            tag.name,
                                                                        ),
                                                                        listening: false,
                                                                        y: 0,
                                                                        x: 0,
                                                                        padding:
                                                                            TAG_PADDING,
                                                                        height:
                                                                            ROW_HEIGHT -
                                                                            5,
                                                                        align: "center",
                                                                        fontSize: 13.5,
                                                                        verticalAlign:
                                                                            "middle",
                                                                        fill:
                                                                            $current
                                                                                .song
                                                                                ?.id ===
                                                                            song.id
                                                                                ? PLAYING_TEXT_COLOR
                                                                                : TEXT_COLOR,
                                                                        ellipsis:
                                                                            f.value.match(
                                                                                /^(title|artist|album|genre)/,
                                                                            ) !==
                                                                            null,
                                                                    }}
                                                                />
                                                            </Label>
                                                        {/each}
                                                        {#if tags.hidden?.length}
                                                            <Label
                                                                config={{
                                                                    x:
                                                                        f
                                                                            .viewProps
                                                                            .x +
                                                                        tags.hiddenLabelOffset,
                                                                    y:
                                                                        sandwichTopHeight +
                                                                        ROW_HEIGHT *
                                                                            songIdx +
                                                                        scrollOffset +
                                                                        2.5,
                                                                    height:
                                                                        ROW_HEIGHT -
                                                                        5,
                                                                }}
                                                                on:mouseenter={() => {
                                                                    hoveredField =
                                                                        f.value;
                                                                    hoveredTag =
                                                                        "+overflow";
                                                                }}
                                                                on:mouseleave={() => {
                                                                    hoveredField =
                                                                        null;
                                                                    hoveredTag =
                                                                        null;
                                                                }}
                                                                on:click={(
                                                                    e,
                                                                ) => {
                                                                    // TODO: Show overflowed tags in menu
                                                                    menuPos = {
                                                                        x: e
                                                                            .detail
                                                                            .evt
                                                                            .clientX,
                                                                        y: e
                                                                            .detail
                                                                            .evt
                                                                            .clientY,
                                                                    };
                                                                    $rightClickedTrack =
                                                                        song;
                                                                    showTrackMenu = true;
                                                                }}
                                                            >
                                                                <Tag
                                                                    config={{
                                                                        fill:
                                                                            hoveredSongIdx ===
                                                                                songIdx &&
                                                                            hoveredField ===
                                                                                f.value &&
                                                                            hoveredTag ===
                                                                                "+overflow"
                                                                                ? CLICKABLE_CELL_BG_COLOR_HOVERED
                                                                                : CLICKABLE_CELL_BG_COLOR,
                                                                        cornerRadius: 10,
                                                                    }}
                                                                />
                                                                <Text
                                                                    config={{
                                                                        text: validatedValue(
                                                                            `+${tags.hidden.length}`,
                                                                        ),
                                                                        listening: false,
                                                                        y: 0,
                                                                        x: 0,
                                                                        padding: 10,
                                                                        height:
                                                                            ROW_HEIGHT -
                                                                            5,
                                                                        align: "center",
                                                                        fontSize: 13.5,
                                                                        verticalAlign:
                                                                            "middle",
                                                                        fill:
                                                                            $current
                                                                                .song
                                                                                ?.id ===
                                                                            song.id
                                                                                ? PLAYING_TEXT_COLOR
                                                                                : TEXT_COLOR,
                                                                        ellipsis:
                                                                            f.value.match(
                                                                                /^(title|artist|album|genre)/,
                                                                            ) !==
                                                                            null,
                                                                    }}
                                                                />
                                                            </Label>
                                                        {/if}
                                                    {/await}
                                                {/if}
                                            {:else}
                                                <Text
                                                    config={{
                                                        x:
                                                            f.value.match(
                                                                /^(title|artist|album|track)/,
                                                            ) !== null
                                                                ? f.viewProps
                                                                      .x + 10
                                                                : f.viewProps.x,
                                                        y:
                                                            sandwichTopHeight +
                                                            ROW_HEIGHT *
                                                                songIdx +
                                                            1,
                                                        text: validatedValue(
                                                            getValue(song, f),
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
                                                            displayFields.length -
                                                                1
                                                                ? f.viewProps
                                                                      .width -
                                                                  10
                                                                : f.value.match(
                                                                        /^(title|artist|album|track)/,
                                                                    ) !== null
                                                                  ? f.value ===
                                                                        "title" &&
                                                                    $current
                                                                        .song
                                                                        ?.id ===
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
                                                                /^(genre)/,
                                                            ) !== null
                                                                ? 10
                                                                : 2,
                                                        height: HEADER_HEIGHT,
                                                        fontSize: 13.5,
                                                        verticalAlign: "middle",
                                                        fill:
                                                            $current.song
                                                                ?.id === song.id
                                                                ? PLAYING_TEXT_COLOR
                                                                : f.name ===
                                                                    "none"
                                                                  ? TEXT_COLOR_SECONDARY
                                                                  : TEXT_COLOR,
                                                        ellipsis:
                                                            f.value.match(
                                                                /^(title|artist|album|genre)/,
                                                            ) !== null,
                                                    }}
                                                />
                                            {/if}
                                            <!-- Drag handle icon-->
                                            {#if hoveredSongIdx === songIdx && !(draggingSongIdx !== null || (draggingSongIdx === null && $draggedSongs?.length))}
                                                <Path
                                                    config={{
                                                        x: -2,
                                                        y:
                                                            sandwichTopHeight +
                                                            ROW_HEIGHT *
                                                                songIdx +
                                                            6,
                                                        listening: false,
                                                        scaleX: 0.9,
                                                        scaleY: 0.9,
                                                        data: "M7.375 3.67c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m0 8.66c0-.646-.56-1.17-1.25-1.17s-1.25.524-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17m-1.25-5.5c.69 0 1.25.525 1.25 1.17c0 .645-.56 1.17-1.25 1.17S4.875 8.645 4.875 8c0-.645.56-1.17 1.25-1.17m5-3.16c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m-1.25 7.49c.69 0 1.25.524 1.25 1.17c0 .645-.56 1.17-1.25 1.17s-1.25-.525-1.25-1.17c0-.646.56-1.17 1.25-1.17M11.125 8c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17",
                                                        fill: "rgba(255, 255, 255, 0.05)",
                                                    }}
                                                />
                                            {/if}
                                            {#if f.value === "title"}
                                                <!-- Now playing icon -->
                                                {#if $current.song?.id === song.id}
                                                    <Path
                                                        config={{
                                                            x:
                                                                f.viewProps.x +
                                                                f.viewProps
                                                                    .width -
                                                                40,
                                                            y:
                                                                sandwichTopHeight +
                                                                ROW_HEIGHT *
                                                                    songIdx +
                                                                scrollOffset +
                                                                7,
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
                                                                f.viewProps.x +
                                                                f.viewProps
                                                                    .width -
                                                                20,
                                                            y:
                                                                sandwichTopHeight +
                                                                ROW_HEIGHT *
                                                                    songIdx +
                                                                scrollOffset +
                                                                6,
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
                                                                f.viewProps.x +
                                                                f.viewProps
                                                                    .width -
                                                                20,
                                                            y:
                                                                sandwichTopHeight +
                                                                ROW_HEIGHT *
                                                                    songIdx +
                                                                scrollOffset +
                                                                6,
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

                                    {#if hoveredSongIdx === songIdx && draggingSongIdx !== null}
                                        <Rect
                                            config={{
                                                x: 0,
                                                y:
                                                    sandwichTopHeight +
                                                    ROW_HEIGHT * songIdx +
                                                    scrollOffset +
                                                    (song.viewModel?.index >
                                                    draggingSongIdx
                                                        ? ROW_HEIGHT
                                                        : 0),
                                                width: width,
                                                height: DROP_HINT_HEIGHT,
                                                fill: PLAYING_BG_COLOR,
                                                listening: false,
                                            }}
                                        />
                                    {/if}
                                {/each}
                            {/if}
                        </Layer>
                        <!-- COLUMN HEADERS -->
                        <Layer>
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
                            {#each displayFields as f, idx (f.value)}
                                <Group
                                    config={{
                                        x: f.viewProps.x,
                                        y: sandwichTopHeight,
                                        width: f.viewProps.width,
                                        draggable: true,
                                        dragBoundFunc(pos) {
                                            return handleColumnDrag(pos, idx);
                                        },
                                    }}
                                    on:click={(ev) => {
                                        if (ev.detail.evt.button === 0)
                                            updateOrderBy(f.value);
                                        else if (ev.detail.evt.button === 2) {
                                            if ($uiView.match(/^(albums)/)) {
                                                columnPickerPos = {
                                                    x: ev.detail.evt.clientX,
                                                    y: ev.detail.evt.clientY,
                                                };
                                            } else {
                                                columnPickerPos = {
                                                    x: ev.detail.evt.clientX,
                                                    y: 15,
                                                };
                                            }
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
                                                    : f.name === "none" &&
                                                        $query.orderBy ===
                                                            "none"
                                                      ? HEADER_BG_COLOR_ACCENT
                                                      : hoveredColumnIdx ===
                                                              idx ||
                                                          $query.orderBy ===
                                                              f.value
                                                        ? HEADER_BG_COLOR_HOVERED
                                                        : HEADER_BG_COLOR,
                                        }}
                                    />
                                    {#if hoveredColumnIdx === idx}
                                        <Path
                                            config={{
                                                x: -2,
                                                y: 4,
                                                listening: false,
                                                scaleX: 0.9,
                                                scaleY: 0.9,
                                                data: "M7.375 3.67c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m0 8.66c0-.646-.56-1.17-1.25-1.17s-1.25.524-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17m-1.25-5.5c.69 0 1.25.525 1.25 1.17c0 .645-.56 1.17-1.25 1.17S4.875 8.645 4.875 8c0-.645.56-1.17 1.25-1.17m5-3.16c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .646.56 1.17 1.25 1.17s1.25-.524 1.25-1.17m-1.25 7.49c.69 0 1.25.524 1.25 1.17c0 .645-.56 1.17-1.25 1.17s-1.25-.525-1.25-1.17c0-.646.56-1.17 1.25-1.17M11.125 8c0-.645-.56-1.17-1.25-1.17s-1.25.525-1.25 1.17c0 .645.56 1.17 1.25 1.17s1.25-.525 1.25-1.17",
                                                fill: "rgba(255, 255, 255, 0.5)",
                                            }}
                                        />

                                        <!-- File order tooltip -->
                                        {#if f.name === "none"}
                                            <Rect
                                                config={{
                                                    x: 0,
                                                    y: 25,
                                                    width: 120,
                                                    height: 20,
                                                    listening: false,
                                                    fill: "#212121d5",
                                                    cornerRadius: 4,
                                                }}
                                            />
                                            <Text
                                                config={{
                                                    x: 5,
                                                    y: 27,
                                                    text: $LL.library.resetToFileOrder(),
                                                    fontSize: 14,
                                                    fill: "rgba(255, 255, 255)",
                                                }}
                                            />
                                        {/if}
                                    {/if}

                                    <!-- Sort arrow icons -->
                                    {#if $query.orderBy === f.value || ($query.orderBy === "none" && f.name === "none")}
                                        {#if $query.reverse}
                                            <Path
                                                config={{
                                                    x: f.viewProps.width - 16,
                                                    y: 4,
                                                    listening: false,
                                                    scaleX: 0.6,
                                                    scaleY: 0.6,
                                                    data: "m7.293 8.293l3.995-4a1 1 0 0 1 1.32-.084l.094.083l4.006 4a1 1 0 0 1-1.32 1.499l-.094-.083l-2.293-2.291v11.584a1 1 0 0 1-.883.993L12 20a1 1 0 0 1-.993-.884L11 19.001V7.41L8.707 9.707a1 1 0 0 1-1.32.084l-.094-.084a1 1 0 0 1-.084-1.32zl3.995-4z",
                                                    fill: "rgba(255, 255, 255, 0.8)",
                                                }}
                                            />
                                        {:else}
                                            <Path
                                                config={{
                                                    x: f.viewProps.width - 16,
                                                    y: 4,
                                                    listening: false,
                                                    scaleX: 0.6,
                                                    scaleY: 0.6,
                                                    data: "M11.883 4.01L12 4.005a1 1 0 0 1 .993.883l.007.117v11.584l2.293-2.294a1 1 0 0 1 1.32-.084l.094.083a1 1 0 0 1 .084 1.32l-.084.095l-3.996 4a1 1 0 0 1-1.32.083l-.094-.083l-4.004-4a1 1 0 0 1 1.32-1.498l.094.083L11 16.583V5.004a1 1 0 0 1 .883-.992L12 4.004z",
                                                    fill: "rgba(255, 255, 255, 0.8)",
                                                }}
                                            />
                                        {/if}
                                    {/if}

                                    <!-- File icon -->
                                    {#if f.name === "none"}
                                        <Path
                                            config={{
                                                x: f.viewProps.x + 19,
                                                y: 6,
                                                listening: false,
                                                scaleX: 0.6,
                                                scaleY: 0.6,
                                                data: "M9.5 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V4.5zm0 1v2A1.5 1.5 0 0 0 11 4.5h2V14a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1z",
                                                fill: "rgba(255, 255, 255, 0.8)",
                                            }}
                                        />
                                    {/if}
                                    <Text
                                        config={{
                                            x:
                                                !$isSidebarOpen &&
                                                !$isQueueOpen &&
                                                $os === "macos" &&
                                                idx === 0
                                                    ? WINDOW_CONTROLS_WIDTH
                                                    : null,
                                            text:
                                                f.name === "none" ? "" : f.name,
                                            align: "left",
                                            padding:
                                                f.value.match(
                                                    /^(track|duration)/,
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
                                            fill: HEADER_TEXT_COLOR,
                                            listening: false,
                                            visible: !(
                                                !$isSidebarOpen &&
                                                !$isQueueOpen &&
                                                $os === "macos" &&
                                                idx === 0
                                            ),
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
                                            listening: false,
                                        }}
                                    />
                                {/if}
                            {/each}
                        </Layer>
                    </Stage>
                    {#if isScrollable}
                        <Scrollbar
                            onScroll={(s) => {
                                onScroll(null, s);
                            }}
                            height={virtualViewportHeight}
                            yPercent={scrollbarY}
                            topPadding={HEADER_HEIGHT}
                        />
                    {/if}
                {/if}
                {#if $isSmartQueryBuilderOpen && noSongs}
                    <SmartQueryResultsPlaceholder />
                {/if}
                {#if $query.query?.length && noSongs}
                    <QueryResultsPlaceholder />
                {/if}
            </div>
        </div>
    {/if}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    {#if $uiView === "library" && $isPlaying && $current.song && !currentSongInView}
        <div
            in:fly={{ duration: 150, y: 30 }}
            out:fly={{ duration: 150, y: 30 }}
            class="scroll-now-playing"
            class:light={$currentThemeObject.type === "light"}
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
    <ShadowGradient type="bottom" />
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
        border-left: 0.7px solid var(--panel-primary-border-main);
        border-bottom: 0.7px solid var(--panel-primary-border-main);
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
        background-color: color-mix(
            in srgb,
            var(--panel-background) 80%,
            var(--type-bw)
        );
        border: 1px solid
            color-mix(in srgb, var(--panel-background) 80%, var(--inverse));
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

        &.light {
            background-color: #d9d9e0;
            color: var(--text);
            border: 1px solid rgba(163, 158, 158, 0.497);
            box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.334);

            &:hover {
                background-color: #d2d2dd;
                border: 1px solid rgba(101, 98, 98, 0.31);
                box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.534);
            }
            &:active {
                background-color: #c0c0ca;
                border: 2px solid rgba(101, 98, 98, 0.262);
                box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.534);
            }
            .eq {
                span {
                    background-color: #a0a0a0;
                }
            }
        }
        @media only screen and (max-width: 522px) {
            display: none;
        }
        &:hover {
            background-color: color-mix(
                in srgb,
                var(--panel-background) 70%,
                var(--type-bw)
            );
            border: 1px solid
                color-mix(in srgb, var(--panel-background) 60%, var(--inverse));
            box-shadow: 10px 10px 10px rgba(31, 31, 31, 0.934);
        }
        &:active {
            background-color: color-mix(
                in srgb,
                var(--panel-background) 70%,
                var(--type-bw)
            );
            border: 1px solid
                color-mix(in srgb, var(--panel-background) 60%, var(--inverse));
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
    }

    .tag-cloud {
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
