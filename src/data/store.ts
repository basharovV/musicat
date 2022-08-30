import { type, type OsType } from "@tauri-apps/api/os";
import { writable, type Writable } from "svelte/store";

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
export const uiView: Writable<UIView> = writable("library");
export const os: Writable<OsType> = writable("Darwin");
export const importStatus: Writable<ImportStatus> = writable({
    totalTracks: 0,
    importedTracks: 0,
    isImporting: false,
    currentFolder: ''
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
