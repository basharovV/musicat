import { type, type OsType } from "@tauri-apps/api/os";
import { appConfigDir, downloadDir } from "@tauri-apps/api/path";
import type {
    ActionEvent,
    AddOriginCountryStatus,
    Album,
    ArrowFocus,
    ArtistContentItem,
    ArtworkSrc,
    BottomBarNotification,
    Compression,
    CurrentSongLyrics,
    IACollection,
    IAFile,
    IAItem,
    ImportStatus,
    LastPlayedInfo,
    PlaylistType,
    QueueMode,
    SidebarItem,
    Song,
    StreamInfo,
    UserSettings,
    WaveformPlayerState
} from "src/App";
import { derived, writable, type Writable } from "svelte/store";
import { locale } from "../i18n/i18n-svelte";
import { i18nString } from "../i18n/i18n-util";
import SmartQuery from "../lib/smart-query/Query";
import Query from "./SmartQueries";
import { fs, path } from "@tauri-apps/api";

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

export const isInfoPopupOpen = writable(false);
export const isTrackInfoPopupOpen = writable(false);
export const uiView: Writable<SidebarItem> = writable("library");
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
export const shouldFocusFind: Writable<ActionEvent | null> = writable(null);

// Smart query
export const isSmartQueryUiOpen = writable(false);
export const isSmartQueryBuilderOpen = writable(false);
export const isSmartQuerySaveUiOpen = writable(false);
export const smartQuery: Writable<SmartQuery> = writable(new SmartQuery());
export const smartQueryInitiator = writable("sidebar");
export const selectedSmartQuery = writable(Query.favourites.value);
export const isSmartQueryValid = writable(false);
export const smartQueryUpdater = writable(0);
export const smartQueryResults: Writable<Song[]> = writable([]);

// Playlists
export const selectedPlaylistId: Writable<number> = writable(null);
export const draggedSongs: Writable<Song[]> = writable([]);
export const draggedAlbum: Writable<Album> = writable(null);
export const isDraggingFromQueue = writable(false);
export const dragGhostReset = writable(false);

// Settings
export const isSettingsOpen = writable(false);
const defaultSettings: UserSettings = {
    foldersToWatch: [],
    albumArtworkFilenames: ["cover.jpg", "artwork.jpg", "folder.jpg"],
    miniPlayerLocation: "bottom-left",
    llm: "ollama",
    openAIApiKey: null,
    aiFeaturesEnabled: false,
    geniusApiKey: null,
    isArtistsToolkitEnabled: false,
    downloadLocation: null,
    theme: "dark"
};

/**
 * Retrieve settings from app config dir
 */
async function getSettings() {
    // use fs to read stetings file
    const configDir = await appConfigDir();
    const settingsPath = await path.join(configDir, "settings.json");
    // Read contents
    const contents = await fs.readTextFile(settingsPath);
    return JSON.parse(contents);
}

async function setSettings(settings: UserSettings) {
    // use fs to write settings file
    const configDir = await appConfigDir();
    const settingsPath = await path.join(configDir, "settings.json");
    await fs.writeTextFile(settingsPath, JSON.stringify(settings));
}

export const userSettings: Writable<UserSettings> = writable(defaultSettings);

export const foldersToWatch = derived(
    userSettings,
    (val) => val.foldersToWatch
);

export const libraryScrollPos: Writable<number> = writable(
    localStorage.getItem("libraryScrollPos")
        ? parseFloat(localStorage.getItem("libraryScrollPos"))
        : 0
);
export const scrollToSong: Writable<Song> = writable(null);

libraryScrollPos.subscribe((scrollPos) => {
    localStorage.setItem("libraryScrollPos", String(scrollPos));
});

export const os: Writable<OsType> = writable("Darwin");

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

    // Set default download location
    const dir = await downloadDir();

    const fileSettings = await getSettings();
    if (!fileSettings.downloadLocation) {
        fileSettings.downloadLocation = dir;
    }
    // Get user settings
    userSettings.set({
        ...defaultSettings,
        ...fileSettings
    });

    // Auto-persist settings
    userSettings.subscribe(async (val) => {
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
    "duration"
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
