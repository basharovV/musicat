<!-- SongDataGrid.svelte -->

<script lang="ts">
    import { onMount } from "svelte";
    import { Group, Layer, Rect, Stage, Text } from "svelte-konva";

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
                width: 50,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Year",
            value: "year",
            show: true,
            viewProps: {
                width: 50,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Genre",
            value: "genre",
            show: true,
            viewProps: {
                width: 80,
                x: 0,
                autoWidth: false
            }
        },
        {
            name: "Origin",
            value: "originCountry",
            show: true,
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
                width: 65,
                x: 0,
                autoWidth: false
            }
        }
    ];

    export let fields = DEFAULT_FIELDS;

    $: displayFields = fields.filter((f) => f.show);
    let songsSlice: Song[];
    let canvas;

    let shouldRender = false;

    let scrollContainer: HTMLDivElement;
    let container: HTMLElement;
    let stage;
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
    const HEADER_HEIGHT = 20;
    const ROW_HEIGHT = 22;
    const BORDER_WIDTH = 1;
    const SCROLL_PADDING = 200;
    const DUMMY_COUNT = 5;
    const DUMMY_PADDING = DUMMY_COUNT * ROW_HEIGHT;

    // COLORS
    const BG_COLOR = "rgba(36, 33, 34, 0.688)";
    const HEADER_BG_COLOR = "#71658e7e";
    const TEXT_COLOR = "rgb(211, 211, 211)";
    const HIGHLIGHT_BG_COLOR = "#2e3357";
    const ROW_BG_COLOR = "transparent";
    const PLAYING_BG_COLOR = "#5123dd";
    const PLAYING_TEXT_COLOR = "#00ddff";
    onMount(() => {
        init();
    });

    function init() {
        drawSongDataGrid();
    }

    let contentHeight = HEADER_HEIGHT;

    $: if (songs?.length) {
        contentHeight = HEADER_HEIGHT + songs?.length * ROW_HEIGHT ;
        calculateSongSlice();
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
        console.log("fields", fields);
        console.log("canvas size", width, virtualViewportHeight);
        console.log("dpr", dpr);
    }

    function calculateCanvasSize() {
        // Get the device pixel ratio
        dpr = window.devicePixelRatio || 1;
        // Set canvas size to fill the parent and account for high-DPI displays
        const rect = scrollContainer.getBoundingClientRect();
        width = rect.width;
        viewportHeight = container.getBoundingClientRect().height;
        virtualViewportHeight = viewportHeight + SCROLL_PADDING * 2;
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
        console.log("width", width, "totalFixedWidth", totalFixedWidth);
        // Calculate available width for 'auto' size rectangles
        const availableWidth = width - totalFixedWidth;

        // Calculate the width for each 'auto' size rectangle
        const autoWidth =
            availableWidth / (displayFields.length - fixedWidths.length);

        displayFields.forEach((f) => {
            const rectWidth = f.viewProps.autoWidth
                ? autoWidth
                : f.viewProps.width;
            f.viewProps.x = runningX += previousWidth;
            f.viewProps.width = rectWidth;
            previousWidth = f.viewProps.width;
            return f;
        });
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
            const scrollableArea = contentHeight - viewportHeight;
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
            let songsStartSlice = Math.floor(
                Math.max(0, Math.floor(scrollNormalized * songsCountScrollable))
            );
            let songsEndSlice = Math.min(
                songs.length,
                Math.ceil(songsStartSlice + songsCountViewport)
            );

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
                        ...songs.slice(firstSongIdx, lastSongIdx).map(s => ({...s, dummy: true}))
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
                        ...songs.slice(firstSongIdx, lastSongIdx).map(s => ({...s, dummy: true}))
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

            var dy = scrollPos;
            // stage.container().style.transform = "translateY(" + -dy + "px)";
            // requestAnimationFrame(() => stage.y(-dy));
        }
    }

    // LIBRARY FUNCTIONALITY

    function onDoubleClickSong(song, idx) {
        $currentSongIdx = idx;
        $playlist = $queriedSongs;
        $playlistIsAlbum = false;
        AudioPlayer.playSong(song);
    }

    let isCmdOrCtrlPressed = false;
    let isShiftPressed = false;
    let rangeStartSongIdx = null;
    let rangeEndSongIdx = null;
    let highlightedSongIdx = 0;

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
</script>

<div id="scroll-container" on:scroll={onScroll} bind:this={scrollContainer}>
    <div id="large-container" style="height: {contentHeight}px;"></div>
</div>

<div class="container" bind:this={container}>
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
                        fill: HEADER_BG_COLOR
                    }}
                />
                {#each displayFields as f, idx (f.name)}
                    <Rect
                        config={{
                            x: f.viewProps.x,
                            y: sandwichTopHeight,
                            width:
                                idx === displayFields.length - 1
                                    ? f.viewProps.width
                                    : f.viewProps.width - 2,
                            height: HEADER_HEIGHT,
                            fill: HEADER_BG_COLOR
                        }}
                    />
                    <Text
                        config={{
                            x: f.viewProps.x,
                            y: sandwichTopHeight,
                            text: f.name,
                            align: "center",
                            width:
                                idx === displayFields.length - 1
                                    ? f.viewProps.width
                                    : f.viewProps.width - 2,
                            height: HEADER_HEIGHT,
                            fontSize: 14,
                            verticalAlign: "middle",
                            fill: TEXT_COLOR
                        }}
                    />
                {/each}
                {#if songsSlice?.length}
                    {#each songsSlice as song, songIdx (song.id)}
                        <Group
                            on:dblclick={() => onDoubleClickSong(song, songIdx)}
                            on:click={(e) => {
                                console.log("e", e);
                                toggleHighlight(song, songIdx);
                            }}
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
                                              : ROW_BG_COLOR
                                }}
                            />
                            {#each displayFields as f, idx (f.value)}
                                <Text
                                    config={{
                                        x: f.viewProps.x,
                                        y:
                                            sandwichTopHeight +
                                            HEADER_HEIGHT +
                                            ROW_HEIGHT * songIdx +
                                            -DUMMY_PADDING +
                                            scrollOffset,
                                        text: song[f.value],
                                        padding: f.value === "title" ? 10 : 0,
                                        align: f.value === "title" ? "left" : "center",
                                        width:
                                            idx === displayFields.length - 1
                                                ? f.viewProps.width
                                                : f.viewProps.width - 2,
                                        height: HEADER_HEIGHT,
                                        fontSize: 14,
                                        verticalAlign: "middle",
                                        fill:
                                            $currentSong?.id === song.id
                                                ? PLAYING_TEXT_COLOR
                                                : TEXT_COLOR
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
                            fill: HEADER_BG_COLOR
                        }}
                    />
                {/if}
            </Layer>
        </Stage>
    {/if}
</div>

<style>
    .container {
        width: 100%;
        height: 100vh;
        pointer-events: auto;
    }

    #large-container {
        width: 100%;
        overflow: hidden;
    }

    #scroll-container {
        width: calc(100%);
        height: 100vh;
        position: absolute;
        overflow: auto;
        z-index: 10;
        pointer-events: visible;
    }
</style>
