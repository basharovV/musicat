<script lang="ts">
    import { liveQuery } from "dexie";
    import type { Song } from "src/App";
    import { get } from "svelte/store";
    import BuiltInQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import {
        isInit,
        isLyricsHovered,
        isLyricsOpen,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isTagOrCondition,
        lastPlayedInfo,
        playlist,
        playlistDuration,
        queriedSongs,
        query,
        selectedPlaylistId,
        selectedSmartQuery,
        selectedTags,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        uiView
    } from "../../data/store";
    import CanvasLibrary from "../library/CanvasLibrary.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import SmartQuery from "../smart-query/Query";

    let isLoading = true;

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;

        if ($selectedPlaylistId !== null) {
            const playlist = await db.playlists.get($selectedPlaylistId);
            console.log("playlist to get", playlist);
            results = await db.songs.bulkGet(playlist.tracks);
            // Add display ID
            results = results.map((s, idx) => ({
                ...s,
                viewModel: {
                    viewId: idx.toString()
                }
            }));
            isIndexed = false;
            // Filter within playlist
            if ($query.query.length) {
                results = results.filter(
                    (song) =>
                        song.title
                            .toLowerCase()
                            .includes($query.query.toLowerCase()) ||
                        song.artist
                            .toLowerCase()
                            .includes($query.query.toLowerCase()) ||
                        song.album
                            .toLowerCase()
                            .includes($query.query.toLowerCase())
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
                    const queryName = Number($selectedSmartQuery.substring(5));
                    const savedQuery = await db.smartQueries.get(queryName);
                    const query = new SmartQuery(savedQuery);
                    results = await query.run();
                    console.log("results query: ", results);
                    isIndexed = false;
                } else {
                    // Run the query from built-in functions
                    results = await BuiltInQueries[$selectedSmartQuery].query();
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
                            .includes($query.query.toLowerCase()) ||
                        song.artist
                            .toLowerCase()
                            .includes($query.query.toLowerCase()) ||
                        song.album
                            .toLowerCase()
                            .includes($query.query.toLowerCase()) ||
                        song.tags
                            ?.map((t) => t.toLowerCase())
                            .join(" ")
                            .includes($query.query.toLowerCase())
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
            if ($uiView === "smart-query" && $query.orderBy !== "none") {
                resultsArray = await results.sortBy($query.orderBy);
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

        $playlistDuration = resultsArray.reduce((total, song) => {
            return total + song.fileInfo.duration;
        }, 0);

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
        if ($uiView === "library" && get(isInit)) {
            console.log("init");
            const lastPlayed = get(lastPlayedInfo);
            // TESTING ONLY: Uncomment when needed
            // window["openedUrls"] = "file:///Users/slav/Downloads/Intrusive Thoughts 14-12-2023.mp3";

            if (window["openedUrls"]?.length) {
                await audioPlayer.handleOpenedUrls(window["openedUrls"]);
            } else if (lastPlayed.songId) {
                audioPlayer.shouldRestoreLastPlayed = lastPlayed;
                // audioPlayer.currentSong = await db.songs.get(lastPlayed.songId);
                playlist.set(resultsArray);
            }
            isInit.set(false);
        }
        isLoading = false;
        return resultsArray;
    });
</script>

<div class="container" class:has-lyrics={$isLyricsOpen}>
    <CanvasLibrary allSongs={songs} {isLoading} dim={$isLyricsHovered} />
</div>

<style lang="scss">
    .container {
        position: relative;
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: 1fr;
        margin: 3.5px 5px 0 0;
        border-radius: 5px;
        box-sizing: content-box;
        overflow: hidden;
        border-top: 0.7px solid
            color-mix(in srgb, var(--inverse) 40%, transparent);
        border-bottom: 0.7px solid
            color-mix(in srgb, var(--inverse) 30%, transparent);
    }
</style>
