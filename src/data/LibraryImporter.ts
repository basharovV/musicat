import { open } from "@tauri-apps/plugin-dialog";
import ImportWorker from "../ImportWorker?worker";
import { invoke } from "@tauri-apps/api/core";
import { BaseDirectory, exists, readDir } from "@tauri-apps/plugin-fs";
import { audioDir } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import md5 from "md5";
import * as musicMetadata from "music-metadata-browser";
import type {
    ToImportAlbums,
    Album,
    LookForArtResult,
    MetadataEntry,
    Song,
    TagType,
    ToImport
} from "../App";
import { get } from "svelte/store";
import { getImageFormat, isAudioFile, isImageFile } from "../utils/FileUtils";
import { getMapForTagType, getTagTypeFromCodec } from "./LabelMap";
import { db } from "./db";
import {
    bottomBarNotification,
    importStatus,
    shouldShowToast,
    songsJustAdded,
    userSettings
} from "./store";
const appWindow = getCurrentWebviewWindow();

let addedSongs: Song[] = [];

export async function getMetadataFromFile(filePath: string, fileName: string) {
    if (!isAudioFile(fileName)) {
        return null;
    }
    return await musicMetadata.fetchFromUrl(convertFileSrc(filePath), {
        duration: false,
        skipCovers: false,
        skipPostHeaders: true,
        includeChapters: false
    });
}

/**
 * When we need to get the metadata directly from the file on disk.
 * @param song
 * @returns
 */
export async function readMappedMetadataFromSong(
    song: Song
): Promise<{ mappedMetadata: MetadataEntry[]; tagType: TagType }> {
    console.log("song read", song);
    const metadata = await getMetadataFromFile(song.path, song.file);
    console.log("meta", metadata);
    const tagType = metadata.format.tagTypes.length
        ? metadata.format.tagTypes[0]
        : getTagTypeFromCodec(metadata.format.codec);
    const map = getMapForTagType(tagType, false);
    const mappedMetadata: MetadataEntry[] = tagType
        ? metadata?.native[tagType]
              ?.map((tag) => ({
                  genericId: map && tag ? map[tag.id] : "unknown",
                  id: tag.id,
                  value: tag.value
              }))
              .filter((tag) => typeof tag.value === "string") ?? []
        : [];
    return { mappedMetadata, tagType };
}

export async function importPaths(
    selected: string[],
    background = true,
    percent = 0
) {
    importStatus.update((importStatus) => ({
        ...importStatus,
        currentFolder: selected[0],
        status: "Reading metadata",
        isImporting: true,
        backgroundImport: background,
        percent
    }));
    if (background) {
        bottomBarNotification.set({
            text: `Reading metadata`
        });
    }
    console.log("toImport called");

    await invoke<ToImport>("scan_paths", {
        event: {
            paths: selected,
            recursive: true,
            process_albums: true,
            is_async: true
        }
    });

    if (background) {
        bottomBarNotification.set({
            text: "Adding to library",
            timeout: 2000
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
                    percent: chunk.progress
                };
            });
            if (isBackground && albumCount) {
                bottomBarNotification.set({
                    text: `Updating library (${albumCount} albums imported)`
                });
            }
            console.log("importing album chunk", chunk);
            worker.postMessage({
                function: "bulkAlbumPut",
                toImport: chunk
            });
        }
    };

    const onBulkAlbumPutDone = async (ev) => {
        console.log("bulkAlbumPutDone", ev.data, albumChunksToProcess);
        // Import chunk completed
        let idx = albumChunksToProcess.findIndex(
            (c) => c.progress === ev.data.progress
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
                backgroundImport: false
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
                            (c) => c.progress === ev.data.progress
                        ),
                        1
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
                            backgroundImport: false
                        }));

                        bottomBarNotification.set(null);
                    }
                    break;
                case "bulkAlbumPutDone":
                    await onBulkAlbumPutDone(ev);
                    break;
            }
        };

        let isBackground = false;
        importStatus.update((importStatus) => {
            isBackground = importStatus.backgroundImport;
            return {
                ...importStatus,
                status: "Writing to library",
                percent: event.payload.progress
            };
        });
        if (isBackground && songCount) {
            bottomBarNotification.set({
                text: `Updating library (${songCount} imported)`
            });
        }
        const toImport = event.payload;
        gotAllChunks = event.payload.progress === 100;
        songCount += toImport.songs.length;
        if (toImport.songs.length) {
            chunksToProcess.push(toImport);
            worker.postMessage({
                function: "handleImport",
                toImport: toImport
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
        defaultPath: await audioDir()
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

export async function rescanAlbumArtwork(album: Album) {
    // console.log("adding artwork from song", song, newAlbum);

    const response = await invoke<ToImport>("scan_paths", {
        event: {
            paths: [album.path],
            recursive: false,
            process_albums: true,
            is_async: false
        }
    });

    // TODO: Write updated album with updated artwork to DB
}

export async function runScan() {
    const settings = get(userSettings);

    for (const folder of settings.foldersToWatch) {
        bottomBarNotification.set({
            text: `Scanning ${folder} ...`,
            timeout: 2000
        });
        await importPaths([folder], true);
    }
}

export async function getArtistProfileImage(
    folder: string
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
                    artworkSrc: convertFileSrc(folder + "/" + filename.name),
                    artworkFormat: format,
                    artworkFilenameMatch: filename.name
                };
            }
        }

        return foundResult;
    } catch (err) {
        console.error("Couldn't find artwork " + err);
        return null;
    }
}
