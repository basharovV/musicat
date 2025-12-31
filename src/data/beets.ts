import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";
import type { Album, BeetsQuery, Song } from "../App";

interface BeetsStore {
    songs: Song[];
    loading: boolean;
    error: string | null;
}
// src/lib/stores/beetsStore.ts

export function createBeetsSearch() {
    const { subscribe, set, update } = writable({
        songs: [] as Song[],
        loading: false,
        query: { query: "", sortBy: "artist", descending: false },
    });

    // We pull the core logic out so it can be called by updateSearch
    async function executeSearch(
        q: BeetsQuery,
        albumId?: string,
    ): Promise<Song[]> {
        update((s) => ({ ...s, loading: true }));

        try {
            const results = albumId
                ? await invoke<Song[]>("get_beets_album_tracks", {
                      albumId: albumId,
                  })
                : await invoke<Song[]>("search_beets", {
                      query: q.query,
                      sortBy: q.sortBy,
                      descending: q.descending,
                  });

            update((s) => ({ ...s, songs: results, loading: false, query: q }));
            return results; // Return the results for the caller
        } catch (e) {
            update((s) => ({ ...s, loading: false }));
            throw e; // Pass the error to the caller if needed
        }
    }

    return {
        subscribe,
        // Now an async function that returns Song[]
        updateSearch: async (
            changes: Partial<BeetsQuery>,
            albumId?: string,
        ): Promise<Song[]> => {
            let currentQuery: BeetsQuery;

            // 1. Update the state and capture the new query object
            update((s) => {
                currentQuery = { ...s.query, ...changes };
                return { ...s, query: currentQuery };
            });

            // 2. Execute the search and return the array
            return await executeSearch(currentQuery!, albumId);
        },
    };
}
function createBeetsAlbumSearch() {
    const { subscribe, set, update } = writable({
        albums: [] as Album[],
        loading: false,
        query: { query: "", sortBy: "title", descending: false },
    });

    // We pull the core logic out so it can be called by updateSearch
    async function executeSearch(q: BeetsQuery): Promise<Album[]> {
        update((s) => ({ ...s, loading: true }));

        try {
            const results = await invoke<Album[]>("search_beets_albums", {
                query: q.query,
                sortBy: q.sortBy,
                descending: q.descending,
            });
            console.log("Albums", results);

            update((s) => ({
                ...s,
                albums: results,
                loading: false,
                query: q,
            }));
            return results; // Return the results for the caller
        } catch (e) {
            update((s) => ({ ...s, loading: false }));
            throw e; // Pass the error to the caller if needed
        }
    }

    return {
        subscribe,
        updateSearch: async (
            changes: Partial<BeetsQuery>,
        ): Promise<Album[]> => {
            let currentQuery: BeetsQuery;

            // 1. Update the state and capture the new query object
            update((s) => {
                currentQuery = { ...s.query, ...changes };
                return { ...s, query: currentQuery };
            });

            // 2. Execute the search and return the array
            return await executeSearch(currentQuery!);
        },
    };
}

// Tracks
export const beetsSearch = createBeetsSearch();
export const beetsSongsOnly = derived(beetsSearch, ($state) => {
    return $state.songs;
});
export const beetsLoading = derived(beetsSearch, ($state) => $state.loading);

// Albums
export const beetsAlbumSearch = createBeetsAlbumSearch();
export const beetsAlbumsOnly = derived(beetsAlbumSearch, ($state) => {
    return $state.albums;
});
export const beetsAlbumLoading = derived(
    beetsAlbumSearch,
    ($state) => $state.loading,
);

// Album tracks
