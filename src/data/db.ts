// db.ts
import Dexie, { type Table } from "dexie";
import type {
    Album,
    ArtistProject,
    ContentItem,
    Playlist,
    Song,
    SongProject
} from "src/App";
import type { SavedSmartQuery } from "src/lib/smart-query/QueryPart";

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
            playlists: "++id, title"
        });
        this.version(19)
            .stores({
                songs: "id, title, artist, composer, album, genre, year, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [artist+album+trackNumber], [album+trackNumber], [artist+album], tags", // Primary key and indexed props
                albums: "id, title, displayTitle, artist, year",
                smartQueries: "++id, name",
                artistProjects: "++id, name",
                songProjects: "++id, title, artist, album",
                scrapbook: "++id, name",
                playlists: "++id, title"
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

        this.version(20)
            .stores({
                songs: "id, title, artist, composer, album, genre, year, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [artist+album+trackNumber], [album+trackNumber], [artist+album], tags", // Primary key and indexed props
                albums: "id, title, displayTitle, artist, year",
                smartQueries: "++id, name",
                artistProjects: "++id, name",
                songProjects: "++id, title, artist, album",
                scrapbook: "++id, name",
                playlists: "++id, title",
                internalPlaylists: "id, title"
            });
        
        this.version(21)
            .stores({
                songs: "id, title, artist, composer, album, albumArtist, genre, year, compilation, path, duration, isFavourite, originCountry, dateAdded, [artist+year+album+trackNumber], [albumArtist+album+trackNumber], [album+trackNumber], [albumArtist+album], tags", // Primary key and indexed props
                albums: "id, title, displayTitle, artist, year, compilation",
                smartQueries: "++id, name",
                artistProjects: "++id, name",
                songProjects: "++id, title, artist, album",
                scrapbook: "++id, name",
                playlists: "++id, title",
                internalPlaylists: "id, title"
            });
    }
}

export const db = new MySubClassedDexie();
