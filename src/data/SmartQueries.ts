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

export async function favourites() {
    return db.songs.filter((s) => s.isFavourite);
}

export async function whereGenreIs(genre: string) {
    return db.songs.where("genre").equals(genre);
}

export default {
    favourites: {
        name: "favourites",
        value: "favourites",
        query: favourites
    }
};
