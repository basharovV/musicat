import { path } from "@tauri-apps/api";
import { appConfigDir, audioDir, downloadDir } from "@tauri-apps/api/path";
import * as fs from "@tauri-apps/plugin-fs";
import { type, type OsType } from "@tauri-apps/plugin-os";
import type {
    ActionEvent,
    AddOriginCountryStatus,
    Album,
    ArrowFocus,
    ArtistContentItem,
    ArtistProject,
    ArtworkSrc,
    BottomBarNotification,
    Compression,
    CurrentSongLyrics,
    IACollection,
    IAFile,
    IAItem,
    ImportStatus,
    LastPlayedInfo,
    PlaylistFile,
    PlaylistType,
    Popup,
    PopupType,
    QueueMode,
    UiView,
    Song,
    StreamInfo,
    UIPreferences,
    UserSettings,
    WaveformPlayerState
} from "src/App";
import { derived, get, writable, type Writable } from "svelte/store";
import { locale } from "../i18n/i18n-svelte";
import { i18nString } from "../i18n/i18n-util";
import SmartQuery from "../lib/smart-query/Query";
import Query from "./SmartQueries";
import { scanPlaylists } from "./M3UUtils";
import { liveQuery } from "dexie";
import { db } from "./db";

export const L = derived(locale, (l) => {
    return i18nString(l);
});

interface Query {
    orderBy: string;
    libraryOrderBy: string;
    reverse: boolean;
    query: string;
}

type UIView = "library" | "albums";

export const isInit = writable(true);
export const forceRefreshLibrary = writable(false);
export const query: Writable<Query> = writable({
    orderBy: "artist",
    libraryOrderBy: "artist",
    reverse: false,
    query: ""
});

export const isPlaying = writable(false);
export const currentSong: Writable<Song> = writable(null);
export const currentSongIdx = writable(0);

const defaultLastPlayedInfo: LastPlayedInfo = {
    songId: null,
    position: 0
};

export const lastPlayedInfo: Writable<LastPlayedInfo> = writable(
    {
        ...defaultLastPlayedInfo,
        ...JSON.parse(localStorage.getItem("lastPlayedInfo"))
    } || defaultLastPlayedInfo
);
// Auto-persist settings
lastPlayedInfo.subscribe((val) =>
    localStorage.setItem("lastPlayedInfo", JSON.stringify(val))
);

export const nextUpSong: Writable<Song> = writable(null);
export const queriedSongs: Writable<Song[]> = writable([]);

export const allSongs: Writable<Song[]> = writable([]);
export const playlist: Writable<Song[]> = writable([]);
export const shuffledPlaylist: Writable<Song[]> = writable([]);
export const albumPlaylist: Writable<Song[]> = writable([]);
export const playlistIsAlbum = writable(false);
export const playlistCountry = writable(null); // ISO Country code
export const playlistType: Writable<PlaylistType> = writable("library");
export const playlistDuration: Writable<number> = writable(0);
export const isShuffleEnabled = writable(false);
export const songsJustAdded: Writable<Song[]> = writable([]);
export const songJustAdded = writable(false);
export const shouldShowToast = writable(true);
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

// Playlists (populated from folder)
export const userPlaylists: Writable<PlaylistFile[]> = writable([]);
export const toDeletePlaylist = liveQuery(async () => {
    try {
        const toDeletePlaylist = await db.internalPlaylists.get("todelete");
        return toDeletePlaylist;
    } catch (e) {
        console.error("Error fetching todelete playlist", e);
    }
    return null;
})

