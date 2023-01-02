<script lang="ts">
    import { liveQuery } from "dexie";
    import "iconify-icon";
    import type { Song } from "src/App";
    import { get } from "svelte/store";
    import { db } from "../../data/db";
    import BuiltInQueries from "../../data/SmartQueries";
    import {
        isSmartQueryBuilderOpen,
        playlist,
        playlistIsAlbum,
        queriedSongs,
        query,
        selectedPlaylistId,
        selectedSmartQuery,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        uiView
    } from "../../data/store";
    import Library from "../Library.svelte";
    import SmartQuery from "../smart-query/Query";

    let isLoading = true;

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;

        if ($selectedPlaylistId !== null) {
            const playlist = await db.playlists.get($selectedPlaylistId);
            console.log('playlist to get', playlist);
            results = await db.songs.bulkGet(playlist.tracks);
            isIndexed = false;
        } else if ($uiView === "smart-query") {
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
                .orderBy(
                    $query.orderBy === "artist"
                        ? "[artist+year+album+trackNumber]"
                        : $query.orderBy === "album"
                        ? "[album+trackNumber]"
                        : $query.orderBy
                )
                .and(
                    (song) =>
                        song.title
                            .toLowerCase()
                            .startsWith($query.query.toLowerCase()) ||
                        song.artist
                            .toLowerCase()
                            .startsWith($query.query.toLowerCase()) ||
                        song.album
                            .toLowerCase()
                            .startsWith($query.query.toLowerCase())
                );
        } else {
            results = db.songs.orderBy(
                $query.orderBy === "artist"
                    ? "[artist+year+album+trackNumber]"
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

        isLoading = false;
        $playlist = resultsArray;
        $playlistIsAlbum = false;
        return resultsArray;
    });
</script>

<Library allSongs={songs} showMyArtists={true} {isLoading} />
