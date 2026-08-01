import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import { queryAllTracks } from "./query";
import { LL } from "../i18n/i18n-svelte";

async function favourites() {
    return queryAllTracks({
        filter: { kind: "cmp", field: "isFavourite", op: "eq", value: true },
    });
}

async function recentlyAdded() {
    return queryAllTracks({ sort: { field: "dateAdded", descending: true } });
}

async function withStems() {
    return queryAllTracks({ filter: { kind: "notEmpty", field: "stems" } });
}

export async function findQuery(queryId: string) {
    if (queryId === undefined) return null;
    let found = BUILT_IN_QUERIES[queryId];
    if (!found) {
        let id = Number(queryId?.substring(5));
        if (id !== undefined && id !== null && !isNaN(id)) {
            found = (await invoke("get_smart_query", { id })) ?? null;
        }
    }

    // If it's a built-in query, resolve the name function to get the translated text
    if (found && typeof found.name === "function") {
        found = { ...found, name: found.name() };
    }

    return found;
}

const BUILT_IN_QUERIES = {
    favourites: {
        name: () => get(LL).smartPlaylists.builtIn.favourites(),
        value: "favourites",
        run: favourites,
    },
    recentlyAdded: {
        name: () => get(LL).smartPlaylists.builtIn.recentlyAdded(),
        value: "recentlyAdded",
        run: recentlyAdded,
    },
    withStems: {
        name: () => get(LL).smartPlaylists.builtIn.withStems(),
        value: "withStems",
        run: withStems,
    },
};

export default BUILT_IN_QUERIES;
