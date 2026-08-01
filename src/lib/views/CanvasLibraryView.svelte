<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { PlaylistFile, Song, SongOrder } from "src/App";
    import { onMount } from "svelte";
    import { get, writable, type Writable } from "svelte/store";
    import { songMatchesQuery } from "../../data/LibraryUtils";
    import { parsePlaylist } from "../../data/M3UUtils";
    import { queryAllTracks } from "../../data/query";
    import BuiltInQueries from "../../data/SmartQueries";
    import {
        beetsLoading,
        beetsSearch,
        beetsSongsOnly,
    } from "../../data/beets";
    import {
        expandedSongWithStems,
        importStatus,
        isInit,
        isLyricsHovered,
        isLyricsOpen,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isTagOrCondition,
        libraryColumns,
        libraryUpdater,
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
        userSettings,
    } from "../../data/store";
    import { setQueue } from "../../data/storeHelper";
    import CanvasLibrary from "../library/CanvasLibrary.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import SmartQuery from "../smart-query/Query";
    import ImportPlaceholder from "../library/ImportPlaceholder.svelte";
    export let songOrder: SongOrder;

    let isLoading = true;
    const librarySongs: Writable<Song[]> = writable([]);

    $: songs = $userSettings.beetsDbLocation ? beetsSongsOnly : librarySongs;

    /**
     * Orders `results` in place, including the compound tie-breaking Dexie's
     * `[artist+year+album+trackNumber]`/`[album+trackNumber]` indexes used to
     * provide -- every path (to-delete, m3u, smart query, default browse)
     * returns a plain array now, so they all share this one sort instead of
     * each needing their own (the old "artist" case here was a straight
     * localeCompare with a "needs to match the multiple indexes sorting from
     * Dexie" TODO; this actually does that now, for every path, not just the
     * ones that used to go through a Dexie compound index).
     */
    function sortSongs(results: Song[], order: SongOrder): Song[] {
        const dir = order.reverse ? -1 : 1;
        return results.sort((a, b) => {
            let result = 0;
            switch (order.orderBy) {
                case "artist":
                    result =
                        a.artist.localeCompare(b.artist) ||
                        (a.year ?? 0) - (b.year ?? 0) ||
                        a.album.localeCompare(b.album) ||
                        (a.trackNumber ?? 0) - (b.trackNumber ?? 0);
                    break;
                case "album":
                    result =
                        a.album.localeCompare(b.album) ||
                        (a.trackNumber ?? 0) - (b.trackNumber ?? 0);
                    break;
                case "trackNumber":
                    result = (a.trackNumber ?? 0) - (b.trackNumber ?? 0);
                    break;
                case "title":
                case "year":
                case "duration":
                case "genre":
                    result = String(a[order.orderBy]).localeCompare(
                        String(b[order.orderBy]),
                    );
                    break;
                default:
                    result = 0;
            }
            return result * dir;
        });
    }

    async function fetchLibrarySongs(
        uiView: string,
        toDeletePlaylist: { tracks: string[] } | null,
        selectedPlaylistFile: PlaylistFile | null,
        query: string,
        isSmartQueryBuilderOpen: boolean,
        smartQuery: SmartQuery,
        smartQueryUpdater: number,
        selectedSmartQuery: string,
        isTagCloudOpen: boolean,
        selectedTags: Set<string>,
        isTagOrCondition: boolean,
        songOrder: SongOrder,
        _libraryUpdater: number,
    ) {
        isLoading = true;
        let results: Song[];
        let isSmartQueryResults = false;

        if (uiView === "to-delete") {
            if (toDeletePlaylist === null || toDeletePlaylist.tracks.length === 0) {
                results = [];
            } else {
                results = (
                    await invoke<(Song | null)[]>("get_tracks_by_ids", {
                        ids: toDeletePlaylist.tracks,
                    })
                ).filter((s): s is Song => s !== null);
            }
            results = results.map((s, idx) => ({
                ...s,
                viewModel: { index: idx, viewId: idx.toString() },
            }));
        } else if (selectedPlaylistFile !== null) {
            results = await parsePlaylist(selectedPlaylistFile);
            results = results.map((s, idx) => ({
                ...s,
                viewModel: { index: idx, viewId: idx.toString() },
            }));
            if (query.length) {
                results = results.filter((song) => songMatchesQuery(song, query));
            }
        } else if (uiView === "smart-query" || uiView === "favourites") {
            if (isSmartQueryBuilderOpen) {
                if (smartQuery.isEmpty) {
                    results = [];
                } else if (smartQueryUpdater) {
                    results = await smartQuery.run();
                } else {
                    results = [];
                }
            } else if (selectedSmartQuery.startsWith("~usq:")) {
                const loaded = await SmartQuery.loadWithUQI(
                    selectedSmartQuery as `~usq:${string}`,
                );
                results = await loaded.run();
            } else {
                results = await BuiltInQueries[selectedSmartQuery].run();
            }
            isSmartQueryResults = true;
        } else {
            results = await queryAllTracks({});
            if (query.length) {
                results = results.filter((song) => songMatchesQuery(song, query));
            }
        }

        // Filter by tags
        if (isTagCloudOpen && selectedTags.size) {
            results = results.filter((song) => {
                let matches = [];
                if (selectedTags.size === 0) {
                    return true;
                } else if (song.tags?.length === 0) {
                    return false;
                }
                selectedTags.forEach((t) => {
                    song.tags?.forEach((tag) => {
                        if (tag === t) {
                            matches.push(t);
                        }
                    });
                });

                return isTagOrCondition
                    ? matches.length > 0 && matches.every((m) => selectedTags.has(m))
                    : matches.length === selectedTags?.size &&
                          matches.every((m) => selectedTags.has(m));
            });
        }

        results = sortSongs(results, songOrder);

        if (get(expandedSongWithStems)) {
            const idx = results.findIndex(
                (s) => s.id === get(expandedSongWithStems).id,
            );
            if (idx !== -1) {
                expandedSongWithStems.update((s) => ({
                    ...s,
                    viewModel: { ...s.viewModel, index: idx },
                }));
            } else {
                expandedSongWithStems.set(null);
            }
        }

        if (isSmartQueryResults) {
            smartQueryResults.set(results);
        } else {
            queriedSongs.set(results);
        }

        if (uiView === "library" && get(isInit)) {
            if (window["openedUrls"]?.length) {
                await audioPlayer.handleOpenedUrls(window["openedUrls"]);
            }
            isInit.set(false);
        }

        if (get(queueMirrorsSearch)) {
            setQueue(results, false);
            if (query.length === 0) {
                queueMirrorsSearch.set(false);
            }
        }

        librarySongs.set(results);
        isLoading = false;
    }

    $: if (!$userSettings.beetsDbLocation) {
        fetchLibrarySongs(
            $uiView,
            $toDeletePlaylist,
            $selectedPlaylistFile,
            $query,
            $isSmartQueryBuilderOpen,
            $smartQuery,
            $smartQueryUpdater,
            $selectedSmartQuery,
            $isTagCloudOpen,
            $selectedTags,
            $isTagOrCondition,
            songOrder,
            $libraryUpdater,
        );
    }

    async function onQueryChanged(query: string, songOrder: SongOrder) {
        if ($uiView === "library") {
            queriedSongs.set(
                await beetsSearch.updateSearch({
                    query,
                    sortBy: songOrder.orderBy,
                    descending: songOrder.reverse,
                }),
            );
        } else {
            smartQueryResults.set(
                await beetsSearch.updateSearch({
                    query,
                    sortBy: songOrder.orderBy,
                    descending: songOrder.reverse,
                }),
            );
        }
    }
    $: {
        $userSettings.beetsDbLocation &&
            onQueryChanged($query || "", songOrder);
    }

    $: noSongs = !$songs || ($songs.length === 0 && $query.length === 0);
</script>

{#if isLoading}
    <span />
{:else if ($importStatus.isImporting && $importStatus.backgroundImport === false) || (noSongs && $query?.length === 0 && $uiView.match(/^(smart-query|favourites|to-delete)/) === null && $isTagCloudOpen === false)}
    <ImportPlaceholder />
{:else}
    <div class="container" class:has-lyrics={$isLyricsOpen}>
        <CanvasLibrary
            columnOrder={libraryColumns}
            bind:query={$query}
            bind:songOrder
            songsReadable={songs}
            dim={$isLyricsHovered}
            isLoading={false}
        />
    </div>
{/if}

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
