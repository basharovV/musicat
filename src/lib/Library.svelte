<script lang="ts">
    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import "iconify-icon";
    import type { Song } from "src/App";
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
        isSmartQueryValid,
        isTrackInfoPopupOpen,
        os,
        queriedSongs,
        query,
        rightClickedTrack,
        rightClickedTracks,
        selectedSmartQuery,
        singleKeyShortcutsEnabled,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        songsJustAdded
    } from "../data/store";
    import AudioPlayer from "./AudioPlayer";
    import ImportPlaceholder from "./ImportPlaceholder.svelte";
    import SmartQuery from "./smart-query/Query";
    import SmartQueryBuilder from "./smart-query/SmartQueryBuilder.svelte";
    import SmartQueryMainHeader from "./smart-query/SmartQueryMainHeader.svelte";
    import SmartQueryResultsPlaceholder from "./smart-query/SmartQueryResultsPlaceholder.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    // TODO
    async function calculateSize(songs: Song[]) {}

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;
        if ($isSmartQueryUiOpen) {
            /**
             * User-built smart queries don't support indexing
             */
            if ($isSmartQueryBuilderOpen) {
                if ($smartQuery.isEmpty) {
                    results = [];
                } else if ($smartQueryUpdater) {
                    results = await $smartQuery.run();
                }

                isSmartQueryResults = true;
                isIndexed = false;
            } else {
                /**
                 * Built-in smart queries support indexing, so they return a
                 * Collection instead of an array.
                 */
                console.log("selected query: ", $selectedSmartQuery);
                if ($selectedSmartQuery.startsWith("~usq:")) {
                    // Run the query from the user-built blocks
                    const queryName = $selectedSmartQuery.substring(5);
                    const savedQuery = await db.smartQueries.get(queryName);
                    const query = new SmartQuery(savedQuery);
                    results = await query.run();
                    console.log("results query: ", results);
                    isIndexed = false;
                } else {
                    // Run the query from built-in functions
                    results = await BuiltInQueries.find(
                        (q) => q.value === $selectedSmartQuery
                    ).query();

                    isIndexed = true;
                }
                isSmartQueryResults = true;
            }
        } else if ($query.query.length) {
            results = db.songs
                .where("title")
                .startsWithIgnoreCase($query.query)
                .or("artist")
                .startsWithIgnoreCase($query.query)
                .or("album")
                .startsWithIgnoreCase($query.query);
        } else {
            results = db.songs.orderBy(
                $query.orderBy === "artist"
                    ? "[artist+album+trackNumber]"
                    : $query.orderBy === "album"
                    ? "[album+trackNumber]"
                    : $query.orderBy
            );
        }
        let resultsArray: Song[] = [];

        // Depending whether this is a smart query or not
        if (isIndexed) {
            if ($query.reverse) {
                results = results.reverse();
            }
            resultsArray = await results.toArray();
        } else {
            resultsArray = results;
        }

        // Do sorting for non-indexed results
        if (!isIndexed) {
            resultsArray = resultsArray.sort((a, b) => {
                switch ($query.orderBy) {
                    case "title":
                    case "album":
                    case "track":
                    case "year":
                    case "duration":
                    case "genre":
                        return a[$query.orderBy].localeCompare(
                            b[$query.orderBy]
                        );
                    case "artist":
                        // TODO this one needs to match the multiple indexes sorting from Dexie
                        // i.e Artist -> Album -> Track N.
                        return a.artist.localeCompare(b.artist);
                }
            });
        }

        /**
         * Set in store
         */
        if (isSmartQueryResults) {
            smartQueryResults.set(resultsArray);
        } else {
            queriedSongs.set(resultsArray);
        }

        if ($query.query.length && resultsArray.length === 1) {
            // Highlight the only result
            highlightSong(resultsArray[0], 0, false);
        } else {
            // Keep it highlighted after?
        }
        return resultsArray;
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

    let songIdsHighlighted: Song[] = [];

    function isSongHighlighted(song: Song) {
        return songIdsHighlighted.map((s) => s.id).includes(song.id);
    }

    function toggleHighlight(song, idx, isKeyboardArrows = false) {
        if (!song) song = $songs[0];
        highlightedSongIdx = idx;
        if (isSongHighlighted(song)) {
            if (songIdsHighlighted.length) {
                songIdsHighlighted = [];
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
    // hotkeys("up", function (event, handler) {});

    // hotkeys("down", function (event, handler) {});
    addEventListener("keydown", (event) => {
        if (event.keyCode === 16) {
            isShiftPressed = true;
            console.log("shift pressed");
        } else if (
            event.keyCode === 38 &&
            document.activeElement.tagName.toLowerCase() !== "input"
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
            document.activeElement.tagName.toLowerCase() !== "input"
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
            document.activeElement.tagName.toLowerCase() !== "input"
        ) {
            // 'i' for info popup
            event.preventDefault();
            console.log("active element", document.activeElement.tagName);
            // Check if there an input in focus currently
            if (!$isTrackInfoPopupOpen && songIdsHighlighted.length) {
                $rightClickedTrack = songIdsHighlighted[0];
                $isTrackInfoPopupOpen = true;
            }
        } else if (
            event.keyCode === 13 &&
            !$isTrackInfoPopupOpen &&
            document.activeElement.tagName.toLowerCase() !== "input"
        ) {
            // 'Enter' to play highlighted track
            event.preventDefault();
            if (!$isTrackInfoPopupOpen) {
                $currentSongIdx = highlightedSongIdx;
                AudioPlayer.playSong(songIdsHighlighted[0]);
            }
        }
    });
    addEventListener("keyup", (event) => {
        if (event.keyCode === 16) {
            isShiftPressed = false;
            console.log("shift lifted");
        }
    });

    function highlightSong(song: Song, idx, isKeyboardArrows: boolean) {
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
                songIdsHighlighted = $songs.slice(
                    rangeStartSongIdx,
                    rangeEndSongIdx + 1
                );
                rangeStartSongIdx = null;
                rangeEndSongIdx = null;
                $rightClickedTrack = null;
                console.log("highlighted2", songIdsHighlighted);
            }
        } else if (
            (isKeyboardArrows && isShiftPressed) ||
            isCmdOrCtrlPressed ||
            hotkeys.isPressed(91)
        ) {
            songIdsHighlighted.push(song);
            rangeStartSongIdx = idx;
        } else {
            // Highlight single song, via a good old click
            songIdsHighlighted = [song];
            highlightedSongIdx = idx;
            $rightClickedTracks = [];
            $rightClickedTracks = $rightClickedTracks;
            rangeStartSongIdx = idx;

            // Extra - if the Info overlay is shown, use the arrows to replace the track shown in the overlay
            if ($isTrackInfoPopupOpen && isKeyboardArrows) {
                $rightClickedTrack = song;
            }
        }
        console.log("start", rangeStartSongIdx);

        songIdsHighlighted = songIdsHighlighted;
    }

    function unhighlightSong(song: Song) {
        songIdsHighlighted.splice(songIdsHighlighted.indexOf(song), 1);
        songIdsHighlighted = songIdsHighlighted;
    }

    let showTrackMenu = false;
    let pos;

    function onRightClick(e, song, idx) {
        if (!songIdsHighlighted.includes(song)) {
            highlightSong(song, idx, false);
        }

        console.log("songIdsHighlighted", songIdsHighlighted);
        if (songIdsHighlighted.length > 1) {
            $rightClickedTracks = songIdsHighlighted;
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
        if ($isSmartQueryUiOpen) {
            showSmartQuery();
        } else {
            hideSmartQuery();
        }

        if ($isSmartQueryBuilderOpen) {
            showSmartQueryBuilder();
        } else {
            hideSmartQueryBuilder();
        }
    }
</script>

{#if $importStatus.isImporting || (noSongs && $query.query.length === 0 && !$isSmartQueryBuilderOpen)}
    <ImportPlaceholder />
{:else}
    <container>
        <library>
            <table>
                <thead class:smart-query={$isSmartQueryUiOpen}>
                    {#each tableHeaders as header (header)}
                        <tr animate:flip={{ duration: 220, easing: quadOut }}>
                            {#if header === "smart-query-builder"}
                                <td
                                    class="query-header-cell builder"
                                    colspan="7"
                                >
                                    <SmartQueryBuilder />
                                </td>
                            {/if}

                            {#if header === "smart-query"}
                                <td class="query-header-cell query" colspan="7">
                                    <SmartQueryMainHeader />
                                </td>
                            {/if}

                            {#if header === "track-fields"}
                                <td
                                    data-type="title"
                                    on:click={() => updateOrderBy("title")}
                                    ><div>
                                        Title
                                        {#if $query.orderBy === "title"}
                                            {#if $query.reverse}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-ascending"
                                                />
                                            {:else}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-descending"
                                                />
                                            {/if}
                                        {/if}
                                    </div></td
                                >
                                <td
                                    data-type="artist"
                                    on:click={() => updateOrderBy("artist")}
                                    use:tippy={{
                                        allowHTML: true,
                                        content:
                                            "Artist sort shows tracks like this:<br/>Artist > Album > Track №. <br/><br/>Albums for that artist are in alphabetical order, and tracks are in correct album order.",
                                        placement: "bottom"
                                    }}
                                    ><div>
                                        Artist
                                        {#if $query.orderBy === "artist"}
                                            {#if $query.reverse}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-ascending"
                                                />
                                            {:else}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-descending"
                                                />
                                            {/if}
                                        {/if}
                                    </div></td
                                >
                                <td
                                    data-type="album"
                                    on:click={() => updateOrderBy("album")}
                                    ><div>
                                        Album
                                        {#if $query.orderBy === "album"}
                                            {#if $query.reverse}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-ascending"
                                                />
                                            {:else}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-descending"
                                                />
                                            {/if}
                                        {/if}
                                    </div></td
                                >
                                <td data-type="track"><div>Track</div></td>
                                <td
                                    data-type="year"
                                    on:click={() => updateOrderBy("year")}
                                    ><div>
                                        Year
                                        {#if $query.orderBy === "year"}
                                            {#if $query.reverse}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-ascending"
                                                />
                                            {:else}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-descending"
                                                />
                                            {/if}
                                        {/if}
                                    </div></td
                                >
                                <td
                                    data-type="genre"
                                    on:click={() => updateOrderBy("genre")}
                                    ><div>
                                        Genre
                                        {#if $query.orderBy === "genre"}
                                            {#if $query.reverse}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-ascending"
                                                />
                                            {:else}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-descending"
                                                />
                                            {/if}
                                        {/if}
                                    </div></td
                                >
                                <td
                                    data-type="duration"
                                    on:click={() => updateOrderBy("duration")}
                                    ><div>
                                        <p>Duration</p>
                                        <iconify-icon icon="akar-icons:clock" />
                                        {#if $query.orderBy === "duration"}
                                            {#if $query.reverse}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-ascending"
                                                />
                                            {:else}
                                                <iconify-icon
                                                    icon="heroicons-solid:sort-descending"
                                                />
                                            {/if}
                                        {/if}
                                    </div></td
                                >
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
                            class:highlight={songIdsHighlighted &&
                                isSongHighlighted(song)}
                            on:contextmenu|preventDefault={(e) =>
                                onRightClick(e, song, idx)}
                            on:click={() => toggleHighlight(song, idx)}
                            on:dblclick={() => onDoubleClickSong(song, idx)}
                        >
                            <td data-type="title">
                                <div>
                                    <p>
                                        {song.title === "" ? "-" : song.title}
                                    </p>
                                    {#if get(currentSong) && song.id === $currentSong.id}
                                        <iconify-icon
                                            icon="heroicons-solid:volume-up"
                                        />
                                    {/if}
                                </div>
                            </td>
                            <td data-type="artist"
                                ><p>
                                    {song.artist === "" ? "-" : song.artist}
                                </p></td
                            >
                            <td data-type="album">
                                <p>
                                    {song.album === "" ? "-" : song.album}
                                </p></td
                            >
                            <td data-type="track"
                                >{song.trackNumber === -1 ||
                                song.trackNumber === null
                                    ? "-"
                                    : song.trackNumber}
                            </td>
                            <td data-type="year"
                                >{song.year === 0 ? "-" : song.year}</td
                            >
                            <td data-type="genre"
                                >{song.genre?.length ? song.genre : "-"}</td
                            >
                            <td data-type="duration">{song.duration}</td>
                        </tr>
                    {/each}
                {/if}
            </table>

            {#if $isSmartQueryBuilderOpen && noSongs}
                <SmartQueryResultsPlaceholder />
            {/if}
            <div>
                <button style="margin-top:2em" on:click={openTauriImportDialog}
                    >Add music +</button
                >
            </div>
            <bottom-bar>
                <p>{$count} songs</p>
                <p>{$artistCount} artists</p>
                <p>{$albumCount} albums</p>
            </bottom-bar>
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

            &[data-type="track"] {
                max-width: min-content;
            }

            p {
                margin: 0;
                text-overflow: ellipsis;
                overflow: hidden;
                white-space: nowrap;
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
                    iconify-icon {
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
