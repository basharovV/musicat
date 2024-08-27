import { exists, watchImmediate } from "@tauri-apps/plugin-fs";
import md5 from "md5";
import { get } from "svelte/store";
import { isAudioFile, isFileOrDirectory } from "../utils/FileUtils";
import { importPaths } from "./LibraryImporter";
import { db } from "./db";
import {
    bottomBarNotification,
    isFolderWatchUpdate,
    userSettings
} from "./store";

// can also watch an array of paths
export async function startWatching() {
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

                                let id = md5(`${albumPath} - ${song.album}`.toLowerCase());
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
