import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { audioDir } from "@tauri-apps/api/path";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import md5 from "md5";
import { get } from "svelte/store";
import type {
    LookForArtResult,
    MetadataEntry,
    Song,
    Stem,
    TagType,
    ToImport,
    ToImportAlbums,
} from "../App";
import { getImageFormat, isImageFile } from "../utils/FileUtils";
import {
    bottomBarNotification,
    importStatus,
    libraryUpdater,
    userSettings,
} from "./store";
const appWindow = getCurrentWebviewWindow();

/**
 * Reads metadata from Rust (lofty) and adds in generic IDs for common fields.
 */
export async function readMappedMetadataFromSong(
    song: Song,
): Promise<{ mappedMetadata: MetadataEntry[]; tagType: TagType | null }> {
    // 1. Fetch metadata from Rust
    const metadata: Song | null = await invoke("get_song_metadata", {
        event: {
            path: song.path,
            isImport: false,
            includeFolderArtwork: false,
            includeRawTags: true,
        },
    });
    if (!metadata) return { mappedMetadata: [], tagType: null };

    const mappedMetadata: MetadataEntry[] = [];

    for (let { id, value } of Object.values(metadata?.metadata) || []) {
        if (id?.length === 0) continue;

        if (typeof value === "number") {
            value = `${value}`;
        } else if (typeof value !== "string") {
            continue;
        }

        mappedMetadata.push({
            id,
            value,
        });
    }

    return { mappedMetadata, tagType: metadata.fileInfo.tagType as TagType };
}

export async function importPaths(
    selected: string[],
    background = true,
    percent = 0,
) {
    importStatus.update((importStatus) => ({
        ...importStatus,
        currentFolder: selected[0],
        status: "Reading metadata",
        isImporting: true,
        backgroundImport: background,
        percent,
    }));
    if (background) {
        bottomBarNotification.set({
            text: `Reading metadata`,
        });
    }
    console.log("toImport called");

    await invoke<ToImport>("scan_paths", {
        event: {
            paths: selected,
            recursive: true,
            process_albums: true,
            process_m3u: false,
            is_async: true,
            is_cover_fullcheck: get(userSettings).isCoverFullCheckEnabled,
        },
    });

    if (background) {
        bottomBarNotification.set({
            text: "Adding to library",
            timeout: 2000,
        });
    }
}

/**
 * Rust streams scanned songs/albums as separate, independently-timed chunks
 * (`import_chunk` / `import_albums` events) as it walks the filesystem, so
 * albums for a batch of songs can arrive before or after the songs
 * themselves. `upsert_tracks` copes with that ordering on its own (see
 * `db::mutations`), so this just needs to write each chunk as it arrives and
 * track when both streams are done to flip `importStatus` to "Done".
 *
 * This used to hand every chunk to a Web Worker running Dexie, since
 * IndexedDB writes could block the main thread. `invoke()` calls don't --
 * the write itself happens on Rust's writer thread -- so that indirection is
 * gone; chunks are written directly here.
 */
export async function startImportListener() {
    let songCount = 0;
    let albumCount = 0;
    let songChunksInFlight = 0;
    let gotAllSongChunks = false;
    let albumChunksInFlight = 0;
    let gotAllAlbumChunks = false;

    const maybeFinish = () => {
        if (
            songChunksInFlight === 0 &&
            gotAllSongChunks &&
            albumChunksInFlight === 0 &&
            gotAllAlbumChunks
        ) {
            importStatus.update((importStatus) => ({
                ...importStatus,
                status: "Done",
                isImporting: false,
                backgroundImport: false,
            }));
            scanExistingStems();
            bottomBarNotification.set(null);
            libraryUpdater.update((n) => n + 1);
        }
    };

    await appWindow.listen<ToImport>("import_chunk", async (event) => {
        const toImport = event.payload;
        gotAllSongChunks = toImport.progress === 100;

        let isBackground = false;
        importStatus.update((importStatus) => {
            isBackground = importStatus.backgroundImport;
            return {
                ...importStatus,
                status: "Writing to library",
                percent: toImport.progress,
            };
        });
        if (isBackground && songCount) {
            bottomBarNotification.set({
                text: `Updating library (${songCount} imported)`,
            });
        }

        if (!toImport.songs.length) {
            maybeFinish();
            return;
        }

        songCount += toImport.songs.length;
        songChunksInFlight++;
        try {
            await invoke("upsert_tracks", { tracks: toImport.songs });
        } finally {
            songChunksInFlight--;
            maybeFinish();
        }
    });

    await appWindow.listen<ToImportAlbums>("import_albums", async (event) => {
        const chunk = event.payload;
        gotAllAlbumChunks = chunk.progress === 100;

        if (!chunk.albums.length) {
            maybeFinish();
            return;
        }

        albumCount += chunk.albums.length;
        let isBackground = false;
        importStatus.update((importStatus) => {
            isBackground = importStatus.backgroundImport;
            return {
                ...importStatus,
                status: "Processing albums",
                percent: chunk.progress,
            };
        });
        if (isBackground && albumCount) {
            bottomBarNotification.set({
                text: `Updating library (${albumCount} albums imported)`,
            });
        }

        albumChunksInFlight++;
        try {
            await invoke("upsert_albums", { albums: chunk.albums });
        } finally {
            albumChunksInFlight--;
            maybeFinish();
        }
    });
}

