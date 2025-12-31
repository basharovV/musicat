<!-- SongDataGrid.svelte -->

<script lang="ts">
    import { liveQuery, type Observable } from "dexie";
    import hotkeys from "hotkeys-js";
    import { Layer as Lyr } from "konva/lib/Layer";
    import { Stage as Stg } from "konva/lib/Stage";
    import { Rect as Rct } from "konva/lib/shapes/Rect";
    import { cloneDeep, debounce } from "lodash-es";
    import type {
        ColumnViewModel,
        LibraryColumn,
        Query,
        Song,
        SongOrder,
    } from "src/App";
    import { onDestroy, onMount, tick } from "svelte";
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
        type KonvaMouseEvent,
    } from "svelte-konva";
    import { db } from "../../data/db";

    import Konva from "konva";
    import { Context } from "konva/lib/Context";
    import toast from "svelte-french-toast";
    import {
        addSongsToPlaylist,
        insertSongsToPlaylist,
        reorderSongsInPlaylist,
    } from "../../data/M3UUtils";
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
        isSidebarShowing,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isTagCloudOpen,
        libraryScrollPos,
        os,
        popupOpen,
        queriedSongs,
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
        rightClickedAlbum,
        expandedSongWithStems,
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
    import {
        resetDraggedSongs,
        setDraggedSongs,
        setQueue,
    } from "../../data/storeHelper";
    import QueryResultsPlaceholder from "./QueryResultsPlaceholder.svelte";
    import ScrollTo from "../ui/ScrollTo.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import SongHighlighter from "./SongHighlighter.svelte";
    import { isInputFocused } from "../../utils/ActiveElementUtils";
    import type { PersistentWritable } from "../../data/storeUtils";
    import { getAllColumns } from "./LibraryColumns";
    import { contextMenu, openContextMenu } from "../ui/ContextMenu";
    import StemsDropdown from "./StemsDropdown.svelte";
    import type { Readable } from "svelte/store";

    export let songsReadable: Readable<Song[]> | Observable<Song[]> = null;
    export let songsArray: Song[] = [];
    export let columnOrder: PersistentWritable<LibraryColumn[]>;
    export let dim = false;
    export let isInit = true;
    export let isLoading = false;
    export let query: Query = null;
    export let songOrder: SongOrder = null;
    export let theme = "default";

    $: allSongs = $songsReadable ?? songsArray;

    $: songs =
        allSongs
            ?.filter((song: Song) => {
                if ($compressionSelected === "lossless") {
                    return song?.fileInfo?.lossless;
                } else if ($compressionSelected === "lossy") {
                    return song?.fileInfo?.lossless === false;
                } else return song !== undefined;
            })
            .reduce(
                (status, s, idx, songsArray: any[]) => {
                    if (status?.songs?.length === 0) {
                        status.songs = songsArray;
                    }

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

                    // Putting this in a separate function to avoid the reactivity loop
                    if (idx === status.songs.length - 1) {
                        // Run once on last song
                        updateHighlights(status.songs, idx);
                    }

                    return {
                        songs:
                            status.songs?.length > 0
                                ? status.songs
                                : songsArray,
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

    let hoveredColumnIdx = null;
    let hoveredSongIdx = null; // Index is slice-specific
    let columnToInsertIdx = null;
    let columnToInsertXPos = 0;
    let hoveredField = null;
    let hoveredTag = null;
    $: isOrderChanged =
        JSON.stringify($columnOrder.map((c) => c.fieldName)) !==
        JSON.stringify(displayFields.map((f) => f.value));

    let showColumnPicker = false;
    let columnPickerIndex;
    let columnPickerPos;

    let displayFields = [];

    let songsSlice: Song[];
    let songsIdxSlice: number[];

    // Compute it as a derived value
    $: {
        const start = Math.max(0, songsStartSlice);
        const end = Math.min(songs?.length ?? 0, songsEndSlice);
        const length = Math.max(0, end - start);

        songsIdxSlice =
            songs?.length && length > 0 && length <= songs.length
                ? Array(length)
                      .fill(0)
                      .map((_, idx) => start + idx)
                : [];
    }

    let songsStartSlice = 0;
    let songsEndSlice = 0;
    let canvas;

    export let shouldRender = false;
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

    // if cursor over the library
    let isOver = false;

    // drag-n-drop
    let isDraggingOver = false;

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
    const RESIZE_HANDLE_WIDTH = 5; // half on each column edge

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
    let ROW_ODD_BG_COLOR: string;
    let ROW_BG_COLOR_HOVERED: string;
    let PLAYING_BG_COLOR: string;
    let PLAYING_TEXT_COLOR: string;
    let COLUMN_INSERT_HINT_COLOR: string;
    let DROP_HIGHLIGHT_BG_COLOR: string;
    let CLICKABLE_CELL_BG_COLOR: string;
    let CLICKABLE_CELL_BG_COLOR_HOVERED: string;
    let DRAGGING_SOURCE_COLOR: string;
    let COLUMN_DIVIDER_COLOR: string;
    let COLUMN_RESIZE_DIVIDER_COLOR: string;

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

        const resizeObserver = new ResizeObserver((_) => {
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

    $: noSongs =
        !songs ||
        songs.length === 0 ||
        ($uiView.match(/^(smart-query|favourites)/) &&
            $isSmartQueryBuilderOpen &&
            $smartQuery.isEmpty);

    // Trigger: on songs updated
    $: if (songs && libraryContainer && prevSongCount !== songs.length) {
        drawSongDataGrid();
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
        TEXT_COLOR = $currentThemeObject["library-text"];
        TEXT_COLOR_SECONDARY = $currentThemeObject["text-secondary"];
        HIGHLIGHT_BG_COLOR = $currentThemeObject["library-highlight-bg"];
        ROW_BG_COLOR = "transparent";
        ROW_ODD_BG_COLOR = $currentThemeObject["library-odd-row-bg"];
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
        COLUMN_DIVIDER_COLOR = $currentThemeObject["library-column-divider"];
        COLUMN_RESIZE_DIVIDER_COLOR = $currentThemeObject["accent-secondary"];
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
        isInit = false;
    } else if ($uiView.match(/^(smart-query|favourites)/)) {
        onScroll(null, 0, null, false, true);
        ready = true;
        isInit = false;
    } else if ($uiView.match(/^(albums)/)) {
        onScroll(null, 0, null, false, true);
        ready = true;
        isInit = false;
    }

    async function drawSongDataGrid(isResize: boolean = false) {
        await calculateCanvasSize(isResize);
        calculateColumns();
        if (isResize) {
            onScroll(null, scrollNormalized, null, true);
        } else {
            calculateSongSlice();
        }
        shouldRender = true;
        // drawHeaders();
        // drawRows();
        prevSongCount = songs.length;
    }

    function calculateViewport() {
        viewportHeight = libraryContainer.clientHeight;
        let area = contentHeight - viewportHeight;
        width = scrollContainer?.clientWidth ?? libraryContainer.clientWidth;
        // Set canvas size to fill the parent
        virtualViewportHeight = viewportHeight;
        if (area < 0) {
            area = 0;
            virtualViewportHeight = viewportHeight;
        }

        scrollableArea = area;
        isScrollable = contentHeight > viewportHeight;
    }

    async function calculateCanvasSize(isResize: boolean) {
        contentHeight = HEADER_HEIGHT + songs?.length * ROW_HEIGHT;
        calculateViewport();
        if (!isResize) {
            await new Promise((resolve) => {
                setTimeout(async () => {
                    width =
                        scrollContainer?.clientWidth ??
                        libraryContainer?.clientWidth ??
                        0;

                    calculateColumns();
                    await tick();
                    calculateViewport(); // One more time to fix height
                    resolve(null);
                }, 50);
            });
        }
    }

    function calculateColumns(saveWidths: boolean = false) {
        if (width === 0) return;

        let runningX = 0;
        let previousWidth = 0;

        const allFields = cloneDeep(getAllColumns());

        if (displayFields?.length === 0) {
            displayFields = allFields;
        }

        const sortedFields = $columnOrder
            .map((c) => {
                const field = allFields.find((f) => f.value === c.fieldName);
                if (field === undefined) return null;
                const cloned = { ...field };
                // Restore user saved width
                if (isInit && cloned && c.width) {
                    cloned.viewProps.autoWidth = false;
                    cloned.viewProps.width = c.width;
                }
                return cloned;
            })
            .filter(Boolean);

        // By default, the library fits the columns automatically
        // Until you manually resize, in which case we ignore the default sizes going forward
        // and save the new widths to the column config
        let shouldUseAutoWidth = $columnOrder.every((c) => !c.width);

        /* Playlists show an additional file order column */
        if ($uiView === "playlists") {
            sortedFields.unshift({
                name: "none",
                value: "none",
                displayValue: "viewModel.index",
                operation: "+1",
                viewProps: {
                    width: 50,
                    x: 0,
                    autoWidth: false,
                },
            });
        }

        let autoWidth = null;
        if (shouldUseAutoWidth) {
            // Calculate total width of fixed-width rectangles

            const fixedWidths = sortedFields
                .filter((f) => !f.viewProps?.autoWidth)
                .map((f) => f.viewProps?.width);

            const totalFixedWidth = fixedWidths.reduce(
                (total, width) => total + width,
                0,
            );
            // Calculate total available width for 'auto' size rectangles
            let availableWidth = width - totalFixedWidth;
            if (availableWidth < 0) {
                availableWidth = 0;
                shouldUseAutoWidth = false;
            }
            autoWidth =
                availableWidth / (sortedFields.length - fixedWidths.length);
        }

        // Final display fields
        displayFields = [
            ...sortedFields.map((f) => {
                const rectWidth =
                    shouldUseAutoWidth && f.viewProps.autoWidth
                        ? autoWidth
                        : f.viewProps.width;
                f.viewProps.x = runningX += previousWidth;
                f.viewProps.width = rectWidth;
                previousWidth = f.viewProps.width;
                return f;
            }),
        ];

        // Optionally, save the widths
        if (saveWidths) {
            $columnOrder = displayFields.map((f) => ({
                fieldName: f.value,
                width: f.viewProps.width,
            }));
        }
    }

    let prevRemainder = 0; // To fix choppiness when jumping from eg. 18 to 1.

    async function calculateSongSlice() {
        if (!songs || songs.length === 0) {
            songsStartSlice = 0;
            songsEndSlice = 0;
            return;
        }

        const songsCountScrollable = Math.ceil(scrollableArea / ROW_HEIGHT);
        const songsCountViewport = Math.ceil(viewportHeight / ROW_HEIGHT);

        let contentY = scrollableArea * scrollNormalized;
        sandwichTopHeight = contentY;
        sandwichBottomY = contentY + viewportHeight;
        sandwichBottomHeight =
            contentHeight - sandwichTopHeight - viewportHeight;

        songsStartSlice = Math.floor(
            Math.max(0, Math.floor(scrollNormalized * songsCountScrollable)),
        );
        songsEndSlice = Math.min(
            songs.length,
            Math.ceil(songsStartSlice + songsCountViewport),
        );

        prevScrollPos = scrollPos;
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
        } else if (e) {
            // From mouse wheel
            newScrollPos = scrollPos + e.detail.evt.deltaY;
        } else if (scrollPosY != null) {
            // From programmatic scroll
            newScrollPos = scrollPosY;
        }
        // Handle boundaries
        if (newScrollPos < 0) {
            newScrollPos = 0;
        } else if (
            newScrollPos >
            contentHeight - viewportHeight - HEADER_HEIGHT
        ) {
            newScrollPos = contentHeight - viewportHeight - HEADER_HEIGHT;
        } else if (
            newScrollPos ===
            contentHeight - viewportHeight - HEADER_HEIGHT
        ) {
            scrollNormalized = 1;
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

            if (currentSongScrollIdx !== null) {
                currentSongInView =
                    currentSongScrollIdx >= songsStartSlice &&
                    currentSongScrollIdx <= songsEndSlice;
            }
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

    let currentSongY = 0;

    $: if (columnOrder && $current.song) {
        const { id } = $current.song;
        const idx = allSongs?.findIndex((s) => s.id === id);

        if (idx !== undefined) {
            currentSongScrollIdx = idx;
            currentSongY = idx * ROW_HEIGHT;
            currentSongInView = idx >= songsStartSlice && idx <= songsEndSlice;
        }
    }

    function scrollToCurrentSong() {
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
        let idx = allSongs?.findIndex((s) => {
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
            songHighlighter?.toggleHighlight(
                found,
                found.viewModel.index,
                false,
            );
        }
    }

    // LIBRARY FUNCTIONALITY

    let isShiftPressed = false;
    let rangeStartSongIdx = null;
    let rangeEndSongIdx = null;
    let highlightedSongIdx = 0;
    let trackMenu: TrackMenu;
    let currentSongInView = false;
    let currentSongScrollIdx = null;

    async function onDoubleClickSong(song, idx) {
        if (song.isStem) {
            audioPlayer.playSong(song);
            return;
        }
        if ($uiView.match(/^(albums)/)) {
            setQueue(songsArray, idx);
        } else if ($uiView === "smart-query") {
            setQueue($smartQueryResults, idx);
        } else if ($uiView === "favourites") {
            setQueue($smartQueryResults, idx);
        } else {
            setQueue($queriedSongs, idx);
        }

        if (query?.length) {
            $queueMirrorsSearch = true;
        }
    }

    async function onRightClick(e, song, idx) {
        if (!songsHighlighted.includes(song)) {
            songHighlighter?.toggleHighlight(song, idx, false, false);
        }

        // console.log("songIdsHighlighted", songsHighlighted);
        if (songsHighlighted.length > 1) {
            $rightClickedTracks = songsHighlighted;
            $rightClickedTrack = null;
        } else {
            $rightClickedTrack = song;
        }

        await tick();
        openTrackMenu(e, songsHighlighted);
    }

    function openTrackMenu(evt: MouseEvent, songs: Song[]) {
        openContextMenu(evt, {
            component: TrackMenu,
            props: {
                songs: songs,
                onUnselect: () => {
                    songsHighlighted.length = 0;
                },
            },
        });
    }

    function openStemsDropdown(evt: MouseEvent, song: Song) {
        openContextMenu(evt, {
            component: StemsDropdown,
            props: {
                song: song,
            },
        });
    }

    /**
     * Highlight behaviour on query input:
     * - if a single song is highlighted, it will stay highlighted as you type (if still in results)
     * - if multiple songs are highlighted, they will be unhighlighted and the first song will be highlighted as you type
     */
    $: {
        if (query?.length && $popupOpen !== "track-info") {
            if (songs?.length === 0) {
                songHighlighter?.reset();
            } else {
                songHighlighter?.highlightFirst();
            }
        }
    }

    let songHighlighter: SongHighlighter;
    export let songsHighlighted: Song[] = [];
    export let onSongsHighlighted = null;

    async function updateHighlights(songs: Song[], idx: number) {
        // Highlighted songs indexes might need to be updated
        if (songsHighlighted.length > 0) {
            songsHighlighted = songsHighlighted.filter(Boolean).map((s) => {
                s.viewModel.index = songs?.find(
                    (song) => song.id === s.id,
                )?.viewModel?.index;
                highlightedSongIdx = s.viewModel.index;
                return s;
            });
        }
    }

    let draggingSongIdx = null;

    function onSongDragStart(song: Song, idx: number) {
        $arrowFocus = "library";
        // console.log("songshighlighted", songsHighlighted);
        if (
            songsHighlighted.length > 1 &&
            songHighlighter?.isSongIdxHighlighted(idx)
        ) {
            setDraggedSongs(songsHighlighted, "Library");
        } else {
            setDraggedSongs([song], "Library");

            if (idx !== undefined && $uiView === "playlists") {
                draggingSongIdx = idx;
            }
        }
    }

    async function onMouseUpContainer(e) {
        if (isDraggingOver) {
            const playlist = $selectedPlaylistFile;
            const songs = $draggedSongs;

            isDraggingOver = false;
            resetDraggedSongs();

            console.log("[Library] Adding to playlist: ", playlist);
            await addSongsToPlaylist(playlist, songs);
            toast.success(
                `${
                    songs.length > 1 ? songs.length + " songs" : songs[0].title
                } added to ${playlist.path}`,
                {
                    position: "bottom-center",
                },
            );

            $selectedPlaylistFile = $selectedPlaylistFile; // trigger re-render
        }
    }

    async function onMouseUpSong(song: Song, idx: number) {
        if (isDraggingOver) {
            const playlist = $selectedPlaylistFile;
            const songs = $draggedSongs;

            // reset immediately so mouseUp on container won't trigger another addition
            isDraggingOver = false;
            resetDraggedSongs();

            console.log("[Library] Insert to playlist: ", playlist);
            await insertSongsToPlaylist(playlist, songs, idx);
            toast.success(
                `${
                    songs.length > 1 ? songs.length + " songs" : songs[0].title
                } added to ${playlist.path}`,
                {
                    position: "bottom-center",
                },
            );

            $selectedPlaylistFile = $selectedPlaylistFile; // trigger re-render
        } else if (draggingSongIdx !== null && $selectedPlaylistFile) {
            console.log("reorder song", idx);
            if (idx === draggingSongIdx) {
                draggingSongIdx = null;
                return;
            }

            if (songOrder?.orderBy !== "none") {
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
    function onKeyDown(event) {
        // Allow ctrl/cmd+a to select all songs
        if (
            isOver &&
            event.code === "KeyA" &&
            (($os === "macos" && event.metaKey) || event.ctrlKey)
        ) {
            event.preventDefault();
            songHighlighter?.highlightAll();
        }

        if ($arrowFocus !== "library") return;

        if (
            event.key === "Escape" &&
            $popupOpen !== "track-info" &&
            $singleKeyShortcutsEnabled &&
            !isInputFocused()
        ) {
            console.log("hmmm");
            if (trackMenu?.isOpen()) {
                trackMenu.close();
            } else {
                songHighlighter?.reset();
            }
        }
        // Single key shortcuts
        else if (
            event.code === "KeyI" &&
            $popupOpen !== "track-info" &&
            $singleKeyShortcutsEnabled &&
            !isInputFocused()
        ) {
            // 'i' for info popup
            event.preventDefault();
            // Check if there an input in focus currently
            if (songsHighlighted.length) {
                console.log("opening info", songsHighlighted);
                if (songsHighlighted.length > 1) {
                    $rightClickedTracks = songsHighlighted;
                    $rightClickedTrack = null;
                } else {
                    $rightClickedTrack = songsHighlighted[0];
                    $rightClickedTracks = [];
                }
                $rightClickedAlbum = null;
                $popupOpen = "track-info";
            }
        } else if (
            event.code === "KeyT" &&
            $popupOpen !== "track-info" &&
            $singleKeyShortcutsEnabled &&
            !isInputFocused()
        ) {
            // 't' for tags/right-click menu
            event.preventDefault();
            // Check if there an input in focus currently
            if (!trackMenu.isOpen && songsHighlighted.length) {
                console.log("opening info", songsHighlighted);

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

                trackMenu.open(
                    songsHighlighted.length > 1 ? songsHighlighted : topTrack,
                    { x: 250, y: topTrackY },
                );
            }
        } else if (
            event.key === "Enter" &&
            $popupOpen !== "track-info" &&
            $singleKeyShortcutsEnabled &&
            !isInputFocused()
        ) {
            // 'Enter' to play highlighted track
            event.preventDefault();
            AudioPlayer.shouldPlay = true;
            setQueue($queriedSongs, highlightedSongIdx);

            if (query?.length) {
                $queueMirrorsSearch = true;
            }
        } else {
            // Let the song highlighter handle it
            songHighlighter?.onKeyDown(event);
        }
    }

    function onKeyUp(event) {
        songHighlighter?.onKeyUp(event);
    }

    addEventListener("keydown", onKeyDown);
    addEventListener("keyup", onKeyUp);

    // COLUMNS

    function updateOrderBy(newOrderBy) {
        if (!songOrder) return;
        if (newOrderBy === "trackNumber") return; // Not supported
        if (songOrder.orderBy === newOrderBy) {
            songOrder.reverse = !songOrder.reverse;
        } else {
            songOrder.orderBy = newOrderBy;
        }
        songOrder = songOrder;
    }

    // Re-order columns

    $: {
        displayFields && $columnOrder && calculateColumns();
    }

    let dropColumnIdx = null;

    function handleColumnDrag(
        pos: { x: number; y: number },
        index,
    ): { x: number; y: number } {
        return { x: pos.x, y: 0 };
    }

    function onDragStart(event: KonvaDragTransformEvent, index) {
        event.detail.target.moveToBottom();
        $draggedColumnIdx = index;
    }

    function onDragEnd(event: KonvaDragTransformEvent, index) {
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

    function onGroupClick(ev: KonvaMouseEvent, field) {
        if (ev.detail.evt.button === 0 && resizingColumnIdx === null) {
            updateOrderBy(field.value);
        } else if (ev.detail.evt.button === 2) {
            if ($uiView === "albums") {
                const list = ev.detail.evt.target.closest(".konvajs-content");
                const rect = list.getBoundingClientRect();
                console.log("rect", rect, "clientY", ev.detail.evt.clientY);
                columnPickerPos = {
                    x: ev.detail.evt.clientX - rect.left,
                    y: ev.detail.evt.clientY - rect.top,
                };
            } else {
                columnPickerPos = {
                    x: ev.detail.evt.clientX,
                    y: 15,
                };
            }

            const list = ev.detail.evt.target.closest(".konvajs-content");
            const rect = list.getBoundingClientRect();
            const x = ev.detail.evt.clientX - rect.left;
            columnPickerIndex = displayFields.findIndex(
                ({ viewProps }) =>
                    viewProps.x <= x && x <= viewProps.x + viewProps.width,
            );

            openColumnPicker(ev.detail.evt);
        }
    }

    function openColumnPicker(event: MouseEvent) {
        openContextMenu(event, {
            component: ColumnPicker,
            props: {
                columnIndex: columnPickerIndex,
                displayedColumns: columnOrder,
                onResetOrder: () => {
                    resetColumnOrder();
                },
            },
        });
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
        const columnOrderOldIdx = $columnOrder.findIndex(
            (c) => c.fieldName === oldIdxField.value,
        );
        const columnOrderNewIdx = $columnOrder.findIndex(
            (c) => c.fieldName === newIdxField.value,
        );
        $columnOrder = moveArrayElement(
            $columnOrder,
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
        const columnOrderOldIdx = $columnOrder.findIndex(
            (c) => c.fieldName === oldIdxField.value,
        );
        const columnOrderNewIdx = $columnOrder.findIndex(
            (c) => c.fieldName === newIdxField.value,
        );
        console.log("swap column", columnOrderOldIdx, columnOrderNewIdx);
        $columnOrder = swapArrayElements(
            $columnOrder,
            columnOrderOldIdx,
            columnOrderNewIdx,
        );
        // displayFields = swapArrayElements(displayFields, oldIndex, newIndex);
        console.log("column order", columnOrder);
        resetColumnOrderUi();
    }

    // Sets back to default
    function resetColumnOrder() {
        displayFields = [];
        columnOrder.reset();
    }

    // Resize columns
    let resizingColumnIdx = null;
    let resizeStartX = 0;
    let resizeStartWidth = 0;

    function onColumnResize(e) {
        if (resizingColumnIdx !== null) {
            const deltaX = e.clientX - resizeStartX;
            const newWidth = Math.max(50, resizeStartWidth + deltaX); // Min width 50px

            const field = displayFields[resizingColumnIdx];
            field.viewProps.width = newWidth;
            field.viewProps.autoWidth = false; // Lock the width
            calculateColumns(true);
        }
    }

    function onColumnResizeStop() {
        if (resizingColumnIdx !== null) {
            resizingColumnIdx = null;
            if (stage) {
                stage.container().style.cursor = "default";
            }
        }
        removeEventListener("mousemove", onColumnResize);
        removeEventListener("mouseup", onColumnResizeStop);
    }

    function onColumnMouseDown(
        ev: KonvaMouseEvent,
        f: ColumnViewModel,
        idx: number,
    ) {
        const mouseX = ev.detail.evt.offsetX - f.viewProps.x;
        const isInResizeZoneRightEdge =
            mouseX > f.viewProps.width - RESIZE_HANDLE_WIDTH / 2;
        const isInResizeZoneLeftEdge = mouseX < RESIZE_HANDLE_WIDTH / 2;
        if (
            idx > 0 &&
            (isInResizeZoneRightEdge || isInResizeZoneLeftEdge) &&
            ev.detail.evt.button === 0
        ) {
            ev.detail.evt.cancelBubble = true;
            resizingColumnIdx = isInResizeZoneRightEdge ? idx : idx - 1;
            resizeStartX = ev.detail.evt.clientX;
            resizeStartWidth = isInResizeZoneRightEdge
                ? f.viewProps.width
                : displayFields[idx - 1].viewProps.width;

            addEventListener("mousemove", onColumnResize);
            addEventListener("mouseup", onColumnResizeStop);
        } else {
            resizingColumnIdx = null;
        }
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

    let expandChevronHoverSongId = null;

    onDestroy(() => {
        hotkeys.unbind("esc");
        removeEventListener("keydown", onKeyDown);
        removeEventListener("keyup", onKeyUp);
    });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-click-events-have-key-events -->

<SongHighlighter
    bind:this={songHighlighter}
    bind:songs
    bind:songsHighlighted
    bind:onSongsHighlighted
/>

<div
    class="library-container"
    class:dragover={isDraggingOver}
    bind:this={libraryContainer}
>
    {#if isLoading}
        <!-- <div class="loading" out:fade={{ duration: 90, easing: cubicInOut }}>
            <p>💿 one sec...</p>
        </div> -->
    {:else if noSongs && $selectedPlaylistFile}
        <div
            id="scroll-container"
            class:ready
            on:mouseenter={() => {
                isDraggingOver =
                    $selectedPlaylistFile && $draggedSongs?.length > 0;
            }}
            on:mouseleave={() => {
                isDraggingOver = false;
            }}
        >
            <div class="container" on:mouseup={onMouseUpContainer}>
                <h2>Empty playlist</h2>
                <p>🪣</p>
            </div>
        </div>
    {:else if theme === "default" && (($importStatus.isImporting && $importStatus.backgroundImport === false) || (noSongs && query?.length === 0 && $uiView.match(/^(smart-query|favourites|to-delete)/) === null && $isTagCloudOpen === false))}
        <ImportPlaceholder />
    {:else}
        <div
            id="scroll-container"
            style="overflow-y: {false ? 'visible' : 'hidden'}"
            class:ready
            bind:this={scrollContainer}
            on:mouseenter={() => {
                isOver = true;
                isDraggingOver =
                    $selectedPlaylistFile && $draggedSongs?.length > 0;
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
                on:mouseup={onMouseUpContainer}
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
                                    <!-- MAIN ROW -->
                                    {#if !song.isStem}
                                        <Group
                                            on:dblclick={() =>
                                                onDoubleClickSong(
                                                    song,
                                                    song.viewModel.index,
                                                )}
                                            on:click={(e) => {
                                                // console.log("e", e);
                                                if (e.detail.evt.button === 0) {
                                                    songHighlighter?.toggleHighlight(
                                                        song,
                                                        song.viewModel.index,
                                                        false,
                                                        false,
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
                                                    songIdx >
                                                        songsIdxSlice.length -
                                                            15
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
                                                y:
                                                    sandwichTopHeight +
                                                    ROW_HEIGHT * songIdx,
                                                height: ROW_HEIGHT,
                                                width: width,
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
                                                    id:
                                                        song.viewModel
                                                            ?.viewId ??
                                                        song?.id,
                                                    x: 0,
                                                    y: 0,
                                                    width: width,
                                                    height: ROW_HEIGHT,
                                                    listening: true,
                                                    fill:
                                                        draggingSongIdx ===
                                                        song.viewModel?.index
                                                            ? DRAGGING_SOURCE_COLOR
                                                            : $current.song
                                                                    ?.id ===
                                                                song?.id
                                                              ? PLAYING_BG_COLOR
                                                              : songsHighlighted &&
                                                                  songHighlighter?.isSongIdxHighlighted(
                                                                      idx,
                                                                  )
                                                                ? HIGHLIGHT_BG_COLOR
                                                                : hoveredSongIdx ===
                                                                    songIdx
                                                                  ? ROW_BG_COLOR_HOVERED
                                                                  : songIdx %
                                                                          2 ===
                                                                      0
                                                                    ? ROW_BG_COLOR
                                                                    : ROW_ODD_BG_COLOR,
                                                }}
                                            />
                                            {#each displayFields as f, idx (f.value)}
                                                <!-- Smart query fields -->
                                                {#if f.value.match(/^(year|genre|originCountry)/) !== null && !isInvalidValue(getValue(song, f))}
                                                    <Label
                                                        config={{
                                                            x:
                                                                f.viewProps.x +
                                                                5,
                                                            y: 2.5,
                                                            width:
                                                                f.viewProps
                                                                    .width - 10,
                                                            height:
                                                                ROW_HEIGHT - 5,
                                                        }}
                                                        on:mouseenter={() => {
                                                            hoveredField =
                                                                f.value;
                                                        }}
                                                        on:mouseleave={() => {
                                                            hoveredField = null;
                                                        }}
                                                        on:click={() => {
                                                            filterByField(
                                                                f.value,
                                                                getValue(
                                                                    song,
                                                                    f,
                                                                ),
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
                                                                        .width -
                                                                    10,
                                                                height:
                                                                    ROW_HEIGHT -
                                                                    5,
                                                                align: "center",
                                                                fontSize: 13.5,
                                                                verticalAlign:
                                                                    "middle",
                                                                fill: TEXT_COLOR,
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
                                                                        y: 2.5,
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
                                                                        y: 2.5,
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
                                                                        trackMenu.open(
                                                                            song,
                                                                            {
                                                                                x: e
                                                                                    .detail
                                                                                    .evt
                                                                                    .clientX,
                                                                                y: e
                                                                                    .detail
                                                                                    .evt
                                                                                    .clientY,
                                                                            },
                                                                        );
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
                                                                idx === 0
                                                                    ? 12.5
                                                                    : f.value.match(
                                                                            /^(title|artist|album|track)/,
                                                                        ) !==
                                                                        null
                                                                      ? f
                                                                            .viewProps
                                                                            .x +
                                                                        10
                                                                      : f
                                                                            .viewProps
                                                                            .x,
                                                            y: 1,
                                                            text: validatedValue(
                                                                getValue(
                                                                    song,
                                                                    f,
                                                                ),
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
                                                                    ? f
                                                                          .viewProps
                                                                          .width -
                                                                      10
                                                                    : f.value.match(
                                                                            /^(title|artist|album|track)/,
                                                                        ) !==
                                                                        null
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
                                                                      : f
                                                                            .viewProps
                                                                            .width,
                                                            padding:
                                                                f.value.match(
                                                                    /^(genre)/,
                                                                ) !== null
                                                                    ? 10
                                                                    : 2,
                                                            height: HEADER_HEIGHT,
                                                            fontSize: 13.5,
                                                            verticalAlign:
                                                                "middle",
                                                            fill:
                                                                $current.song
                                                                    ?.id ===
                                                                song.id
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
                                                {#if f.value === "title"}
                                                    <!-- Now playing icon -->
                                                    {#if $current.song?.id === song.id}
                                                        <Path
                                                            config={{
                                                                x:
                                                                    f.viewProps
                                                                        .x +
                                                                    f.viewProps
                                                                        .width -
                                                                    40,
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
                                                                        .x +
                                                                    f.viewProps
                                                                        .width -
                                                                    20,
                                                                y: 6,
                                                                scaleX: 0.36,
                                                                scaleY: 0.36,
                                                                data: "M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87",
                                                                fill:
                                                                    $current
                                                                        .song
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
                                                                favouriteSong(
                                                                    song,
                                                                )}
                                                            config={{
                                                                x:
                                                                    f.viewProps
                                                                        .x +
                                                                    f.viewProps
                                                                        .width -
                                                                    20,
                                                                y: 6,
                                                                scaleX: 0.36,
                                                                scaleY: 0.36,
                                                                data: "M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87",
                                                                fill: "transparent",
                                                                stroke:
                                                                    $current
                                                                        .song
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
                                            <!-- Show stems/expand track indicator -->
                                            {#if song.stems?.length}
                                                <Path
                                                    config={{
                                                        x:
                                                            expandChevronHoverSongId ===
                                                            song.id
                                                                ? 0
                                                                : -3,
                                                        y: ROW_HEIGHT / 2 - 8,
                                                        listening: false,
                                                        scaleX: 0.6,
                                                        scaleY: 0.6,
                                                        data:
                                                            $expandedSongWithStems ===
                                                            song
                                                                ? "m6 9l6 6l6-6"
                                                                : "m9 18l6-6l-6-6",
                                                        stroke: $currentThemeObject[
                                                            "accent"
                                                        ],
                                                    }}
                                                />
                                                <!-- Slightly bigger hitbox for clicking -->
                                                <Rect
                                                    config={{
                                                        x: -2,
                                                        y: 0,
                                                        width: 15,
                                                        height: ROW_HEIGHT,
                                                        listening: true,
                                                        strokeWidth: 2,
                                                    }}
                                                    on:mousedown={(e) => {
                                                        e.preventDefault();
                                                    }}
                                                    on:click={(e) => {
                                                        console.log("CLICKED");
                                                        openStemsDropdown(
                                                            e.detail.evt,
                                                            song,
                                                        );
                                                    }}
                                                    on:mouseenter={() => {
                                                        expandChevronHoverSongId =
                                                            song.id;
                                                    }}
                                                    on:mouseleave={() => {
                                                        expandChevronHoverSongId =
                                                            null;
                                                    }}
                                                />
                                                {#if expandChevronHoverSongId === song.id}
                                                    <Label
                                                        config={{
                                                            x: 15,
                                                            height: ROW_HEIGHT,
                                                            listening: false,
                                                        }}
                                                    >
                                                        <Tag
                                                            config={{
                                                                fill: $currentThemeObject[
                                                                    "button-solid-bg"
                                                                ],
                                                                stroke: $currentThemeObject[
                                                                    "menu-border"
                                                                ],
                                                                strokeWidth: 1,
                                                                cornerRadius: 2,
                                                            }}
                                                        />
                                                        <Text
                                                            config={{
                                                                fill: "white",
                                                                text: "Expand stems",
                                                                height: ROW_HEIGHT,
                                                                fontSize: 13,
                                                                padding: 10,
                                                                verticalAlign:
                                                                    "middle",
                                                            }}
                                                        />
                                                    </Label>
                                                {/if}
                                            {/if}
                                            {#if hoveredSongIdx === songIdx && draggingSongIdx !== null}
                                                <Rect
                                                    config={{
                                                        x: 0,
                                                        y:
                                                            song.viewModel
                                                                ?.index >
                                                            draggingSongIdx
                                                                ? ROW_HEIGHT
                                                                : 0,
                                                        width: width,
                                                        height: DROP_HINT_HEIGHT,
                                                        fill: PLAYING_BG_COLOR,
                                                        listening: false,
                                                    }}
                                                />
                                            {/if}
                                        </Group>
                                    {:else if song.isStem}
                                        <Group
                                            config={{
                                                y:
                                                    sandwichTopHeight +
                                                    ROW_HEIGHT * songIdx,
                                                height: ROW_HEIGHT,
                                                width: width,
                                                listening: true,
                                            }}
                                            on:dblclick={() =>
                                                onDoubleClickSong(
                                                    song,
                                                    songIdx,
                                                )}
                                            on:click={(e) => {
                                                // console.log("e", e);
                                                if (e.detail.evt.button === 0) {
                                                    songHighlighter?.toggleHighlight(
                                                        song,
                                                        song.viewModel.index,
                                                        false,
                                                        false,
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
                                            }}
                                            on:mouseleave={() => {
                                                hoveredSongIdx = null;
                                            }}
                                        >
                                            <Rect
                                                config={{
                                                    x: 5,
                                                    y: song.isFirst ? 2 : 0,
                                                    width: width - 10,
                                                    height: song.isFirst
                                                        ? ROW_HEIGHT - 2
                                                        : song.isLast
                                                          ? ROW_HEIGHT - 2
                                                          : ROW_HEIGHT,
                                                    stroke: $currentThemeObject[
                                                        "library-column-divider"
                                                    ],
                                                    fill:
                                                        $current.song?.path ===
                                                        song.path
                                                            ? PLAYING_BG_COLOR
                                                            : songsHighlighted &&
                                                                songHighlighter?.isSongIdxHighlighted(
                                                                    idx,
                                                                )
                                                              ? HIGHLIGHT_BG_COLOR
                                                              : hoveredSongIdx ===
                                                                  songIdx
                                                                ? ROW_BG_COLOR_HOVERED
                                                                : "transparent",
                                                    strokeWidth: 1,
                                                    cornerRadius: song.isFirst
                                                        ? [5, 5, 0, 0]
                                                        : [0, 0, 5, 5],
                                                    listening: true,
                                                }}
                                            />
                                            <Text
                                                config={{
                                                    x: 10,
                                                    y: 0,
                                                    text: song.name,
                                                    height: ROW_HEIGHT,
                                                    fill: $currentThemeObject[
                                                        "library-text"
                                                    ],
                                                    fontSize: 13,
                                                    verticalAlign: "middle",
                                                    listening: false,
                                                }}
                                            />
                                        </Group>
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
                                        draggable: resizingColumnIdx === null, // Disable drag while resizing
                                        dragBoundFunc(pos) {
                                            return handleColumnDrag(pos, idx);
                                        },
                                    }}
                                    on:click={(ev) => {
                                        onGroupClick(ev, f);
                                    }}
                                    on:mouseenter={() => {
                                        hoveredColumnIdx = idx;
                                    }}
                                    on:mouseleave={(ev) => {
                                        hoveredColumnIdx = null;
                                        ev.detail.target
                                            .getStage()
                                            .container().style.cursor =
                                            "default";
                                    }}
                                    on:mousemove={(ev) => {
                                        const mouseX =
                                            ev.detail.evt.offsetX -
                                            f.viewProps.x;
                                        const isInResizeZoneRightEdge =
                                            mouseX >
                                            f.viewProps.width -
                                                RESIZE_HANDLE_WIDTH / 2;
                                        const isInResizeZoneLeftEdge =
                                            mouseX < RESIZE_HANDLE_WIDTH / 2;
                                        if (
                                            idx > 0 &&
                                            (isInResizeZoneRightEdge ||
                                                isInResizeZoneLeftEdge)
                                        ) {
                                            ev.detail.target
                                                .getStage()
                                                .container().style.cursor =
                                                "col-resize";
                                        } else {
                                            ev.detail.target
                                                .getStage()
                                                .container().style.cursor =
                                                "default";
                                        }
                                    }}
                                    on:mousedown={(ev) => {
                                        onColumnMouseDown(ev, f, idx);
                                    }}
                                    on:dragmove={(ev) => {
                                        onDragMove(ev);
                                    }}
                                    on:dragstart={(ev) => {
                                        if (resizingColumnIdx === null) {
                                            onDragStart(ev, idx);
                                        }
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
                                                        songOrder?.orderBy ===
                                                            "none"
                                                      ? HEADER_BG_COLOR_ACCENT
                                                      : query !== null &&
                                                          (hoveredColumnIdx ===
                                                              idx ||
                                                              songOrder?.orderBy ===
                                                                  f.value)
                                                        ? HEADER_BG_COLOR_HOVERED
                                                        : HEADER_BG_COLOR,
                                        }}
                                    />
                                    {#if query !== null && hoveredColumnIdx === idx}
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
                                    {#if songOrder}
                                        {#if songOrder.orderBy === f.value || (songOrder.orderBy === "none" && f.name === "none")}
                                            {#if songOrder.reverse}
                                                <Path
                                                    config={{
                                                        x:
                                                            f.viewProps.width -
                                                            16,
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
                                                        x:
                                                            f.viewProps.width -
                                                            16,
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
                                            visible: true,
                                        }}
                                    />
                                </Group>
                                <!-- Column divider -->
                                {#if idx > 0}
                                    <Rect
                                        config={{
                                            x: f.viewProps.x - 1,
                                            y:
                                                sandwichTopHeight +
                                                HEADER_HEIGHT,
                                            height: viewportHeight,
                                            width: 0.5,
                                            fill: COLUMN_DIVIDER_COLOR,
                                            listening: false,
                                        }}
                                    />
                                {/if}
                            {/each}
                            {#if resizingColumnIdx !== null}
                                <Rect
                                    config={{
                                        x:
                                            displayFields[resizingColumnIdx]
                                                .viewProps.x +
                                            displayFields[resizingColumnIdx]
                                                .viewProps.width -
                                            RESIZE_HANDLE_WIDTH / 2,
                                        y: sandwichTopHeight + HEADER_HEIGHT,
                                        height: viewportHeight,
                                        width: RESIZE_HANDLE_WIDTH,
                                        fill: COLUMN_RESIZE_DIVIDER_COLOR,
                                        listening: false,
                                    }}
                                />
                            {/if}
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
                {#if query?.length && noSongs}
                    <QueryResultsPlaceholder />
                {/if}
            </div>
        </div>
    {/if}
    {#if $uiView === "library" && $isPlaying && $current.song && !currentSongInView}
        <ScrollTo equalizer={true} on:click={scrollToCurrentSong}>
            {$LL.albums.scrollToNowPlaying()}
        </ScrollTo>
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
        overflow: hidden;
        &.dragover {
            border-color: var(--accent-secondary);
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
        align-items: center;
        justify-content: center;
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
