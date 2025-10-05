<script lang="ts">
    import { liveQuery } from "dexie";
    import type { Song, SongOrder } from "src/App";
    import { get } from "svelte/store";
    import { songMatchesQuery } from "../../data/LibraryUtils";
    import { parsePlaylist } from "../../data/M3UUtils";
    import BuiltInQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import {
        libraryColumns,
        isInit,
        isLyricsHovered,
        isLyricsOpen,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isTagOrCondition,
        queriedSongs,
        query,
        queueMirrorsSearch,
        selectedPlaylistFile,
        selectedSmartQuery,
        selectedTags,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        toDeletePlaylist,
        uiView,
    } from "../../data/store";
    import { setQueue } from "../../data/storeHelper";
    import CanvasLibrary from "../library/CanvasLibrary.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import SmartQuery from "../smart-query/Query";

    export let songOrder: SongOrder;

    let isLoading = true;

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;

        if ($uiView === "to-delete") {
            if (
                $toDeletePlaylist === null ||
                $toDeletePlaylist.tracks.length === 0
            ) {
                results = [];
            } else {
                results = await db.songs.bulkGet($toDeletePlaylist.tracks);
            }
            // Add display ID
            results = results.map((s, idx) => ({
                ...s,
                viewModel: {
                    viewId: idx.toString(),
                },
            }));
            console.log("to delete songs", results);
            isIndexed = false;
        } else if ($selectedPlaylistFile !== null) {
            results = await parsePlaylist($selectedPlaylistFile);
            console.log("m3uresults", results);

            // Add display ID
            results = results.map((s, idx) => ({
                ...s,
                viewModel: {
                    viewId: idx.toString(),
                },
            }));
            isIndexed = false;
            // Filter within playlist
            if ($query.length) {
                results = results.filter((song) =>
                    songMatchesQuery(song, $query),
                );
            }
        } else if ($uiView === "smart-query" || $uiView === "favourites") {
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
                    const query =
                        await SmartQuery.loadWithUQI($selectedSmartQuery);
                    results = await query.run();
                    console.log("results query: ", results);
                    isIndexed = false;
                } else {
                    // Run the query from built-in functions
                    results = await BuiltInQueries[$selectedSmartQuery].run();
                    isIndexed = true;
                }
                isSmartQueryResults = true;
            }
        } else if ($query.length) {
            results = db.songs
                .orderBy(
                    songOrder.orderBy === "artist"
                        ? "[artist+year+album+trackNumber]"
                        : songOrder.orderBy === "album"
                          ? "[album+trackNumber]"
                          : songOrder.orderBy,
                )
                .and((song) => songMatchesQuery(song, $query));
        } else {
            results = db.songs.orderBy(
                songOrder.orderBy === "artist"
                    ? "[artist+year+album+trackNumber]"
                    : songOrder.orderBy === "album"
                      ? "[album+trackNumber]"
                      : songOrder.orderBy,
            );
        }
        let resultsArray: Song[] = [];

        // Depending whether this is a smart query or not
        if (isIndexed) {
            if (songOrder.reverse) {
                results = results.reverse();
            }
            if ($uiView === "smart-query" && songOrder.orderBy !== "none") {
                resultsArray = await results.sortBy(songOrder.orderBy);
            } else {
                resultsArray = await results.toArray();
                // console.log("results", resultsArray);
            }
        } else {
            resultsArray = results;
        }

        // Filter by tags
        if ($isTagCloudOpen && $selectedTags.size) {
            resultsArray = resultsArray.filter((song) => {
                let matches = [];
                if ($selectedTags.size === 0) {
                    return true;
                } else if (song.tags?.length === 0) {
                    return false;
                }
                $selectedTags.forEach((t) => {
                    song.tags?.forEach((tag) => {
                        if (tag === t) {
                            matches.push(t);
                        }
                    });
                });
                // console.log("matches", matches);

                return $isTagOrCondition
                    ? matches.length > 0 &&
                          matches.every((m) => $selectedTags.has(m))
                    : matches.length === $selectedTags?.size &&
                          matches.every((m) => $selectedTags.has(m));
            });
        }

        // Do sorting for non-indexed results
        if (!isIndexed) {
            resultsArray = resultsArray.sort((a, b) => {
                let result = 0;
                switch (songOrder.orderBy) {
                    case "title":
                    case "album":
                    case "track":
                    case "year":
                    case "duration":
                    case "genre":
                        result = String(a[songOrder.orderBy]).localeCompare(
                            String(b[songOrder.orderBy]),
                        );
                        break;
                    case "artist":
                        // TODO this one needs to match the multiple indexes sorting from Dexie
                        // i.e Artist -> Album -> Track N.
                        result = a.artist.localeCompare(b.artist);
                        break;
                }
                if (songOrder.reverse) {
                    result *= -1;
                }
                return result;
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
        if ($uiView === "library" && get(isInit)) {
            console.log("init");
            // TESTING ONLY: Uncomment when needed
            // window["openedUrls"] = "file:///Users/slav/Downloads/Intrusive Thoughts 14-12-2023.mp3";

            if (window["openedUrls"]?.length) {
                await audioPlayer.handleOpenedUrls(window["openedUrls"]);
            }

            isInit.set(false);
        }

        if ($queueMirrorsSearch) {
            setQueue(resultsArray, false);
            if ($query.length === 0) {
                $queueMirrorsSearch = false;
            }
        }

        isLoading = false;
        return resultsArray;
    });
</script>

<div class="container" class:has-lyrics={$isLyricsOpen}>
    <CanvasLibrary
        columnOrder={libraryColumns}
        bind:query={$query}
        bind:songOrder
        allSongs={songs}
        dim={$isLyricsHovered}
        {isLoading}
    />
</div>

<style lang="scss">
    .container {
        position: relative;
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 1fr;
        box-sizing: content-box;
        overflow: hidden;

        &:has(.library-container.dragover) {
            border-color: var(--accent-secondary);
        }
    }
</style>
