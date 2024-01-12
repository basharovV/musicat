import md5 from "md5";
import { db } from "./data/db";
// import { bottomBarNotification } from "./data/store";
self.window = self;

import { invokeTauriCommand } from "@tauri-apps/api/helpers/tauri";
import type { Album, LookForArtResult, Song, ToImport, UserSettings } from "./App";
import { cacheArtwork } from "./data/Cacher";
import { getImageFormat, isImageFile } from "./utils/FileUtils";

export async function handleImport(toImport: ToImport) {
    await db
        .transaction("rw", db.songs, db.albums, async () => {
            db.songs
                .bulkPut(
                    toImport.songs.map((s) => {
                        const { artwork, ...rest } = s;
                        return rest;
                    })
                )
                .then(function (lastKey) {
                    console.log(`
                        Done putting ${toImport.songs.length} songs`
                    );
                })
                .catch("BulkError", function (e) {
                    // Explicitly catching the bulkAdd() operation makes those successful
                    // additions commit despite that there were errors.
                    console.error(
                        "Some raindrops did not succeed. However, " +
                            e.failures.length +
                            " raindrops was added successfully"
                    );
                });

            const albumsToPut = toImport.songs.reduce((albums, song) => {
                const albumPath = song.path.replace(`/${song.file}`, "");

                let id = md5(`${albumPath} - ${song.album}`);
                if (albums[id] !== undefined) {
                    albums[id].trackCount++;
                    albums[id].tracksIds.push(song.id);
                    albums[id].lossless = albums[id].lossless && song.fileInfo.lossless;
                } else {
                    albums[id] = {
                        id,
                        title: song.album,
                        artist: song.artist,
                        genre: song.genre,
                        duration: song.duration,
                        path: song.path.replace(`/${song.file}`, ""),
                        trackCount: 1,
                        year: song.year,
                        tracksIds: [song.id],
                        lossless: song.fileInfo.lossless
                    };
                }
                return albums;
            }, {});

            db.albums.bulkPut(Object.values(albumsToPut));
        })
        .catch("BulkError", (err) => {
            //
            // Transaction Failed
            //

            console.error(err.stack);
        });
}

async function bulkAlbumPut(albumsToPut: Album[]) {

    console.log("albumsToPut", albumsToPut);
    await db.albums
        .bulkPut(Object.values(albumsToPut))
        .catch("BulkError", (err) => {
            // Explicitly catching the bulkAdd() operation makes those successful
            // additions commit despite that there were errors.
            console.error(
                "Some album writes did not succeed. However, " +
                    err.failures.length +
                    " albums was added successfully"
            );
        });
}

async function addAlbumArtworkFromSong(
    song: Song,
    newAlbum: Album,
    userSettings: UserSettings
) {
    if (song.artwork?.data && song.artwork?.format) {
        const artworkFormat = song.artwork.format;

        const path = await cacheArtwork(
            Uint8Array.from(song.artwork.data),
            newAlbum.id,
            song.artwork.format
        );

        // console.log("artworkSrc", artworkSrc);
        newAlbum.artwork = {
            src: path,
            format: artworkFormat,
            size: {
                width: 200,
                height: 200
            }
        };
    } else {
        const artwork = await lookForArt(song.path, song.file, userSettings);
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

/**
 * To improve performance, we process the album artworks (including looking in folders as fallbacks)
 * after the songs have been imported, and the albums have been added.
 * @param songs List of songs that have been imported
 */
async function addArtworksToAllAlbums(
    songs: Song[],
    userSettings: UserSettings
) {
    // Here we can use Promise.all for concurrency since we don't depend on order
    const albumsToPut = await Promise.all(
        songs.map(async (song) => {
            const existingAlbum = await db.albums.get(
                md5(`${song.artist} - ${song.album}`)
            );
            if (existingAlbum) {
                if (!existingAlbum.artwork) {
                    await addAlbumArtworkFromSong(
                        song,
                        existingAlbum,
                        userSettings
                    );
                }
                return existingAlbum;
                // Re-order trackIds in album order?
            }
        })
    );
    await db.albums.bulkPut(albumsToPut);
}

async function handleMessage(e) {
    switch (e.data.function) {
        case "handleImport":
            await handleImport(e.data.toImport);
            postMessage("handleImport");
            break;
        case "bulkAlbumPut":
            await bulkAlbumPut(e.data.albums);
            postMessage("bulkAlbumPut");
            break;
        case "addArtworksToAllAlbums":
            await addArtworksToAllAlbums(e.data.songs, e.data.userSettings);
            postMessage("addArtworksToAllAlbums");
            break;
        default:
            break;
    }
}

async function readDir(dir, options = {}): Promise<any> {
    return invokeTauriCommand({
        __tauriModule: "Fs",
        message: {
            cmd: "readDir",
            path: dir,
            options
        }
    });
}
async function exists(path, options = {}): Promise<boolean> {
    return invokeTauriCommand({
        __tauriModule: "Fs",
        message: {
            cmd: "exists",
            path,
            options
        }
    });
}

async function checkFolderArtworkByFilename(folder, artworkFilename) {
    const src = folder + artworkFilename;
    let fileExists = false;
    try {
        fileExists = await exists(src);

        console.log("fileExists", fileExists);

        if (fileExists) {
            return {
                artworkSrc: src, // Store like this but convertFileSrc when displaying
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
    songFileName,
    userSettings: UserSettings
): Promise<LookForArtResult | null> {
    const folder = songPath.replace(songFileName, "");
    const filenamesToSearch = userSettings.albumArtworkFilenames;
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
                        artworkSrc: filename.path,
                        artworkFormat: format,
                        artworkFilenameMatch: filename.name
                    };
                }
            }
        } catch (err) {
            console.error("Couldn't find artwork " + err);
        }
    }
    // console.log("found result", foundResult);
    return foundResult;
}

onmessage = (e) => {
    console.log("Message received from main script", e.data);
    handleMessage(e);
    console.log("Posting message back to main script");
};
