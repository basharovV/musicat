import { exists, watchImmediate } from "@tauri-apps/plugin-fs";
import md5 from "md5";
import { get } from "svelte/store";
import { isAudioFile, isFileOrDirectory } from "../utils/FileUtils";
import { importPaths } from "./LibraryImporter";
import { db } from "./db";
import {
    bottomBarNotification,
    currentSongProjects,
    isFolderWatchUpdate,
    userSettings
} from "./store";
import { addScrapbookFile } from "./ArtistsToolkitData";

// can also watch an array of paths
export async function startWatchingLibraryFolders() {
    const settings = get(userSettings);
    const paths = settings.foldersToWatch;

    const startWatching = await watchImmediate(
        paths,
        async (event) => {
            console.log(event);
            let toastText = "";
            if (typeof event.type === "object") {
                isFolderWatchUpdate.set(true);

                // In this case we want to delete and re-add the track(s),
                // since the ID is based on the filepath
                for (const path of event.paths) {
                    const result = isFileOrDirectory(path);
                    console.log("result", result);
                    if (paths.includes(path)) {
                        continue;
                    }
                    if (result === "file") {
                        if (isAudioFile(path)) {
                            const file = path.split("/").pop();
                            let fileExists = false;
                            try {
                                fileExists = await exists(path);
                            } catch (err) {
                                console.error("err in fileexists");
                            }
                            const filehash = md5(path);
                            const song = await db.songs.get(filehash);

                            // New file
                            if (!song) {
                                bottomBarNotification.set({
                                    text: "Folder watcher: File added - updating library...",
                                    timeout: 2000
                                });
                                await importPaths([path], true);
                            }
                            // Deletion
                            else if (!fileExists && song) {
                                bottomBarNotification.set({
                                    text: "Folder watcher: File deleted - updating library...",
                                    timeout: 2000
                                });
                                await db.songs.delete(filehash);
                            }
                        }
                    } else if (result === "directory") {
                        const pathExists = await exists(path);
                        if (!pathExists) {
                            bottomBarNotification.set({
                                text: "Folder watcher: Folder deleted - updating library...",
                                timeout: 2000
                            });
                            // Deleted folder - delete all songs from db
                            const songsToDelete = await db.songs
                                .where("path")
                                .startsWith(path);
                            // Check if album needs to be deleted too
                            const songsToCheck = await songsToDelete.toArray();
                            const keys = await songsToDelete.primaryKeys();
                            await db.songs.bulkDelete(keys);
                            if (
                                songsToCheck.every(
                                    (s) => s.album === songsToCheck[0].album
                                )
                            ) {
                                const song = songsToCheck[0];
                                const albumPath = song.path.replace(
                                    `/${song.file}`,
                                    ""
                                );

                                let id = md5(
                                    `${albumPath} - ${song.album}`.toLowerCase()
                                );
                                await db.albums.delete(id);
                            }
                        } else {
                            bottomBarNotification.set({
                                text: "Folder watcher: Folder added - updating library...",
                                timeout: 2000
                            });
                            // await importPaths([path], true);
                        }
                    }
                }
            }
            isFolderWatchUpdate.set(false);
        },
        { recursive: true }
    );

    return startWatching;
}

export async function startWatchingScrapbookFolder() {
    const settings = get(userSettings);
    const scrapbookLocation = settings.scrapbookLocation;

    const startWatching = await watchImmediate(
        scrapbookLocation,
        async (event) => {
            console.log(event);
            if (typeof event.type === "object") {
                // In this case we want to delete and re-add the track(s),
                // since the ID is based on the filepath
                for (const path of event.paths) {
                    const result = isFileOrDirectory(path);
                    console.log("result", result);

                    if (result === "file") {
                        const file = path.split("/").pop();
                        let fileExists = false;
                        try {
                            fileExists = await exists(path);
                        } catch (err) {
                            console.error("err in fileexists");
                        }
                        const filehash = md5(path);
                        const song = await db.scrapbook.get(filehash);

                        // New file
                        if (fileExists && !song) {
                            bottomBarNotification.set({
                                text: "Folder watcher: File added - updating library...",
                                timeout: 2000
                            });
                            await addScrapbookFile(path);
                        }
                        // Deletion
                        else if (!fileExists && song) {
                            bottomBarNotification.set({
                                text: "Folder watcher: File deleted - updating library...",
                                timeout: 2000
                            });
                            await db.scrapbook.delete(filehash);
                        }
                    }
                }
            }
        },
        { recursive: true }
    );

    return startWatching;
}

export async function startWatchingSongbookArtistsFolder(artistName: string) {
    const settings = get(userSettings);
    const songbookLocation = settings.songbookLocation;
    const artistPath = songbookLocation + "/" + artistName;

    const startWatching = await watchImmediate(
        artistPath,
        async (event) => {
            console.log(event);
            if (typeof event.type === "object") {
                // Handle folders only
                for (const path of event.paths) {
                    const result = isFileOrDirectory(path);
                    console.log("result", result);
                    if (result === "file") {
                        //
                    } else if (result === "directory") {
                        let pathExists = false;
                        let folderName = path.split("/").pop();
                        try {
                            pathExists = await exists(path);
                        } catch (err) {
                            console.error("err in fileexists");
                        }
                        let songProjectExists = false;

                        // Check if song is in current list
                        get(currentSongProjects).forEach((song) => {
                            if (song === folderName) {
                                songProjectExists = true;
                            }
                        });

                        // New/updated folder
                        if (pathExists && !songProjectExists) {
                            bottomBarNotification.set({
                                text: "Folder watcher: Song added - updating songbook...",
                                timeout: 2000
                            });
                            await currentSongProjects.update((p) => {
                                p.push(folderName);
                                return p;
                            });
                        }
                        // Deletion
                        else if (!pathExists && songProjectExists) {
                            bottomBarNotification.set({
                                text: "Folder watcher: Song deleted - updating songbook...",
                                timeout: 2000
                            });
                            // Remove from list
                            await currentSongProjects.update((p) => {
                                p.splice(p.indexOf(folderName), 1);
                                return p;
                            });
                        }
                    }
                }
            }
        },
        { recursive: true }
    );

    return startWatching;
}
