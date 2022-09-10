import { type, type OsType } from "@tauri-apps/api/os";
import SmartQuery from "../lib/smart-query/Query";
import Query from "./SmartQueries";
import { writable, type Writable } from "svelte/store";
import type { ArtworkSrc, ImportStatus, Song, UserSettings } from "src/App";

interface Query {
    orderBy: string;
    reverse: boolean;
    query: string;
}

type UIView = "library" | "albums";

export const query: Writable<Query> = writable({
    orderBy: "artist",
    reverse: false,
    query: ""
});

export const isPlaying = writable(false);
export const currentSong: Writable<Song> = writable(null);
export const currentSongIdx = writable(0);
export const isDraggingFiles = writable(false);
export const queriedSongs: Writable<Song[]> = writable([]);
export const songsJustAdded: Writable<Song[]> = writable([]);
export const songJustAdded = writable(false);
export const rightClickedTrack: Writable<Song> = writable(null);
export const rightClickedTracks: Writable<Song[]> = writable(null);
export const playerTime = writable(0);
export const seekTime = writable(0);
export const volume = writable(1);
export const isInfoPopupOpen = writable(false);
export const isTrackInfoPopupOpen = writable(false);

// Smart query
export const isSmartQueryUiOpen = writable(false);
export const isSmartQueryBuilderOpen = writable(false);
export const isSmartQuerySaveUiOpen = writable(false);
export const smartQuery: Writable<SmartQuery> = writable(new SmartQuery());
export const selectedSmartQuery = writable(Query[0].value);
export const isSmartQueryValid = writable(false);
export const smartQueryUpdater = writable(0);
export const smartQueryResults: Writable<Song[]> = writable([]);

// Settings
export const isSettingsOpen = writable(false);
export const userSettings: Writable<UserSettings> = writable({
    albumArtworkFilenames: ["cover.jpg", "artwork.jpg", "folder.jpg"]
});
export const uiView: Writable<UIView> = writable("library");
export const os: Writable<OsType> = writable("Darwin");

export const importStatus: Writable<ImportStatus> = writable({
    totalTracks: 0,
    importedTracks: 0,
    isImporting: false,
    currentFolder: ""
});

export const singleKeyShortcutsEnabled = writable(true);
export const currentSongArtworkSrc: Writable<ArtworkSrc> = writable(null);
export const isMiniPlayer = writable(false);
async function getOs() {
    const os = await type();
    let explorerName;
    switch (os) {
        case "Darwin":
            explorerName = "Finder";
            break;
        case "Windows_NT":
            explorerName = "Explorer";
            break;
        case "Linux":
            explorerName = "File manager";
            break;
    }
    return explorerName;
}

getOs();