export async function openTauriImportDialog() {
    // Open a selection dialog for directories
    const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: await audioDir(),
    });
    if (Array.isArray(selected)) {
        // user selected multiple directories
    } else if (selected === null) {
        // user cancelled the selection
    } else {
        console.log("selected", selected);

        // user selected a single directory
        // addFolder(selected); // JS IMPLEMENTATION

        await importPaths([selected], false);
    }
}

export async function runScan() {
    const settings = get(userSettings);

    for (const folder of settings.foldersToWatch) {
        bottomBarNotification.set({
            text: `Scanning ${folder} ...`,
            timeout: 2000,
        });
        await importPaths([folder], true);
    }
}

export async function getArtistProfileImage(
    folder: string,
): Promise<LookForArtResult> {
    let foundResult: LookForArtResult = null;
    try {
        // Not `readDir` from @tauri-apps/plugin-fs -- that blocks synchronously with no
        // spawn_blocking (see src-tauri/src/files.rs), and this runs every time an artist is
        // selected in the songbook, which may be a cloud-sync-backed folder.
        const files = await invoke<{ name: string }[]>("read_dir_blocking", { path: folder });
        for (const filename of files) {
            const extension = filename.name.split(".").pop();
            let format = getImageFormat(extension);
            if (
                isImageFile(filename.name) &&
                format &&
                filename.name.includes("profile")
            ) {
                foundResult = {
                    artworkSrc: convertFileSrc(`${folder}/${filename.name}`),
                    artworkFormat: format,
                    artworkFilenameMatch: filename.name,
                };
            }
        }

        return foundResult;
    } catch (err) {
        console.error("Couldn't find artwork " + err);
        return null;
    }
}

/**
 * Deletes tracks and any album left with none of its tracks remaining as a
 * result -- the orphan check now happens against the database's own state
 * (see `db::mutations::delete_tracks`) rather than being computed here.
 * @param tracks list of tracks
 */
export async function deleteFromLibrary(tracks: Song[]) {
    await invoke("delete_tracks", { trackIds: tracks.map((t) => t.id) });
    libraryUpdater.update((n) => n + 1);
}

export function getAlbumId(track: Song): string {
    return md5(
        `${track.path.replace(`/${track.file}`, "")} - ${track.album}`.toLowerCase(),
    );
}

export function songMatchesQuery(song: Song, query: string) {
    return (
        song.title.toLowerCase().includes(query.toLowerCase()) ||
        song.artist.toLowerCase().includes(query.toLowerCase()) ||
        song.albumArtist?.toLowerCase().includes(query.toLowerCase()) ||
        song.album.toLowerCase().includes(query.toLowerCase()) ||
        song.genre.some((g) => g.toLowerCase().includes(query.toLowerCase())) ||
        song.tags
            ?.map((t) => t.toLowerCase())
            .join(" ")
            .includes(query.toLowerCase())
    );
}

/**
 * Rescans one or more existing tracks (e.g. after writing new tags to the
 * file) and refreshes their metadata in place. User-owned fields (favourite,
 * play count, tags, ...) and stale-album cleanup are both handled server-side
 * now -- see `db::mutations::upsert_tracks` -- so this is just a straight
 * write of what the rescan produced, no client-side merge bookkeeping needed.
 */
export async function reImport(response: ToImport) {
    await invoke("upsert_tracks", { tracks: response.songs });
    if (response.albums.length) {
        await invoke("upsert_albums", { albums: response.albums });
    }
    libraryUpdater.update((n) => n + 1);
}

async function scanExistingStems() {
    const stems: Stem[] = await invoke("get_all_stems");

    console.log("Got stems", stems);

    // For each stem, check if a track exists that doesn't have it recorded yet.
    for (const stem of stems) {
        try {
            await invoke("add_track_stem", {
                trackId: stem.id,
                stem: { name: stem.name, path: stem.path },
            });
        } catch (e) {
            // Track may not exist (yet) -- not fatal, next scan will retry.
            console.error("Error recording stem for track", stem.id, e);
        }
    }
}
