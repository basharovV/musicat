import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";
import type { BeetsQuery, Song } from "../App";

interface BeetsStore {
    songs: Song[];
    loading: boolean;
    error: string | null;
}
// src/lib/stores/beetsStore.ts

function createBeetsSearch() {
    const { subscribe, set, update } = writable({
        songs: [] as Song[],
        loading: false,
        query: { query: "", sortBy: "artist", descending: false },
    });

    // We pull the core logic out so it can be called by updateSearch
    async function executeSearch(q: BeetsQuery): Promise<Song[]> {
        update((s) => ({ ...s, loading: true }));

        try {
            const results = await invoke<Song[]>("search_beets", {
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
        updateSearch: async (changes: Partial<BeetsQuery>): Promise<Song[]> => {
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

export const beetsSearch = createBeetsSearch();
export const beetsSongsOnly = derived(beetsSearch, ($state) => {
    return $state.songs;
});
export const beetsLoading = derived(beetsSearch, ($state) => $state.loading);
