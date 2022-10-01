<script lang="ts">
    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import "iconify-icon";
    import type { Song } from "src/App";
    import { onDestroy } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { flip } from "svelte/animate";
    import { quadOut } from "svelte/easing";
    import { get } from "svelte/store";
    import { db } from "../data/db";
    import { openTauriImportDialog } from "../data/LibraryImporter";
    import BuiltInQueries from "../data/SmartQueries";
    import {
        currentSong,
        currentSongIdx,
        importStatus,
        isSmartQueryBuilderOpen,
        isSmartQueryUiOpen,
        isTrackInfoPopupOpen,
        os,
        query,
        rightClickedTrack,
        rightClickedTracks,
        singleKeyShortcutsEnabled,
        smartQuery,
        songsJustAdded,
        uiView
    } from "../data/store";
    import AudioPlayer from "./AudioPlayer";
    import ImportPlaceholder from "./ImportPlaceholder.svelte";
    import SmartQueryBuilder from "./smart-query/SmartQueryBuilder.svelte";
    import SmartQueryMainHeader from "./smart-query/SmartQueryMainHeader.svelte";
    import SmartQueryResultsPlaceholder from "./smart-query/SmartQueryResultsPlaceholder.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    // TODO
    async function calculateSize(songs: Song[]) {}

    export let songs;
    export let fields = [
        { name: "Title", value: "title" },
        { name: "Artist", value: "artist" },
        { name: "Album", value: "album" },
        { name: "Track", value: "trackNumber" },
        { name: "Year", value: "year" },
        { name: "Genre", value: "genre" },
        { name: "Duration", value: "duration" }
    ];

    export let showMyArtists = false;

    export let theme = "default";
    export let isSmartQueryEnabled = true; // Only for main view
    export let onSongsHighlighted = null;

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
        !$songs ||
        $songs.length === 0 ||
        ($isSmartQueryBuilderOpen && $smartQuery.isEmpty);

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
        if (!song) song = $songs[0];
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
            document.activeElement.tagName.toLowerCase() !== "input" &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            event.preventDefault();
            // up
            if (highlightedSongIdx > 0) {
                toggleHighlight(
                    $songs[highlightedSongIdx - 1],
                    highlightedSongIdx - 1,
                    true
                );
            }
        } else if (
            event.keyCode === 40 &&
            document.activeElement.tagName.toLowerCase() !== "input" &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // down
            event.preventDefault();
            if (highlightedSongIdx < $songs.length) {
                toggleHighlight(
                    $songs[highlightedSongIdx + 1],
                    highlightedSongIdx + 1,
                    true
                );
            }
        } else if (
            event.keyCode === 73 &&
            !$isTrackInfoPopupOpen &&
            $singleKeyShortcutsEnabled &&
            document.activeElement.tagName.toLowerCase() !== "input" &&
            document.activeElement.tagName.toLowerCase() !== "textarea"
        ) {
            // 'i' for info popup
            event.preventDefault();
            console.log("active element", document.activeElement.tagName);
            // Check if there an input in focus currently
            if (!$isTrackInfoPopupOpen && songsHighlighted.length) {
                console.log("opening info", songsHighlighted);
                $rightClickedTrack = songsHighlighted[0];
                $isTrackInfoPopupOpen = true;
            }
        } else if (
            event.keyCode === 13 &&
            !$isTrackInfoPopupOpen &&
            document.activeElement.tagName.toLowerCase() !== "input" &&
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
                songsHighlighted = $songs.slice(
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

    const onAudioEnded = () => {
        console.log("audio ended");
        playNext();
    };

    function playNext() {
        AudioPlayer.playSong($songs[++$currentSongIdx]);
    }

    // Play next automatically
    AudioPlayer.setAudioFinishedCallback(onAudioEnded);

    // Smart query stuff

    // Smart query - predefined or user-defined queries i.e smart playlists
    let smartQuerySelectedVal = BuiltInQueries[0].value;

    function hideSmartQueryBuilder() {
        if (tableHeaders[0] === "smart-query-builder") {
            tableHeaders.splice(0, 1);
        }
        tableHeaders = tableHeaders;
    }

    function showSmartQueryBuilder() {
        if (tableHeaders[0] === "smart-query") {
            tableHeaders.unshift("smart-query-builder");
            $isSmartQueryBuilderOpen = true;
        }
    }

    const showSmartQuery = () => {
        console.log("headers showSmartQuery", tableHeaders);

        const found = tableHeaders.findIndex((h) => h === "smart-query");
        if (found === -1) {
            tableHeaders.unshift("smart-query");
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
        if (isSmartQueryEnabled && $uiView === "smart-query") {
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
</script>

{#if theme === "default" && ($importStatus.isImporting || (noSongs && $query.query.length === 0 && $uiView !== "smart-query"))}
    <ImportPlaceholder />
{:else}
    <container class="theme-{theme}">
        <library>
            <table>
                <thead class:smart-query={$uiView === "smart-query"}>
                    {#each tableHeaders as header (header)}
                        <tr animate:flip={{ duration: 220, easing: quadOut }}>
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
                                {#each fields as field (field.value)}
                                    <td
                                        data-type={field.value}
                                        on:click={() =>
                                            updateOrderBy(field.value)}
                                        use:tippy={{
                                            allowHTML: true,
                                            content:
                                                "Artist sort shows tracks like this:<br/>Artist > Album > Track №. <br/><br/>Albums for that artist are in alphabetical order, and tracks are in correct album order.",
                                            placement: "bottom"
                                        }}
                                        ><div>
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

                {#if $songs}
                    {#each $songs as song, idx (song.id)}
                        <tr
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
                        >
                            {#each fields as field (field)}
                                <td data-type={field.value}>
                                    <div>
                                        <p
                                            class:my-artist={field.value ===
                                                "artist" &&
                                                $artists &&
                                               $artists.includes(song[field.value])}
                                        >
                                            {song[field.value] === "" ||
                                            song[field.value] === -1 ||
                                            song[field.value] === 0 ||
                                            song[field.value] === null
                                                ? "-"
                                                : song[field.value]}
                                        </p>
                                        {#if field.value === "title" && get(currentSong) && song.id === $currentSong.id}
                                            <iconify-icon
                                                icon="heroicons-solid:volume-up"
                                            />
                                        {/if}
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
                    <p>{$count} songs</p>
                    <p>{$artistCount} artists</p>
                    <p>{$albumCount} albums</p>
                </bottom-bar>
            {/if}
        </library>
    </container>
{/if}

<TrackMenu bind:showMenu={showTrackMenu} bind:pos />

<style lang="scss">
    $odd_color: transparent;
    $even_color: transparent;
    $selected_color: #5123dd;
    $playing_text_color: #00ddff;
    $highlight_color: #4b61dd45;
    $text_color: rgb(211, 211, 211);
    $added_color: rgb(44, 147, 44);
    $mini_y_breakpoint: 300px;

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
    }

    table {
        -webkit-border-horizontal-spacing: 0px;
        -webkit-border-vertical-spacing: 0px;
        width: 100%;
        overflow: auto;

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
                div {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                }
                &:hover {
                    cursor: ns-resize;
                    background-color: #604d8d;
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
                        content: '© ';
                        color:#855cff;
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
                &[data-type="track"] {
                    max-width: 55px;
                }
                &[data-type="genre"] {
                    max-width: 55px;
                }
                &[data-type="year"] {
                    max-width: 50px;
                }
                &[data-type="duration"] {
                    max-width: 60px;
                    iconify-icon {
                        display: none;
                    }
                }
            }

            div {
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
                display: flex;
                align-items: center;
                justify-content: space-between;
            }
        }

        > tr {
            white-space: nowrap;
            user-select: none;
            color: $text_color;

            .theme-outline & {
                color: #bbb9b9;
                &.highlight {
                    color: white;
                }
            }

            > td {
                font-size: 13px;
                text-overflow: clip;
            }
            &.highlight {
                background-color: $highlight_color !important;
            }
            &.playing {
                background: $selected_color !important;
                color: $playing_text_color;
            }
            &.just-added {
                background-color: $added_color !important;
                color: white;
            }
            &:nth-child(odd) {
                background-color: $odd_color;

                &:hover {
                    background-color: #1f1f1f;
                }
            }
            &:nth-child(even) {
                background-color: $even_color;

                &:hover {
                    background-color: #1f1f1f;
                }
            }
        }
    }
</style>
