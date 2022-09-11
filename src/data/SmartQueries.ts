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
    const uniqueKeys = await db.songs.orderBy('artist').uniqueKeys();
    for (const artist of uniqueKeys) {
        // Perform a album query check
        const albums = await db.songs.orderBy('album').and(s => s.artist === artist).uniqueKeys();
        if (albums.length > 1) {
            artistsWithMultipleAlbums.push(artist);
        }
    }
    return db.songs.where('artist').anyOf(artistsWithMultipleAlbums);
}

function tracksLongerThan6Minutes() {
    return db.songs.where('duration').above("06:00");
}

async function jazzFromThe50s() {
    const keys = await db.songs.orderBy('genre').uniqueKeys();
    console.log('genres', keys);
    return db.songs.where('genre').equals(['Jazz']).and(song => song.year < 1960 && song.year > 1949);
}

export default [
    {
        name: "tracks with no metadata",
        value: 'no-metadata',
        query: tracksWithEmptyMetadata
    },
    {
        name: "artists with multiple albums",
        value: 'multiple-albums',
        query: artistsWithMultipleAlbums
    },
    {
        name: "tracks longer than 6 minutes",
        value: 'longer-6min',
        query: tracksLongerThan6Minutes
    },
    {
        name: "jazz from the 50s",
        value: 'jazz-50s',
        query: jazzFromThe50s
    }
];