export const popupOpen: Writable<PopupType> = writable(null);
export const uiView: Writable<UiView> = writable("library");
export const arrowFocus: Writable<ArrowFocus> = writable("library");
export const draggedColumnIdx: Writable<number | null> = writable(null);

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
export const songbookSelectedArtist: Writable<ArtistProject> = writable(
    localStorage.getItem("selectedArtist")
        ? JSON.parse(localStorage.getItem("selectedArtist"))
        : null || null
);
songbookSelectedArtist.subscribe((val) => {
    if (val !== null && val !== undefined) {
        localStorage.setItem("selectedArtist", JSON.stringify(val));
    } else {
        localStorage.removeItem("selectedArtist");
    }
});
export const songbookArtists = writable<ArtistProject[]>([]);
export const currentSongProjects: Writable<string[]> = writable([]);
export const songDetailsUpdater = writable(0);
export const isScrapbookShown = writable(true);
export const songbookFileSavedTime = writable(0); // timestamp of last save
export const isFullScreenLyrics = writable(false);

// Library menu
export const isFindFocused = writable(false);
export const shouldFocusFind: Writable<ActionEvent | null> = writable(null);

// Smart query
export const isSmartQueryUiOpen = writable(false);
export const isSmartQueryBuilderOpen = writable(false);
export const isSmartQuerySaveUiOpen = writable(false);
export const smartQuery: Writable<SmartQuery> = writable(new SmartQuery());
export const smartQueryInitiator = writable("sidebar");
export const selectedSmartQuery = writable(null);
export const isSmartQueryValid = writable(false);
export const smartQueryUpdater = writable(0);
export const smartQueryResults: Writable<Song[]> = writable([]);

// Tag cloud
export const isTagCloudOpen = writable(false);
export const selectedTags = writable(new Set());
export const isTagOrCondition = writable(false);

// Playlists
export const selectedPlaylistFile: Writable<PlaylistFile> = writable(null);
export const draggedSongs: Writable<Song[]> = writable([]);
export const draggedAlbum: Writable<Album> = writable(null);
export const isDraggingFromQueue = writable(false);
export const dragGhostReset = writable(false);

// Settings
const defaultSettings: UserSettings = {
    foldersToWatch: [],
    albumArtworkFilenames: ["cover.jpg", "artwork.jpg", "folder.jpg"],
    miniPlayerLocation: "bottom-left",
    isArtistsToolkitEnabled: false,
    scrapbookLocation: null,
    songbookLocation: null,
    downloadLocation: null,
    playlistsLocation: null,
    theme: "dark",
    outputDevice: null, // default system device,
    followSystemOutput: true
};

/**
 * Retrieve settings from app config dir
 */
async function getSettings() {
    // use fs to read stetings file
    try {
        const configDir = await appConfigDir();
        const settingsPath = await path.join(configDir, "settings.json");
        // Read contents
        const contents = await fs.readTextFile(settingsPath);
        return JSON.parse(contents);
    } catch (e) {
        console.error("Error reading settings file", e);
        return get(userSettings);
    }
}

async function setSettings(settings: UserSettings) {
    // use fs to write settings file
    const configDir = await appConfigDir();
    const settingsPath = await path.join(configDir, "settings.json");
    await fs.writeTextFile(settingsPath, JSON.stringify(settings));
}

export const userSettings: Writable<UserSettings> = writable(defaultSettings);

const defaultUIPreferences: UIPreferences = {
    albumsViewShowSingles: false,
    albumsViewShowInfo: true,
    albumsViewSortBy: "title",
    albumsViewGridSize: 197
};

// UI preferences
export const uiPreferences: Writable<UIPreferences> = writable({
    ...defaultUIPreferences,
    ...JSON.parse(localStorage.getItem("uiPreferences") || "{}")
});

uiPreferences.subscribe((val) => {
    console.log("uiPreferences", val);
    localStorage.setItem("uiPreferences", JSON.stringify(val));
});

export const foldersToWatch = derived(
    userSettings,
    (val) => val.foldersToWatch
);

export const playlistLocation = derived(
    userSettings,
    (val) => val.playlistsLocation
);

playlistLocation.subscribe((pl) => {
    // Get playlists
    if (pl) {
        scanPlaylists();
    }
});

export const libraryScrollPos: Writable<number> = writable(
    localStorage.getItem("libraryScrollPos")
        ? parseFloat(localStorage.getItem("libraryScrollPos"))
        : 0
);
export const scrollToSong: Writable<Song> = writable(null);

