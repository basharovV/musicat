import type { Collection, IndexableType } from "dexie";
import type { Song } from "src/App";
import { get } from "svelte/store";
import { db } from "./db";
import { query } from "./store";

function allTracks(): Collection<Song, IndexableType> {
    // Not including title, because title is always populated with filename,
    // even if metadata is missing
    return db.songs.orderBy(get(query).orderBy);
}
function tracksWithEmptyMetadata(): Collection<Song, IndexableType> {
    // Not including title, because title is always populated with filename,
    // even if metadata is missing
    return db.songs.where({ artist: "", album: "" });
}

async function artistsWithMultipleAlbums() {
    const artistsWithMultipleAlbums = [];
    const uniqueKeys = await db.songs.orderBy("artist").uniqueKeys();
    for (const artist of uniqueKeys) {
        // Perform a album query check
        const albums = await db.songs
            .orderBy("album")
            .and((s) => s.artist === artist)
            .uniqueKeys();
        if (albums.length > 1) {
            artistsWithMultipleAlbums.push(artist);
        }
    }
    return db.songs.where("artist").anyOf(artistsWithMultipleAlbums);
}

function tracksLongerThan6Minutes() {
    return db.songs.where("duration").above("06:00");
}

async function jazzFromThe50s() {
    const keys = await db.songs.orderBy("genre").uniqueKeys();
    console.log("genres", keys);
    return db.songs
        .where("genre")
        .equals(["Jazz"])
        .and((song) => song.year < 1960 && song.year > 1949);
}

async function favourites() {
    return db.songs.filter((s) => s.isFavourite);
}

async function recentlyAdded() {
    return db.songs.orderBy("dateAdded").reverse();
}

async function whereGenreIs(genre: string) {
    return db.songs.where("genre").equals(genre);
}

export async function findQuery(queryId: string) {
    console.log("find", queryId);
    if (queryId === undefined) return null;
    let found = BUILT_IN_QUERIES[queryId];
    if (!found) {
        let id = Number(queryId?.substring(5));
        if (id !== undefined && id !== null && !isNaN(id)) {
            found = (await db.smartQueries.get(id)) ?? null;
        }
    }
    console.log("found", found);
    return found;
}

const BUILT_IN_QUERIES = {
    favourites: {
        name: "Favourites",
        value: "favourites",
        run: favourites,
    },
    recentlyAdded: {
        name: "Recently Added",
        value: "recentlyAdded",
        run: recentlyAdded,
    },
};

export default BUILT_IN_QUERIES;
