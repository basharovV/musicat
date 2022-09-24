import { type, type OsType } from "@tauri-apps/api/os";
import SmartQuery from "../lib/smart-query/Query";
import Query from "./SmartQueries";
import { writable, type Writable } from "svelte/store";
import type { ArtistContentItem, ArtworkSrc, ImportStatus, SidebarItem, Song, UserSettings } from "src/App";

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
export const queriedSongs: Writable<Song[]> = writable([]);
export const songsJustAdded: Writable<Song[]> = writable([]);
export const songJustAdded = writable(false);
export const rightClickedTrack: Writable<Song> = writable(null);
export const rightClickedTracks: Writable<Song[]> = writable(null);
export const playerTime = writable(0);
export const seekTime = writable(0);
export const volume: Writable<number> = writable(localStorage.getItem('volume') ? parseFloat(localStorage.getItem('volume')) : 0.6);

export const isInfoPopupOpen = writable(false);
export const isTrackInfoPopupOpen = writable(false);
export const uiView: Writable<SidebarItem> = writable('library');

// File drop
export const droppedFiles: Writable<string[]> = writable([]);
export const hoveredFiles: Writable<string[]> = writable([]);
export const isDraggingExternalFiles = writable(false);
export const draggedScrapbookItems: Writable<ArtistContentItem[]> = writable([]);
export const emptyDropEvent: Writable<{ x: number, y: number }|null> = writable(null);
export const fileDropHandler: Writable<string> = writable(null);

// Artist's toolkit details
export const songDetailsUpdater = writable(0);

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
const defaultSettings: UserSettings = {
    albumArtworkFilenames: ["cover.jpg", "artwork.jpg", "folder.jpg"],
    miniPlayerLocation: 'bottom-left'
}
export const userSettings: Writable<UserSettings> = writable(JSON.parse(localStorage.getItem('settings')) || defaultSettings);
// Auto-persist settings
userSettings.subscribe(val => localStorage.setItem("settings", JSON.stringify(val)));

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
