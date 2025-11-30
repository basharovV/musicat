import { path } from "@tauri-apps/api";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { audioDir } from "@tauri-apps/api/path";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import { readDir } from "@tauri-apps/plugin-fs";
import { uniq } from "lodash-es";
import md5 from "md5";
import { get } from "svelte/store";
import type {
    Album,
    LookForArtResult,
    MetadataEntry,
    Song,
    Stem,
    TagType,
    ToImport,
    ToImportAlbums,
} from "../App";
import ImportWorker from "../ImportWorker?worker";
import { getImageFormat, isImageFile } from "../utils/FileUtils";
import { db } from "./db";
import { bottomBarNotification, importStatus, userSettings } from "./store";
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

export async function startImportListener() {
    let songCount = 0;
    let albumCount = 0;
    let chunksToProcess: ToImport[] = []; // progress numbers
    let gotAllChunks = false;
    var albumChunksToProcess: ToImportAlbums[] = [];
    let currentAlbumChunk = null;
    let gotAllAlbums = false;
    let isWaitingForAlbumChunk = false;
    let worker: Worker;

    const importAlbumChunk = async (chunk: ToImportAlbums) => {
        console.log("import new chunk", chunk);
        currentAlbumChunk = chunk;
        let isBackground = false;

        if (chunk.albums.length) {
            albumCount += chunk.albums.length;
            // Import album chunk
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
            console.log("importing album chunk", chunk);
            worker.postMessage({
                function: "bulkAlbumPut",
                toImport: chunk,
            });
        }
    };

    const onBulkAlbumPutDone = async (ev) => {
        console.log("bulkAlbumPutDone", ev.data, albumChunksToProcess);
        // Import chunk completed
        let idx = albumChunksToProcess.findIndex(
            (c) => c.progress === ev.data.progress,
        );
        console.log("idx", idx);
        if (idx !== -1) {
            albumChunksToProcess.splice(idx, 1);
        }

        console.log("albumChunksToProcess", albumChunksToProcess);
        if (!ev.data.done && albumChunksToProcess.length === 0) {
            // Wait for chunk
            isWaitingForAlbumChunk = true;
            return;
        } else if (!ev.data.done && albumChunksToProcess.length) {
            isWaitingForAlbumChunk = false;
            await importAlbumChunk(albumChunksToProcess[0]);
        } else if (
            ev.data.done &&
            gotAllAlbums &&
            albumChunksToProcess.length === 0
        ) {
            console.log("ALL ALBUMS IMPORTED., TERMINATING");
            worker.terminate();
            gotAllAlbums = false;
            worker = null;
            currentAlbumChunk = null;
        }

        if (ev.data.done) {
            importStatus.update((importStatus) => ({
                ...importStatus,
                status: "Done",
                isImporting: false,
                backgroundImport: false,
            }));
            bottomBarNotification.set(null);
        }
    };

    await appWindow.listen<ToImport>("import_chunk", async (event) => {
        if (!worker) {
            worker = new ImportWorker();
        }

        worker.onmessage = async (ev) => {
            switch (ev.data.event) {
                case "handleImportDone":
                    // Import chunk completed
                    chunksToProcess.splice(
                        chunksToProcess.findIndex(
                            (c) => c.progress === ev.data.progress,
                        ),
                        1,
                    );

                    console.log("handleImportDone");
                    console.log("chunksToProcess", chunksToProcess);
                    console.log("albumChunksToProcess", albumChunksToProcess);
                    console.log("gotAllChunks", gotAllChunks);
                    if (
                        chunksToProcess.length === 0 &&
                        !ev.data.done &&
                        albumChunksToProcess.length &&
                        !currentAlbumChunk
                    ) {
                        console.log("STARTING ALBUM IMPORT");
                        // We have more stuff to import?
                        await importAlbumChunk(albumChunksToProcess[0]);
                        return;
                    }
                    // Terminate if last one
                    else if (
                        chunksToProcess.length === 0 &&
                        gotAllChunks &&
                        albumChunksToProcess.length === 0 &&
                        toImport.done
                    ) {
                        console.log("Terminating");
                        worker.terminate();
                        chunksToProcess = [];
                        gotAllChunks = false;
                        worker = null;
                    }

                    // Import is complete (no albums)
                    if (toImport.done) {
                        importStatus.update((importStatus) => ({
                            ...importStatus,
                            status: "Done",
                            isImporting: false,
                            backgroundImport: false,
                        }));

                        scanExistingStems();
                        bottomBarNotification.set(null);
                    }
                    break;
                case "bulkAlbumPutDone":
                    await onBulkAlbumPutDone(ev);
                    scanExistingStems();
                    break;
            }
        };

        let isBackground = false;
        importStatus.update((importStatus) => {
            isBackground = importStatus.backgroundImport;
            return {
                ...importStatus,
                status: "Writing to library",
                percent: event.payload.progress,
            };
        });
        if (isBackground && songCount) {
            bottomBarNotification.set({
                text: `Updating library (${songCount} imported)`,
            });
        }
        const toImport = event.payload;
        gotAllChunks = event.payload.progress === 100;
        songCount += toImport.songs.length;
        if (toImport.songs.length) {
            chunksToProcess.push(toImport);
            worker.postMessage({
                function: "handleImport",
                toImport: toImport,
            });
        }
    });

    await appWindow.listen<ToImportAlbums>("import_albums", async (event) => {
        console.log("import_albums", event, albumChunksToProcess.length);
        gotAllAlbums = event.payload.progress === 100;
        albumChunksToProcess.push(event.payload);
        console.log("worker", worker);
        console.log("chunksToProcess", chunksToProcess);
        console.log("gotAllChunks", gotAllChunks);
        console.log("currentAlbumChunk", currentAlbumChunk);
        if (worker && chunksToProcess.length === 0 && gotAllChunks) {
            console.log("STARTING ALBUM IMPORT");
            // Start putting albums
            await importAlbumChunk(albumChunksToProcess[0]);
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
        const files = await readDir(folder);
        for (const filename of files) {
            const extension = filename.name.split(".").pop();
            let format = getImageFormat(extension);
            if (
                isImageFile(filename.name) &&
                format &&
                filename.name.includes("profile")
            ) {
                foundResult = {
                    artworkSrc: convertFileSrc(
                        await path.join(folder, filename.name),
                    ),
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
 * Deletes tracks and associated albums from the library
 * @param tracks list of tracks
 */
export async function deleteFromLibrary(tracks: Song[]) {
    const albumsToDelete = [];
    const checkedAlbums = {};

    for (const track of tracks) {
        const albumId = getAlbumId(track);

        if (!checkedAlbums[albumId]) {
            console.log("checking album", albumId);

            const album = await db.albums.get(albumId);

            if (
                album &&
                album.tracksIds.every((id) =>
                    tracks.map((t) => t.id).includes(id),
                )
            ) {
                albumsToDelete.push(album.id);
            }

            checkedAlbums[albumId] = true;
        }
    }
    console.log("albumsToDelete", albumsToDelete);

    await db.transaction("rw", db.songs, db.albums, async () => {
        await db.songs.bulkDelete(tracks.map((t) => t.id));
        if (albumsToDelete.length) {
            await db.albums.bulkDelete(albumsToDelete);
        }
    });
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

export async function reImport(
    response: ToImport,
    oldSongs: Song[],
    songIdToOldAlbumId: { [key: string]: string },
) {
    await db.transaction("rw", db.songs, db.albums, async () => {
        await reImportTracks(response.songs, oldSongs);

        for (const album of response.albums) {
            await reImportAlbum(album, songIdToOldAlbumId);
        }
    });
}

async function reImportAlbum(
    newAlbum: Album,
    songIdToOldAlbumId: { [key: string]: string },
) {
    const existingAlbum = await db.albums.get(newAlbum.id);

    // remove duplicate
    const oldAlbums = [existingAlbum?.id];

    for await (const trackId of newAlbum.tracksIds) {
        const albumId = songIdToOldAlbumId[trackId];

        if (albumId && !oldAlbums.includes(albumId)) {
            oldAlbums.push(albumId);

            await db.albums.delete(albumId);
        }
    }

    if (existingAlbum) {
        existingAlbum.artist = newAlbum.artist;
        existingAlbum.artwork = newAlbum.artwork;
        existingAlbum.displayTitle = newAlbum.displayTitle;
        existingAlbum.genre = newAlbum.genre;
        existingAlbum.title = newAlbum.title;
        existingAlbum.year = newAlbum.year;
        existingAlbum.tracksIds = uniq([
            ...existingAlbum.tracksIds,
            ...newAlbum.tracksIds,
        ]);

        await db.albums.put(existingAlbum);
    } else {
        await db.albums.put(newAlbum);
    }
}

async function reImportTracks(newSongs: Song[], oldSongs: Song[]) {
    // Map user-specific fields
    for (const oldSong of oldSongs) {
        const newSong = newSongs.find((s) => s.id === oldSong.id);
        if (newSong) {
            newSong.isFavourite = oldSong.isFavourite;
            newSong.playCount = oldSong.playCount;
            newSong.originCountry = oldSong.originCountry;
            newSong.tags = oldSong.tags;
            newSong.markers = oldSong.markers;
        }
    }

    await db.songs.bulkPut(newSongs);
}

async function scanExistingStems() {
    const stems: Stem[] = await invoke("get_all_stems");

    console.log("Got stems", stems);

    // For each stem, check if song exists that has 'stems',
    // if it doesn,t add it

    for (const stem of stems) {
        const song = await db.songs.get(stem.id);
        if (song) {
            if (!song.stems) {
                song.stems = [];
            }
            if (song.stems.find((s) => s.name === stem.name)) {
                continue;
            }
            song.stems.push({
                name: stem.name,
                path: stem.path,
            });
            await db.songs.put(song);
        }
    }
}
