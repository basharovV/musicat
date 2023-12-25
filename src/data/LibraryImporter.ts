import { open } from "@tauri-apps/api/dialog";
import { BaseDirectory, exists, readDir } from "@tauri-apps/api/fs";
import { audioDir } from "@tauri-apps/api/path";
import "iconify-icon";
import md5 from "md5";
import { db } from "./db";

import { convertFileSrc } from "@tauri-apps/api/tauri";
import * as musicMetadata from "music-metadata-browser";
import { importStatus, songsJustAdded, userSettings } from "./store";
import { getMapForTagType } from "./LabelMap";
import { get } from "svelte/store";
import type { Album, MetadataEntry, Song } from "src/App";
import { getImageFormat, isAudioFile, isImageFile } from "../utils/FileUtils";

let addedSongs: Song[] = [];

export async function getMetadataFromFile(filePath: string, fileName: string) {
    if (!isAudioFile(fileName)) {
        return null;
    }
    return await musicMetadata.fetchFromUrl(convertFileSrc(filePath), {
        duration: false,
        skipCovers: false,
        skipPostHeaders: true
    });
}

/**
 * When we need to get the metadata directly from the file on disk.
 * @param song
 * @returns
 */
export async function readMappedMetadataFromSong(
    song: Song
): Promise<MetadataEntry[]> {
    const metadata = await getMetadataFromFile(song.path, song.file);
    console.log('meta', metadata);
    const tagType = metadata.format.tagTypes.length
        ? metadata.format.tagTypes[0]
        : null;
    const map = getMapForTagType(tagType, false);
    const metadataMapped: MetadataEntry[] = tagType
        ? metadata.native[tagType]
              .map((tag) => ({
                  genericId: map && tag ? map[tag.id] : "unknown",
                  id: tag.id,
                  value: tag.value
              }))
              .filter((tag) => typeof tag.value === "string")
        : [];
    return metadataMapped;
}

export async function getSongFromMetadata(
    filePath: string,
    fileName: string,
    metadata: musicMetadata.IAudioMetadata
) {
    // console.log('metadata: ', metadata);
    const fileHash = md5(filePath);
    const tagType = metadata.format.tagTypes.length
        ? metadata.format.tagTypes[0]
        : null;
    const map = getMapForTagType(tagType, false);
    const metadataMapped: MetadataEntry[] = tagType
        ? metadata.native[tagType]
              .map((tag) => ({
                  genericId: map && tag ? map[tag.id] : "unknown",
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
            fileInfo: metadata.format,
            isFavourite: false,
            playCount: 0
        };
        console.log("song: ", songToAdd);
        // Remove image, too large
        let artworkIdx = songToAdd.metadata.findIndex(
            (t) => t.id === "METADATA_BLOCK_PICTURE"
        );
        if (artworkIdx > -1) {
            songToAdd.metadata.splice(artworkIdx, 1);
        }
        return songToAdd;
    } catch (e) {
        console.error(e);
    }
}

async function addAlbumArtworkFromSong(
    metadata: musicMetadata.IAudioMetadata,
    song: Song,
    newAlbum: Album
) {
    if (metadata.common.picture?.length) {
        const artworkFormat = metadata.common.picture[0].format;
        const artworkBuffer = metadata.common.picture[0].data;
        const artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
            "base64"
        )}`;
        // console.log("artworkSrc", artworkSrc);
        newAlbum.artwork = {
            src: artworkSrc,
            format: artworkFormat,
            size: {
                width: metadata.common.picture[0]["width"],
                height: metadata.common.picture[0]["height"]
            }
        };
    } else {
        const artwork = await lookForArt(song.path, song.file);
        if (artwork) {
            const artworkSrc = artwork.artworkSrc;
            const artworkFormat = artwork.artworkFormat;
            newAlbum.artwork = {
                src: artworkSrc,
                format: artworkFormat,
                size: {
                    width: 200,
                    height: 200
                }
            };
        }
    }
}

async function getAlbumFromMetadata(
    song: Song,
    metadata: musicMetadata.IAudioMetadata
) {
    const existingAlbum = await db.albums.get(
        md5(`${song.artist} - ${song.album}`)
    );
    if (existingAlbum) {
        existingAlbum.tracksIds.push(song.id);
        existingAlbum.trackCount++;
        if (!existingAlbum.artwork) {
            await addAlbumArtworkFromSong(metadata, song, existingAlbum);
        }
        return existingAlbum;
        // Re-order trackIds in album order?
    } else {
        const newAlbum: Album = {
            id: md5(`${song.artist} - ${song.album}`),
            title: song.album,
            artist: song.artist,
            genre: song.genre,
            duration: song.duration,
            path: song.path.replace(`/${song.file}`, ""),
            trackCount: 1,
            year: song.year,
            tracksIds: [song.id]
        };
        await addAlbumArtworkFromSong(metadata, song, newAlbum);
        return newAlbum;
    }
}

async function updateAlbum(song: Song, metadata: musicMetadata.IAudioMetadata) {
    const albumToSet = await getAlbumFromMetadata(song, metadata);

    if (!albumToSet) {
        return;
    }
    await db.albums.put(albumToSet);
}

export async function addSong(
    filePath: string,
    fileName: string,
    singleFile = false
): Promise<Song | null> {
    if (singleFile) {
        songsJustAdded.set([]);
    }
    const metadata = await getMetadataFromFile(filePath, fileName);
    if (!metadata) return;
    const songToAdd = await getSongFromMetadata(filePath, fileName, metadata);

    try {
        if (!songToAdd) {
            return;
        }
        await db.songs.put(songToAdd);
        updateAlbum(songToAdd, metadata);
    } catch (err) {
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

    return await db.songs.get(songToAdd.id);
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

export async function addPaths(paths: string[]) {
    for (const path of paths) {
        if (isAudioFile(path)) {
            const file = path.split("/").pop();
            await addSong(path, file, true);
        } else if (path.match(`/.+(?=/)/`).length) {
            await addFolder(path);
        }
    }
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
    const src = folder + artworkFilename;
    let fileExists = false;
    try {
        fileExists = await exists(src);

        console.log("fileExists", fileExists);

        if (fileExists) {
            return {
                artworkSrc: convertFileSrc(src),
                artworkFormat: "image/jpeg",
                artworkFilenameMatch: artworkFilename
            };
        }
        return null;
    } catch (err) {
        console.error(err);
        return null;
    }
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

    if (!foundResult) {
        // Otherwise just use the first image that's in the folder
        try {
            const files = await readDir(folder);
            for (const filename of files) {
                const extension = filename.name.split(".").pop();
                let format = getImageFormat(extension);
                if (isImageFile(filename.name) && format) {
                    foundResult = {
                        artworkSrc: convertFileSrc(filename.path),
                        artworkFormat: format,
                        artworkFilenameMatch: filename.name
                    };
                }
            }
        } catch (err) {
            console.error("Couldn't find artwork " + err);
        }
    }
    console.log("found result", foundResult);
    return foundResult;
}