libraryScrollPos.subscribe((scrollPos) => {
    localStorage.setItem("libraryScrollPos", String(scrollPos));
});

export const os: Writable<OsType> = writable("macos");

export const importStatus: Writable<ImportStatus> = writable({
    totalTracks: 0,
    importedTracks: 0,
    isImporting: false,
    currentFolder: "",
    backgroundImport: false,
    currentSong: null,
    status: null,
    percent: 0
});
export const isFolderWatchUpdate = writable(false);
export const bottomBarNotification: Writable<BottomBarNotification> =
    writable(null);
export const singleKeyShortcutsEnabled = writable(true);
export const currentSongArtworkSrc: Writable<ArtworkSrc> = writable(null);
export const isMiniPlayer = writable(false);
export const compressionSelected: Writable<Compression> = writable("both");

export const addOriginCountryStatus: Writable<AddOriginCountryStatus> =
    writable(null);

export const isLyricsOpen = writable(false);
export const isLyricsHovered = writable(false);
export const currentSongLyrics: Writable<CurrentSongLyrics> = writable(null);

// Queue
export const isQueueOpen = writable(false);
export const isQueueCleared = writable(false);
export const queueMode: Writable<QueueMode> = writable("library");

// Wiki
export const isWikiOpen = writable(false);
export const wikiArtist: Writable<string> = writable(null);

// Sidebar
export const isSidebarOpen = writable(true);
export const sidebarManuallyOpened = writable(false);
export const sidebarTogglePos = writable({ x: 0, y: 0 });
export const isCmdOrCtrlPressed = writable(false);

export const isWaveformOpen = writable(false);
export const waveformPeaks: Writable<WaveformPlayerState> = writable({
    songId: null,
    data: [],
    markers: [],
    loopEnabled: false,
    loopEndPos: 0,
    loopStartPos: 0
});

async function init() {
    // Set OS
    const osType = await type();
    os.set(osType);

    let fileSettings = await getSettings();

    // Set default download location
    if (typeof fileSettings.downloadLocation != 'string') {
        fileSettings.downloadLocation = await downloadDir();
    }

    // Same for playlists location
    if (typeof fileSettings.playlistsLocation != 'string') {
        fileSettings.playlistsLocation =
            await path.join(await audioDir(), "Musicat Playlists");
    }

    // Get user settings
    userSettings.set({
        ...defaultSettings,
        ...fileSettings
    });

    // Auto-persist settings
    userSettings.subscribe(async (val) => {
        console.log("[store] userSettings", val);
        // Write settings to file
        await setSettings(val);
    });
}

export const streamInfo: Writable<StreamInfo> = writable({
    bytesReceived: 0,
    receiveRate: 0,
    bufferedSamples: 0,
    playedSamples: 0,
    timestamp: 0,
    sampleIdx: 0
});

let defaultColumnOrder = [
    "title",
    "artist",
    "composer",
    "album",
    "trackNumber",
    "year",
    "genre",
    "originCountry",
    "duration",
    "tags"
];

export const columnOrder = writable(
    JSON.parse(localStorage.getItem("columnOrder")) || defaultColumnOrder
);

// Auto-persist column order
columnOrder.subscribe((val) =>
    localStorage.setItem("columnOrder", JSON.stringify(val))
);

// Internet Archive
export const iaCollections: Writable<IACollection[]> = writable([]);
export const iaSelectedCollection: Writable<IACollection> = writable(null);
export const iaSelectedCollectionItems: Writable<IAItem[]> = writable([]);
export const iaSelectedItem: Writable<IAItem> = writable(null);

export const currentIAFile: Writable<IAFile> = writable(null);
export const webPlayerBufferedRanges: Writable<TimeRanges> = writable(null);
export const webPlayerVolume: Writable<number> = writable(0.6);
export const webPlayerIsLoading = writable(false);
export const fileToDownload: Writable<IAFile> = writable(null);
init();
