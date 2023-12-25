import { type, type OsType } from "@tauri-apps/api/os";
import SmartQuery from "../lib/smart-query/Query";
import Query from "./SmartQueries";
import { writable, type Writable } from "svelte/store";
import type {
    Album,
    ArtistContentItem,
    ArtworkSrc,
    ImportStatus,
    SidebarItem,
    Song,
    UserSettings
} from "src/App";

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
export const nextUpSong: Writable<Song> = writable(null);
export const queriedSongs: Writable<Song[]> = writable([]);
export const playlist: Writable<Song[]> = writable([]);
export const shuffledPlaylist: Writable<Song[]> = writable([]);
export const albumPlaylist: Writable<Song[]> = writable([]);
export const playlistIsAlbum = writable(false);
export const playlistIsCountry = writable(null); // ISO Country code
export const isShuffleEnabled = writable(false);
export const songsJustAdded: Writable<Song[]> = writable([]);
export const songJustAdded = writable(false);
export const rightClickedAlbum: Writable<Album> = writable(null);
export const rightClickedTrack: Writable<Song> = writable(null);
export const rightClickedTracks: Writable<Song[]> = writable(null);
export const playerTime = writable(0);
export const seekTime = writable(0);
export const volume: Writable<number> = writable(
    localStorage.getItem("volume")
        ? parseFloat(localStorage.getItem("volume"))
        : 0.6
);
export const isFullScreenVisualiser = writable(false);

export const isInfoPopupOpen = writable(false);
export const isTrackInfoPopupOpen = writable(false);
export const uiView: Writable<SidebarItem> = writable("library");

export const isWelcomeSeen: Writable<boolean> = writable(
    Boolean(localStorage.getItem("isWelcomeSeen") || false)
);
isWelcomeSeen.subscribe((val) =>
    localStorage.setItem("isWelcomeSeen", String(val))
);
// File drop
export const droppedFiles: Writable<string[]> = writable([]);
export const hoveredFiles: Writable<string[]> = writable([]);
export const isDraggingExternalFiles = writable(false);
export const draggedScrapbookItems: Writable<ArtistContentItem[]> = writable(
    []
);
export const emptyDropEvent: Writable<{ x: number; y: number } | null> =
    writable(null);
export const fileDropHandler: Writable<string> = writable(null);

// Artist's toolkit details
export const selectedArtistId: Writable<string> = writable(
    localStorage.getItem("selectedArtistId") || null
);
selectedArtistId.subscribe((val) => {
    if (val) {
        localStorage.setItem("selectedArtistId", val);
    } else {
        localStorage.removeItem("selectedArtistId");
    }
});

export const songDetailsUpdater = writable(0);
export const isScrapbookShown = writable(true);

// Library menu
export const isFindFocused = writable(false);

// Smart query
export const isSmartQueryUiOpen = writable(false);
export const isSmartQueryBuilderOpen = writable(false);
export const isSmartQuerySaveUiOpen = writable(false);
export const smartQuery: Writable<SmartQuery> = writable(new SmartQuery());
export const smartQueryInitiator = writable("sidebar");
export const selectedSmartQuery = writable(Query[0].value);
export const isSmartQueryValid = writable(false);
export const smartQueryUpdater = writable(0);
export const smartQueryResults: Writable<Song[]> = writable([]);

// Playlists
export const selectedPlaylistId: Writable<String> = writable(null);
export const draggedSongs: Writable<Song[]> = writable([]);

// Settings
export const isSettingsOpen = writable(false);
const defaultSettings: UserSettings = {
    albumArtworkFilenames: ["cover.jpg", "artwork.jpg", "folder.jpg"],
    miniPlayerLocation: "bottom-left",
    llm: 'ollama',
    openAIApiKey: null,
    aiFeaturesEnabled: false
};

export const userSettings: Writable<UserSettings> = writable(
    {...defaultSettings, ...JSON.parse(localStorage.getItem("settings"))} || defaultSettings
);
// Auto-persist settings
userSettings.subscribe((val) =>
    localStorage.setItem("settings", JSON.stringify(val))
);

export const libraryScrollPos: Writable<number> = writable(
    localStorage.getItem("libraryScrollPos")
        ? parseFloat(localStorage.getItem("libraryScrollPos"))
        : 0
);

libraryScrollPos.subscribe((scrollPos) => {
    localStorage.setItem("libraryScrollPos", String(scrollPos));
});

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
