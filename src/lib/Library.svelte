<script lang="ts">
    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import "iconify-icon";
    import { debounce } from "lodash-es";
    import type { Song } from "src/App";
    import { onDestroy } from "svelte";
    import toast from "svelte-french-toast";
    import { flip } from "svelte/animate";
    import { cubicInOut, quadOut } from "svelte/easing";
    import { get } from "svelte/store";
    import { fade, fly } from "svelte/transition";
    import { openTauriImportDialog } from "../data/LibraryImporter";
    import BuiltInQueries from "../data/SmartQueries";
    import { db } from "../data/db";

    import {
        currentSong,
        currentSongIdx,
        draggedSongs,
        fileDropHandler,
        importStatus,
        isPlaying,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isTrackInfoPopupOpen,
        libraryScrollPos,
        nextUpSong,
        os,
        query,
        rightClickedTrack,
        rightClickedTracks,
        singleKeyShortcutsEnabled,
        smartQuery,
        smartQueryInitiator,
        songsJustAdded,
        uiView,
        draggedColumnIdx,
        emptyDropEvent,
        droppedFiles
    } from "../data/store";
    import { getFlagEmoji } from "../utils/EmojiUtils";
    import AudioPlayer from "./AudioPlayer";
    import ImportPlaceholder from "./ImportPlaceholder.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    import { codes } from "./data/CountryCodes";
    import { getQueryPart } from "./smart-query/QueryParts";
    import SmartQueryBuilder from "./smart-query/SmartQueryBuilder.svelte";
    import SmartQueryMainHeader from "./smart-query/SmartQueryMainHeader.svelte";
    import SmartQueryResultsPlaceholder from "./smart-query/SmartQueryResultsPlaceholder.svelte";
    import { UserQueryPart } from "./smart-query/UserQueryPart";
    import { optionalTippy } from "./ui/TippyAction";
    import ColumnPicker from "./ColumnPicker.svelte";
    import { element } from "svelte/internal";
    import { moveArrayElement, swapArrayElements } from "../utils/ArrayUtils";

    // TODO
    async function calculateSize(songs: Song[]) {}

    export let allSongs = null;

    $: songs = $allSongs
        ?.filter((song: Song) => {
            if (compressionSelected === "lossless") {
                return song.fileInfo.lossless;
            } else if (compressionSelected === "lossy") {
                return song.fileInfo.lossless === false;
            } else return true;
        })
        .reduce(
            (status, s, idx, songsArray) => {
                if (s.album !== status.state.previousAlbum) {
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
                if (s.artist !== status.state.previousArtist) {
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
                status.state.previousAlbum = s.album;
                status.state.previousArtist = s.artist;
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

    export let isLoading = true;

    const DEFAULT_FIELDS = [
        { name: "Title", value: "title", show: true },
        { name: "Artist", value: "artist", show: true },
        { name: "Composer", value: "composer", show: false },
        { name: "Album", value: "album", show: true },
        { name: "Track", value: "trackNumber", show: true },
        { name: "Year", value: "year", show: true },
        { name: "Genre", value: "genre", show: true },
        { name: "Origin", value: "originCountry", show: true },
        { name: "Duration", value: "duration", show: true }
    ];

    export let fields = DEFAULT_FIELDS;

    export let showMyArtists = false;

    export let theme = "default";
    export let isSmartQueryEnabled = true; // Only for main view
    export let onSongsHighlighted = null;

    let showColumnPicker = false;
    let columnPickerPos;

    $: artists = liveQuery(async () => {
        let results = await db.artistProjects.toArray();
        return results.map((a) => a.name);
    });

    $: count = liveQuery(() => {
        return db.songs.count();
    });

    $: artistCount = liveQuery(async () => {
        const artists = await db.songs.orderBy("artist").uniqueKeys();
        return artists.length;
    });

    $: albumCount = liveQuery(async () => {
        const albums = await db.songs.orderBy("album").uniqueKeys();
        return albums.length;
    });

    $: noSongs =
        !songs ||
        songs.length === 0 ||
        ($isSmartQueryBuilderOpen && $smartQuery.isEmpty);

    let container: HTMLElement;

    $: {
        if (container && !isLoading && songs?.length) {
            // Restore scroll position if any
            if ($libraryScrollPos && container) {
                container.scrollTop = $libraryScrollPos;
            }
        }
    }

    function onScroll(evt) {
        console.log("scroll", evt);
        $libraryScrollPos = evt.target.scrollTop;
    }

    function scrollToCurrentSong() {
        if (isPlaying && $currentSong) {
            const foundRow = document.querySelector(
                `[data-song='${$currentSong.id}']`
            );
            console.log("foundRow", foundRow.offsetTop);
            if (foundRow) {
                foundRow.scrollIntoView({
                    block: "center",
                    behavior: "smooth"
                });
            }
        }
    }

    function updateOrderBy(newOrderBy) {
        if ($query.orderBy === newOrderBy) {
            $query.reverse = !$query.reverse;
        }
        $query.orderBy = newOrderBy;
        $query = $query;
    }

    async function clearJustAdded() {
        setTimeout(() => {
            $songsJustAdded = [];
        }, 1000);
    }

    $: {
        if ($songsJustAdded.length) {
            toast.success(
                $songsJustAdded.length +
                    ($songsJustAdded.length === 1 ? " song" : " songs") +
                    " imported.",
                {
                    position: "bottom-center"
                }
            );
            clearJustAdded();
        }
    }

    function isSongJustAdded(songId) {
        // TODO optimize using songjustadded (boolean)
        const isAdded = $songsJustAdded.map((s) => s.id).includes(songId);
        return isAdded;
    }

    export let songsHighlighted: Song[] = [];

    function isSongHighlighted(song: Song) {
        return songsHighlighted.map((s) => s.id).includes(song.id);
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

    let isCmdOrCtrlPressed = false;
    let isShiftPressed = false;
    let rangeStartSongIdx = null;
    let rangeEndSongIdx = null;
    let highlightedSongIdx = 0;

    $: {
        if (songs?.length && $query.query?.length) {
            highlightSong(songs[0], 0, false);
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

    // hotkeys("up", function (event, handler) {});

    // hotkeys("down", function (event, handler) {});
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

    function highlightSong(song: Song, idx, isKeyboardArrows: boolean) {
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

    let showTrackMenu = false;
    let pos;

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
        pos = { x: e.clientX, y: e.clientY };
    }

    function onDoubleClickSong(song, idx) {
        $currentSongIdx = idx;
        AudioPlayer.playSong(song);
    }

    // Smart query stuff

    // Smart query - predefined or user-defined queries i.e smart playlists
    let smartQuerySelectedVal = BuiltInQueries[0].value;

    function hideSmartQueryBuilder() {
        if (tableHeaders[0] === "smart-query-builder") {
            tableHeaders.splice(0, 1);
            tableHeaders = tableHeaders;
        } else if (tableHeaders[1] === "smart-query-builder") {
            tableHeaders.splice(1, 1);
            tableHeaders = tableHeaders;
        }
    }

    function showSmartQueryBuilder() {
        if (!tableHeaders.includes("smart-query-builder")) {
            tableHeaders.unshift("smart-query-builder");
            $isSmartQueryBuilderOpen = true;
        }
    }

    const showSmartQuery = () => {
        console.log("headers showSmartQuery", tableHeaders);

        const found = tableHeaders.findIndex((h) => h === "smart-query");
        const builderAlreadyShown = tableHeaders.includes(
            "smart-query-builder"
        );
        if (found === -1) {
            if (builderAlreadyShown) {
                tableHeaders.splice(1, 0, "smart-query");
            } else {
                tableHeaders.unshift("smart-query");
            }
        }
        tableHeaders = tableHeaders;
    };

    const hideSmartQuery = () => {
        console.log("headers hideSmartQuery", tableHeaders);
        if (tableHeaders[0] === "smart-query-builder") {
            tableHeaders.splice(0, 1);
        }

        const found = tableHeaders.findIndex((h) => h === "smart-query");
        if (found > -1) {
            tableHeaders.splice(found, 1);
        }
        console.log("headers", tableHeaders);
        tableHeaders = tableHeaders;
    };

    /**
     * We need to put this here so that the flip animation works as expected,
     * since it only accepts keyed objects in an each block
     */
    let tableHeaders = ["track-fields"];
    $: {
        if (
            isSmartQueryEnabled &&
            $uiView === "smart-query" &&
            (($isSmartQueryBuilderOpen && $isSmartQuerySaveUiOpen) ||
                !$isSmartQueryBuilderOpen)
        ) {
            showSmartQuery();
        } else {
            hideSmartQuery();
        }

        if (isSmartQueryEnabled && $isSmartQueryBuilderOpen) {
            showSmartQueryBuilder();
        } else {
            hideSmartQueryBuilder();
        }
    }

    let compressionSelected: "lossy" | "lossless" | "both" = "both";

    async function favouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: true
        });
    }

    async function unfavouriteSong(song: Song) {
        await db.songs.update(song, {
            isFavourite: false
        });
    }

    function filterByField(fieldName: string, fieldValue: any) {
        let queryPart;
        console.log("filter", fieldName, fieldValue);
        switch (fieldName) {
            case "genre":
                queryPart = getQueryPart("CONTAINS_GENRE");
                $smartQueryInitiator = "genre-pill";
                break;
            case "year":
                queryPart = getQueryPart("RELEASED_IN");
                $smartQueryInitiator = "genre-pill";
                break;
            case "originCountry":
                queryPart = getQueryPart("FROM_COUNTRY");
                $smartQueryInitiator = "genre-pill";
                break;
            default:
                return;
        }
        if ($uiView !== "smart-query") {
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

    function onSongDragStart(song: Song) {
        console.log("dragstart", song);
        console.log("songshighlighted", songsHighlighted);
        if (songsHighlighted.length > 1) {
            $draggedSongs = songsHighlighted;
        } else {
            $draggedSongs = [song];
        }
    }

    // Re-order columns

    function handleDragStart(event: DragEvent, index) {
        event.dataTransfer.setData("text", index);
        event.dataTransfer.effectAllowed = "move";
        $draggedColumnIdx = index;
        $fileDropHandler = "library-header";
    }

    function handleDragEnter(event, index) {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        dropColumnIdx = index;
    }

    let hoveredColumnIdx = null;
    let columnToInsertIdx = null;
    let columnToInsertXPos = 0;
    $: numColumns = fields.filter((f) => f.show).length;

    function handleDragOver(event: DragEvent, index) {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        const headerColumn = document.querySelector(`[data-index='${index}']`);
        const elementRect = headerColumn.getBoundingClientRect();
        const elementWidth = elementRect.width;
        const dragZoneWidth = 6; // 5% of the element's width

        const mouseX = event.clientX - elementRect.left;
        if (
            index !== $draggedColumnIdx &&
            index !== $draggedColumnIdx + 1 &&
            mouseX < dragZoneWidth
        ) {
            // User is dragging over the sides (5% of width) of the element
            columnToInsertIdx = index;
            columnToInsertXPos = elementRect.left;
            // Add your logic here
        } else if (
            index !== $draggedColumnIdx - 1 &&
            index !== $draggedColumnIdx &&
            index < numColumns - 1 &&
            mouseX > elementWidth - dragZoneWidth
        ) {
            columnToInsertIdx = index + 1;
            columnToInsertXPos = elementRect.right;
        } else {
            columnToInsertIdx = null;
            columnToInsertXPos = 0;
        }
    }

    $: {
        console.log(
            "start",
            $draggedColumnIdx,
            "end",
            dropColumnIdx,
            "columninsert",
            columnToInsertIdx
        );
    }

    let dropColumnIdx = null;

    $: {
        if ($fileDropHandler === "library-header" && $emptyDropEvent) {
            if (columnToInsertIdx !== null) {
                insertColumn($draggedColumnIdx, columnToInsertIdx);
            } else {
                swapColumns($draggedColumnIdx, dropColumnIdx);
            }
        }
    }

    function insertColumn(oldIndex, newIndex) {
        fields = moveArrayElement(
            fields.filter((f) => f.show),
            oldIndex,
            newIndex
        );
        resetColumnOrderUi();
    }

    function swapColumns(oldIndex, newIndex) {
        console.log(oldIndex, newIndex);
        fields = swapArrayElements(
            fields.filter((f) => f.show),
            oldIndex,
            newIndex
        );
        resetColumnOrderUi();
    }

    function resetColumnOrderUi() {
        $fileDropHandler = null;
        $emptyDropEvent = null;
        dropColumnIdx = null;
        $draggedColumnIdx = null;
        columnToInsertIdx = null;
        columnToInsertXPos = 0;
    }

    // Sets back to default
    function resetColumnOrder() {
        fields = DEFAULT_FIELDS;
    }
</script>

{#if isLoading}
    <div class="loading" out:fade={{ duration: 90, easing: cubicInOut }}>
        <p>💿 one sec...</p>
    </div>
{:else if theme === "default" && ($importStatus.isImporting || (noSongs && $query.query.length === 0 && $uiView !== "smart-query"))}
    <ImportPlaceholder />
{:else}
    <div class="library-container">
        <container
            bind:this={container}
            on:scroll={debounce(onScroll, 200)}
            class="theme-{theme}"
            in:fade={{ duration: 170, delay: 100, easing: cubicInOut }}
        >
            <library>
                <table
                    class:discography-sort={$query.orderBy === "artist"}
                    class:smart-query={$uiView === "smart-query"}
                >
                    <thead
                        class:smart-query={$uiView === "smart-query"}
                        on:contextmenu|preventDefault={(e) => {
                            columnPickerPos = { x: e.clientX, y: e.clientY };
                            showColumnPicker = !showColumnPicker;
                        }}
                    >
                        {#each tableHeaders as header (header)}
                            <tr
                                animate:flip={{
                                    duration: 220,
                                    easing: quadOut
                                }}
                            >
                                {#if header === "smart-query-builder"}
                                    <td
                                        class="query-header-cell builder"
                                        colspan={fields.length}
                                    >
                                        <SmartQueryBuilder />
                                    </td>
                                {/if}

                                {#if header === "smart-query"}
                                    <td
                                        class="query-header-cell query"
                                        colspan={fields.length}
                                    >
                                        <SmartQueryMainHeader />
                                    </td>
                                {/if}

                                {#if header === "track-fields"}
                                    {#each fields.filter((f) => f.show) as field, idx (field.value)}
                                        <td
                                            data-index={idx}
                                            draggable="true"
                                            on:dragstart={(e) =>
                                                handleDragStart(e, idx)}
                                            on:dragenter={(e) =>
                                                handleDragEnter(e, idx)}
                                            on:dragover={(e) => {
                                                handleDragOver(e, idx);
                                            }}
                                            on:dragend={resetColumnOrderUi}
                                            class:active={$query.orderBy ===
                                                field.value}
                                            class:dragging={$draggedColumnIdx ===
                                                idx}
                                            data-type={field.value}
                                            class:drop-highlight={columnToInsertIdx ===
                                                null &&
                                                dropColumnIdx === idx &&
                                                dropColumnIdx !==
                                                    $draggedColumnIdx}
                                            on:click={() =>
                                                updateOrderBy(field.value)}
                                            on:mouseenter={() => {
                                                hoveredColumnIdx = idx;
                                            }}
                                            on:mouseleave={() => {
                                                hoveredColumnIdx = null;
                                            }}
                                            use:optionalTippy={{
                                                allowHTML: true,
                                                content:
                                                    "Sorting by artist = viewing the discography<br/><br/> 🕺 Artists shown in alphabetical order <br/> 💿 Albums in chronological order <br/> 🎵 Tracks in album order <br/>",
                                                placement: "bottom",
                                                show: field.value === "artist"
                                            }}
                                            ><div>
                                                {#if hoveredColumnIdx === idx}
                                                    <iconify-icon
                                                        icon="nimbus:drag-dots"
                                                    />
                                                {/if}
                                                <p>
                                                    {field.name}
                                                </p>

                                                <iconify-icon
                                                    class="icon-clock"
                                                    icon="akar-icons:clock"
                                                />

                                                {#if $query.orderBy === field.value}
                                                    {#if $query.reverse}
                                                        <iconify-icon
                                                            class="icon-order"
                                                            icon="heroicons-solid:sort-ascending"
                                                        />
                                                    {:else}
                                                        <iconify-icon
                                                            class="icon-order"
                                                            icon="heroicons-solid:sort-descending"
                                                        />
                                                    {/if}
                                                {/if}
                                            </div></td
                                        >
                                    {/each}
                                {/if}
                            </tr>
                        {/each}
                    </thead>

                    {#if songs}
                        {#each songs as song, idx (song.id)}
                            <tr
                                data-song={song.id}
                                class:playing={get(currentSong) &&
                                    song.id === $currentSong.id}
                                class:just-added={$songsJustAdded.length < 50 &&
                                    isSongJustAdded(song.id)}
                                class:highlight={songsHighlighted &&
                                    isSongHighlighted(song)}
                                on:contextmenu|preventDefault={(e) =>
                                    onRightClick(e, song, idx)}
                                on:click={() => toggleHighlight(song, idx)}
                                on:dblclick={() => onDoubleClickSong(song, idx)}
                                on:mousedown|preventDefault={(e) =>
                                    e.button === 0 && onSongDragStart(song)}
                            >
                                {#each fields.filter((f) => f.show) as field (field)}
                                    <td
                                        data-type={field.value}
                                        class:is-first={(field.value ===
                                            "artist" &&
                                            song.viewModel?.isFirstArtist) ||
                                            (field.value === "album" &&
                                                song.viewModel?.isFirstAlbum)}
                                    >
                                        <div class="field">
                                            <p
                                                on:click|stopPropagation={() => {
                                                    filterByField(
                                                        field.value,
                                                        song[field.value]
                                                    );
                                                }}
                                                class:my-artist={field.value ===
                                                    "artist" &&
                                                    $artists &&
                                                    $artists.includes(
                                                        song[field.value]
                                                    )}
                                            >
                                                {#if field.value === "originCountry"}
                                                    <span>
                                                        {getFlagEmoji(
                                                            codes[
                                                                song[
                                                                    field.value
                                                                ]
                                                            ]
                                                        )}
                                                    </span>
                                                {/if}
                                                {song[field.value] === "" ||
                                                song[field.value] === -1 ||
                                                song[field.value] === 0 ||
                                                song[field.value] === null ||
                                                song[field.value] === undefined
                                                    ? "-"
                                                    : song[field.value]}
                                            </p>
                                            {#if field.value === "title"}
                                                <div class="title-icons-left">
                                                    {#if get(currentSong) && song.id === $currentSong.id}
                                                        <iconify-icon
                                                            icon="heroicons-solid:volume-up"
                                                        />
                                                    {/if}
                                                </div>
                                            {/if}

                                            <div class="title-icons-right">
                                                {#if field.value === "title" && song.isFavourite}
                                                    <iconify-icon
                                                        class="favourite-solid"
                                                        icon="clarity:heart-solid"
                                                        on:click|stopPropagation={() =>
                                                            unfavouriteSong(
                                                                song
                                                            )}
                                                    />
                                                {:else if field.value === "title"}
                                                    <iconify-icon
                                                        class="favourite-outline"
                                                        icon="clarity:heart-line"
                                                        on:click|stopPropagation={() =>
                                                            favouriteSong(song)}
                                                    />
                                                {/if}
                                            </div>
                                        </div>
                                    </td>
                                {/each}
                            </tr>
                        {/each}
                    {/if}
                </table>

                {#if $isSmartQueryBuilderOpen && noSongs}
                    <SmartQueryResultsPlaceholder />
                {/if}
                {#if theme === "default"}
                    <div>
                        <button
                            style="margin-top:2em"
                            on:click={openTauriImportDialog}>Add music +</button
                        >
                    </div>
                    <bottom-bar>
                        <div class="left">
                            <div class="lossy-selector">
                                <div
                                    class:selected={compressionSelected ===
                                        "lossy"}
                                    on:click={() =>
                                        (compressionSelected = "lossy")}
                                >
                                    <p>Lossy</p>
                                </div>
                                <div
                                    class:selected={compressionSelected ===
                                        "lossless"}
                                    on:click={() =>
                                        (compressionSelected = "lossless")}
                                >
                                    <p>Lossless</p>
                                </div>
                                <div
                                    class:selected={compressionSelected ===
                                        "both"}
                                    on:click={() =>
                                        (compressionSelected = "both")}
                                >
                                    <p>both</p>
                                </div>
                            </div>
                            {#if $nextUpSong}
                                <div class="next-up">
                                    <p class="label">Next up:</p>
                                    <p>{$nextUpSong.title}</p>
                                </div>
                            {/if}
                        </div>
                        <p>{$count} songs</p>
                        <p>{$artistCount} artists</p>
                        <p>{$albumCount} albums</p>
                    </bottom-bar>
                {/if}
            </library>
        </container>
        {#if $isPlaying && $currentSong}
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
    </div>
{/if}

<TrackMenu bind:showMenu={showTrackMenu} bind:pos />
<ColumnPicker
    bind:showMenu={showColumnPicker}
    bind:pos={columnPickerPos}
    bind:fields
    onResetOrder={resetColumnOrder}
    isOrderChanged={fields !== DEFAULT_FIELDS}
/>

{#if columnToInsertIdx !== null && columnToInsertXPos}
    <div class="column-insert-hint" style="left: {columnToInsertXPos}px" />
{/if}

<style lang="scss">
    $odd_color: transparent;
    $even_color: transparent;
    $selected_color: #5123dd;
    $playing_text_color: #00ddff;
    $highlight_color: #2e3357;
    $text_color: rgb(211, 211, 211);
    $added_color: rgb(44, 147, 44);
    $mini_y_breakpoint: 300px;

    .library-container {
        position: relative;
    }
    container {
        height: 100vh;
        background-color: rgba(36, 33, 34, 0.688);
        position: relative;
        display: flex;
        flex-direction: column;
        overflow: overlay;
        border-bottom: 1px solid rgb(51, 51, 51);

        &.theme-outline {
            background-color: transparent;
            height: fit-content;
            border-bottom: none;
        }
        @media only screen and (max-width: 320px) {
            display: none;
        }
    }

    header {
        position: sticky;
        top: 0;
    }

    library {
        position: relative;
        display: grid;
        height: 100%;
        grid-template-rows: auto 1fr auto;
    }

    .query-header-cell {
        padding: 0.4em 1em;
        display: table-cell;
        width: 100%;
        background-color: #4d347c;
        border-bottom: 1px solid rgba(237, 194, 250, 0.1);
        cursor: default !important;

        &.query {
            background-color: rgba(77, 52, 124, 0.8);
            &:hover {
                background-color: rgba(77, 52, 124, 0.8);
            }
        }

        &.builder {
            background-color: rgba(77, 52, 124, 0.8);
            border-bottom: 1px solid rgba(255, 255, 255, 0.097);
            &:hover {
                background-color: rgba(77, 52, 124, 0.8);
            }
        }

        &:hover {
            background-color: #4d347c;
        }
    }

    bottom-bar {
        position: sticky;
        bottom: 0;
        left: 0;
        right: 0;
        background-color: rgba(28, 26, 26, 0.645);
        backdrop-filter: blur(8px);
        border-top: 1px solid rgb(51, 51, 51);
        color: white;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 1em;
        justify-content: flex-end;
        padding: 0 1em;
        width: 100%;
        height: 30px;

        p {
            color: rgb(176, 161, 161);
            margin: 0;
        }
        @media only screen and (max-width: 522px) {
            p:not(:nth-child(1)) {
                display: none;
            }
        }

        .left {
            display: flex;
            flex-direction: row;
            flex-grow: 1;
            align-items: center;

            .next-up {
                display: flex;
                flex-direction: row;
                gap: 5px;
                .label {
                    opacity: 0.5;
                }
                padding: 0 1.5em;
                p {
                    margin: 0;
                    font-size: 0.9em;
                }
            }

            .lossy-selector {
                display: flex;
                flex-direction: row;
                background-color: #24232332;
                /* border: 1px solid rgba(128, 128, 128, 0.29); */
                border-radius: 3px;
                overflow: hidden;
                > div {
                    padding: 1px 10px;
                    cursor: default;
                    &:hover {
                        background-color: #bbb9b92e;
                    }
                    &:active {
                        background-color: #bbb9b923;
                    }
                    &.selected {
                        p {
                            color: rgb(224, 218, 218);
                        }
                        background-color: #35309784;
                    }
                    p {
                        margin: 0;
                        line-height: 1.3em;
                        user-select: none;
                        text-transform: lowercase;
                    }
                }
            }
        }
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
        z-index: 10;
        display: inline-flex;
        align-items: center;
        gap: 10px;
        cursor: default;
        user-select: none;

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
                -webkit-animation-name: shorteq;
                -webkit-animation-duration: 0.5s;
                -webkit-animation-iteration-count: infinite;
                animation-delay: 0s;
            }

            .eq2 {
                height: 15px;
                left: 6px;
                animation-name: talleq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.17s;
                -webkit-animation-name: talleq;
                -webkit-animation-duration: 0.5s;
                -webkit-animation-iteration-count: infinite;
                animation-delay: 0.17s;
            }

            .eq3 {
                height: 13px;
                left: 12px;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.34s;
                -webkit-animation-name: shorteq;
                -webkit-animation-duration: 0.5s;
                -webkit-animation-iteration-count: infinite;
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
    table {
        -webkit-border-horizontal-spacing: 0px;
        -webkit-border-vertical-spacing: 0px;
        width: 100%;
        overflow: auto;
        &.discography-sort:not(.smart-query) {
            > tr > td[data-type="album"] {
                color: $text_color;
                &.is-first {
                    color: white;
                    border-top: 0.5px solid rgba(128, 128, 128, 0.422);
                    position: sticky;
                    top: 2em;
                    background-color: rgb(36, 33, 34);
                }
            }
            > tr > td[data-type="artist"] {
                color: $text_color;
                &.is-first {
                    color: white;
                    border-top: 0.5px solid rgba(128, 128, 128, 0.422);
                    position: sticky;
                    top: 2em;
                    background-color: rgb(36, 33, 34);
                }
            }
            > tr.highlight > td[data-type="artist"],
            > tr.highlight > td[data-type="album"] {
                color: $text_color;
                &.is-first {
                    color: white;
                    /* border-top: 0.5px solid rgba(128, 128, 128, 0.422); */
                    position: sticky;
                    top: 2em;
                    background: $highlight_color;
                }
            }
            > tr.playing > td[data-type="artist"],
            > tr.playing > td[data-type="album"] {
                color: $playing_text_color;
                &.is-first {
                    color: white;
                    /* border-top: 0.5px solid rgba(128, 128, 128, 0.422); */
                    position: sticky;
                    top: 2em;
                    background: #5123dd;
                }
            }
        }

        thead {
            font-weight: bold;
            position: sticky;
            top: 0;
            backdrop-filter: blur(8px);
            &.smart-query {
                td {
                    background-color: #4d347c;
                }
            }
            td {
                .theme-outline & {
                    background-color: transparent;
                }
                background-color: #71658e7e;
                border-right: none;
                overflow: hidden;
                text-overflow: ellipsis;
                position: relative;
                div {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                }
                &:hover {
                    background-color: #604d8d;
                }
                &.active {
                    background-color: #604d8d;
                }

                &.drop-highlight {
                    background-color: #b399ffca;
                }

                iconify-icon[icon="nimbus:drag-dots"] {
                    left: 0;
                    position: absolute;
                    z-index: 2;
                    color: grey;
                }

                &:active {
                    cursor: grabbing;
                }
            }
        }

        td {
            text-align: left;
            user-select: none;
            cursor: default;
            padding-inline-start: 1em;
            padding-inline-end: 1em;
            border-right: 0.5px solid rgba(242, 242, 242, 0.144);
            max-width: 120px;
            overflow: hidden;
            .icon-clock {
                display: none;
            }
            &[data-type="track"] {
                max-width: min-content;
            }

            p {
                margin: 0;
                text-overflow: ellipsis;
                overflow: hidden;
                white-space: nowrap;
                &.my-artist {
                    &::before {
                        content: "© ";
                        color: #855cff;
                    }
                }
            }
            @media only screen and (max-width: 522px) {
                &:not(:nth-child(1)) {
                    display: none;
                }
            }
            @media only screen and (max-width: 750px) {
                &:nth-child(3) ~ td {
                    display: none;
                }
            }
            @media only screen and (max-width: 870px) {
                &:nth-child(5) ~ td {
                    display: none;
                }
            }
            @media only screen and (max-width: 950px) {
                &[data-type="duration"] {
                    p {
                        display: none;
                    }
                    .icon-clock {
                        display: block;
                    }
                }
            }
            &[data-type="title"] {
                max-width: 160px;
            }
            @media only screen and (min-width: 950px) {
                &[data-type="artist"] {
                    max-width: 120px;
                }
                &[data-type="album"] {
                    max-width: 120px;
                }
                &[data-type="trackNumber"] {
                    max-width: 52px;
                }
                &[data-type="genre"] {
                    max-width: 55px;
                }
                &[data-type="year"] {
                    max-width: 51px;
                }
                &[data-type="duration"] {
                    max-width: 60px;
                    .icon-clock {
                        display: none;
                    }
                }
            }

            .field {
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
                display: flex;
                align-items: center;
                justify-content: stretch;

                .title-icons-left {
                    margin-left: 10px;
                    display: flex;
                    align-items: center;
                    flex-grow: 0;
                }

                .title-icons-right {
                    display: flex;
                    flex-grow: 1;
                    justify-content: flex-end;
                    .favourite-outline {
                        display: none;
                        color: #784bff;
                    }
                    .favourite-solid {
                        color: #5123dd;
                    }
                }
            }
        }

        > tr {
            contain: strict;
            content-visibility: auto;
            contain-intrinsic-height: auto 26px;
            white-space: nowrap;
            user-select: none;
            color: $text_color;

            .theme-outline & {
                color: #bbb9b9;
                &.highlight {
                    color: white;
                }
            }
            &:hover {
                background-color: #1f1f1f;
            }

            > td {
                font-size: 13px;
                text-overflow: clip;

                .theme-outline & {
                    border-right: none;
                }
                background-color: $odd_color;

                &[data-type="title"],
                &[data-type="artist"],
                &[data-type="album"],
                &[data-type="composer"],
                &[data-type="trackNumber"] {
                    p {
                        pointer-events: none;
                    }
                }

                &[data-type="genre"],
                &[data-type="year"],
                &[data-type="originCountry"] {
                    p {
                        /* border: 1px solid rgb(63, 61, 61); */
                        background-color: rgba(255, 255, 255, 0.03);
                        border-radius: 3px;
                        padding: 0 4px;

                        &:hover {
                            background-color: $highlight_color;
                        }
                        &:active {
                            background-color: $selected_color;
                        }
                    }
                }
            }
            &.highlight {
                background-color: $highlight_color !important;

                .favourite-outline {
                    display: block !important;
                }
            }
            &.playing {
                background: $selected_color !important;
                color: $playing_text_color;
                .title-icons-right {
                    .favourite-outline {
                        display: none;
                        color: #784bff;
                    }
                    .favourite-solid {
                        color: $playing_text_color;
                    }
                }
            }
            &.just-added {
                background-color: $added_color !important;
                color: white;
            }
            /* &:nth-child(odd) {
                
            } */
            /* &:nth-child(even) {
                background-color: $even_color;

                &:hover {
                    background-color: #1f1f1f;

                    .favourite-outline {
                        display: block;
                    }
                }
            } */
        }
    }

    .column-insert-hint {
        position: fixed;
        width: 2px;
        background-color: #b399ffca;
        height: 100vh;
        left: 0;
        pointer-events: none;
    }

    .loading {
        margin: auto;
        p {
            opacity: 0.6;
        }
    }
</style>
