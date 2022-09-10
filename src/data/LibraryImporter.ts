import { open } from "@tauri-apps/api/dialog";
import { BaseDirectory, readDir } from "@tauri-apps/api/fs";
import { audioDir } from "@tauri-apps/api/path";
import "iconify-icon";
import md5 from "md5";
import { db } from "./db";

import { convertFileSrc } from "@tauri-apps/api/tauri";
import * as musicMetadata from "music-metadata-browser";
import { importStatus, songsJustAdded, userSettings } from "./store";
import { getMapForTagType } from "./LabelMap";
import { get } from "svelte/store";
import type { MetadataEntry, Song } from "src/App";

let addedSongs: Song[] = [];

export async function addSong(
    filePath: string,
    fileName: string,
    singleFile = false
): Promise<Song | null> {
    if (!filePath.match(/\.(mp3|ogg|flac|aac|wav)$/)) {
        return null;
    }
    console.log("fetching file", fileName);
    const metadata = await musicMetadata.fetchFromUrl(convertFileSrc(filePath));
    const fileHash = md5(filePath);
    const tagType = metadata.format.tagTypes.length
        ? metadata.format.tagTypes[0]
        : null;
    const map = getMapForTagType(tagType, false);
    const metadataMapped: MetadataEntry[] = tagType
        ? metadata.native[tagType]
              .map((tag) => ({
                  genericId: map[tag.id],
                  id: tag.id,
                  value: tag.value
              }))
              .filter((tag) => typeof tag.value === "string")
        : [];
    try {
        const songToAdd: Song = {
            id: fileHash,
            path: filePath,
            file: fileName,
            title: metadata.common.title || fileName || "",
            artist: metadata.common.artist || "",
            album: metadata.common.album || "",
            year: metadata.common.year || 0,
            genre: metadata.common.genre || [],
            trackNumber: metadata.common.track.no || -1,
            duration: `${(~~(metadata.format.duration / 60))
                .toString()
                .padStart(2, "0")}:${(~~(metadata.format.duration % 60))
                .toString()
                .padStart(2, "0")}`,
            metadata: metadataMapped,
            fileInfo: metadata.format
        };
        // Remove image, too large
        let artworkIdx = songToAdd.metadata.findIndex(
            (t) => t.id === "METADATA_BLOCK_PICTURE"
        );
        if (artworkIdx > -1) {
            songToAdd.metadata.splice(artworkIdx, 1);
        }

        try {
            await db.songs.put(songToAdd);
        } catch(err) {
            console.error(err);
            // Catch 'already exists' case

        }
        if (singleFile) {
            songsJustAdded.update((songs) => {
                songs.push(songToAdd);
                return songs;
            });
        } else {
            addedSongs.push(songToAdd);
        }
        
        return await db.songs.get(fileHash);
    } catch (e) {
        console.error(e);
    }
}

async function processEntries(entries) {
    for (const [index, entry] of entries.entries()) {
        if (entry.children) {
            importStatus.update((importStatus) => ({
                ...importStatus,
                currentFolder: entry.name,
                totalTracks: entry.children.length
            }));
            await processEntries(
                entry.children.filter((e) => !e.name.startsWith("."))
            );
        } else {
            await addSong(entry.path, entry.name);
            importStatus.update((importStatus) => ({
                ...importStatus,
                importedTracks: index
            }));
        }
    }
}

export async function addFolder(folderPath) {
    importStatus.update((importStatus) => ({
        ...importStatus,
        isImporting: true
    }));
    const entries = await readDir(folderPath, {
        dir: BaseDirectory.App,
        recursive: true
    });

    console.log("entries", entries);
    await processEntries(entries.filter((e) => !e.name.startsWith(".")));
    importStatus.set({
        totalTracks: 0,
        importedTracks: 0,
        isImporting: false,
        currentFolder: ""
    });
    
    songsJustAdded.set(addedSongs);
    addedSongs = [];
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
        addFolder(selected);
    }
}

type ArtworkFolderFilename = "folder.jpg" | "cover.jpg" | "artwork.jpg";

interface LookForArtResult {
    artworkSrc?: string;
    artworkFormat: string;
    artworkFilenameMatch: string;
}

async function checkFolderArtworkByFilename(folder, artworkFilename) {
    const src = "asset://" + folder + artworkFilename;
    const response = await fetch(src);
    if (response.status === 200) {
        return {
            artworkSrc: src,
            artworkFormat: "image/jpeg",
            artworkFilenameMatch: artworkFilename
        };
    }
    return null;
}

export async function lookForArt(
    songPath,
    songFileName
): Promise<LookForArtResult | null> {
    const folder = songPath.replace(songFileName, "");
    const filenamesToSearch = get(userSettings).albumArtworkFilenames;
    let foundResult: LookForArtResult | null = null;

    // Check are there any images in the folder?
    try {
        for (const filename of filenamesToSearch) {
            const artworkResult = await checkFolderArtworkByFilename(
                folder,
                filename
            );
            if (artworkResult?.artworkSrc) {
                foundResult = artworkResult;
            }
        }
    } catch (err) {
        console.error("Couldn't find artwork " + err);
    }
    console.log("found result", foundResult);
    return foundResult;
}
