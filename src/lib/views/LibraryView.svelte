<script lang="ts">
    import { liveQuery } from "dexie";
    import type { Song } from "src/App";
    import BuiltInQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import {
        isInit,
        isLyricsHovered,
        isLyricsOpen,
        isSmartQueryBuilderOpen,
        lastPlayedInfo,
        playlist,
        queriedSongs,
        query,
        selectedPlaylistFile,
        selectedSmartQuery,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        uiView,
    } from "../../data/store";
    import Library from "../library/Library.svelte";
    import SmartQuery from "../smart-query/Query";
    import { get } from "svelte/store";
    import audioPlayer from "../player/AudioPlayer";
    import LyricsView from "../library/LyricsView.svelte";
    import { fade, fly } from "svelte/transition";

    let isLoading = true;

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;

        if ($selectedPlaylistFile !== null) {
            const playlist = await db.playlists.get($selectedPlaylistFile);
            console.log("playlist to get", playlist);
            results = await db.songs.bulkGet(playlist.tracks);
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
                            .includes($query.query.toLowerCase()),
                );
            }
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
                    const queryId = Number($selectedSmartQuery.substring(5));
                    const savedQuery = await db.smartQueries.get(queryId);
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
                          : $query.orderBy,
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
                            .startsWith($query.query.toLowerCase()),
                );
        } else {
            results = db.songs.orderBy(
                $query.orderBy === "artist"
                    ? "[artist+year+album+trackNumber]"
                    : $query.orderBy === "album"
                      ? "[album+trackNumber]"
                      : $query.orderBy,
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
                            b[$query.orderBy],
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
        if (get(isInit)) {
            console.log("init");
            const lastPlayed = get(lastPlayedInfo);
            if (lastPlayed.songId) {
                audioPlayer.shouldRestoreLastPlayed = lastPlayed;
                audioPlayer.currentSong = await db.songs.get(lastPlayed.songId);
                playlist.set(resultsArray);
            }
            isInit.set(false);
        }
        isLoading = false;
        return resultsArray;
    });
</script>

<div class="container" class:has-lyrics={$isLyricsOpen}>
    <Library
        allSongs={songs}
        showMyArtists={true}
        {isLoading}
        dim={$isLyricsHovered}
    />
    {#if $isLyricsOpen}
        <div class="lyrics" transition:fade={{ duration: 150 }}>
            <LyricsView />
        </div>
    {/if}
</div>

<style lang="scss">
    .container {
        position: relative;
        display: grid;
        grid-template-columns: 1fr;

        &.has-lyrics {
            /* grid-template-columns: 1fr auto; */
        }

        .lyrics {
        }
    }
</style>
