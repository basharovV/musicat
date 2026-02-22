// db.ts
import { appConfigDir, BaseDirectory } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import Dexie, { type Table } from "dexie";
import { exportDB, importInto } from "dexie-export-import";
import type {
    Album,
    ArtistProject,
    ContentItem,
    Playlist,
    Song,
    SongProject,
} from "src/App";
import type { SavedSmartQuery } from "src/lib/smart-query/QueryPart";
import { get } from "svelte/store";
import { userSettings } from "./store";
import { invoke } from "@tauri-apps/api/core";

export class MySubClassedDexie extends Dexie {
    // 'songs' is added by dexie when declaring the stores()
    // We just tell the typing system this is the case
    songs!: Table<Song>;
    albums!: Table<Album>;
    smartQueries!: Table<SavedSmartQuery>;
    songProjects!: Table<SongProject>;
    artistProjects!: Table<ArtistProject>;
    scrapbook!: Table<ContentItem>;
    playlists!: Table<Playlist>;
    internalPlaylists!: Table<Playlist>;
    constructor() {
        super("musicatdb");
        this.version(14).stores({
            songs: "id, title, artist, composer, album, genre, year, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [artist+album+trackNumber], [album+trackNumber], [artist+album]", // Primary key and indexed props
            albums: "id, title, artist, year",
            smartQueries: "++id, name",
            artistProjects: "name",
            songProjects: "++id, title, artist, album",
            scrapbook: "id, name",
            playlists: "++id, title",
        });
        this.version(19)
            .stores({
                songs: "id, title, artist, composer, album, genre, year, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [artist+album+trackNumber], [album+trackNumber], [artist+album], tags", // Primary key and indexed props
                albums: "id, title, displayTitle, artist, year",
                smartQueries: "++id, name",
                artistProjects: "++id, name",
                songProjects: "++id, title, artist, album",
                scrapbook: "++id, name",
                playlists: "++id, title",
            })
            .upgrade(async (trans) => {
                await trans
                    .table("albums")
                    .toCollection()
                    .modify((album: Album) => {
                        album.displayTitle = album.title;
                        album.title = album.title.toLowerCase();
                    });
            });

        this.version(20).stores({
            songs: "id, title, artist, composer, album, genre, year, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [artist+album+trackNumber], [album+trackNumber], [artist+album], tags", // Primary key and indexed props
            albums: "id, title, displayTitle, artist, year",
            smartQueries: "++id, name",
            artistProjects: "++id, name",
            songProjects: "++id, title, artist, album",
            scrapbook: "++id, name",
            playlists: "++id, title",
            internalPlaylists: "id, title",
        });

        this.version(21).stores({
            songs: "id, title, artist, composer, album, albumArtist, genre, year, compilation, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [albumArtist+album+trackNumber], [album+trackNumber], [albumArtist+album], tags", // Primary key and indexed props
            albums: "id, title, displayTitle, artist, year, compilation",
            smartQueries: "++id, name",
            artistProjects: "++id, name",
            songProjects: "++id, title, artist, album",
            scrapbook: "++id, name",
            playlists: "++id, title",
            internalPlaylists: "id, title",
        });
    }
}

export const db = new MySubClassedDexie();

export async function exportDatabase() {
    const blob = await exportDB(db);
    // Get config dir
    const configDir = await appConfigDir();
    const bytes = await blob.arrayBuffer();
    const selected = await save({
        defaultPath:
            configDir +
            `/${process.env.NODE_ENV === "development" ? "musicat-dev.db" : "musicat.db"}`,
        filters: [
            {
                extensions: ["db"],
                name: "Database",
            },
        ],
    });
    // WRite to file
    await writeFile(selected, new Uint8Array(bytes), {
        baseDir: BaseDirectory.AppConfig,
    });
}

export async function importDatabase() {
    const selected = await open({
        directory: false,
        filters: [
            {
                extensions: ["db"],
                name: "Database",
            },
        ],
        multiple: false,
        defaultPath: await appConfigDir(),
    });
    if (!selected) {
        return;
    }
    const dbToImport = await readFile(selected);
    const blob = new Blob([dbToImport]);
    await db.delete({ disableAutoOpen: false });
    await importInto(db, blob);
}

export async function deleteDatabase() {
    await db.songs.clear();
    await db.albums.clear();
    await db.smartQueries.clear();
    await db.internalPlaylists.clear();
    await db.songProjects.clear();
    await db.artistProjects.clear();
    await db.scrapbook.clear();
    await db.playlists.clear();
    await db.delete();
}

// Helper functions for querying DB (using dexie or beets)

export async function getAlbum(albumId: string): Promise<Album> {
    try {
        if (get(userSettings).beetsDbLocation) {
            const albums = await invoke<Album[]>("get_albums_by_id", {
                albumIds: [albumId],
            });
            return albums[0];
        } else {
            return db.albums.get(albumId);
        }
    } catch (err) {
        console.error(err);
        throw new Error("Error getting album: " + err);
    }
}

// Get album tracks
export async function getAlbumTracks(album: Album): Promise<Song[]> {
    try {
        if (get(userSettings).beetsDbLocation) {
            const tracks = await invoke<Song[]>("get_beets_album_tracks", {
                albumId: album.id,
            });
            return tracks;
        } else {
            return (await db.songs.bulkGet(album.tracksIds))
                .filter((song) => song)
                .sort((a, b) => a.trackNumber - b.trackNumber);
        }
    } catch (err) {
        console.error(err);
        throw new Error("Error getting album tracks: " + err);
    }
}
