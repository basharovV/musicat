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
    songbookArtists,
    songbookSelectedArtist,
    userSettings
} from "./store";
import {
    addScrapbookFile,
    loadArtistsFromSongbook,
    loadSongProjectsForArtist
} from "./ArtistsToolkitData";
import { scanPlaylists } from "./M3UUtils";

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

export async function startWatchingSongbookFolder() {
    const settings = get(userSettings);
    const songbookLocation = settings.songbookLocation;

    const startWatching = await watchImmediate(
        songbookLocation,
        async (event) => {
            console.log("[watcher] songbook: ", event);
            if (typeof event.type === "object") {
                // Handle folders only
                for (const path of event.paths) {
                    const result = isFileOrDirectory(path);
                    const parent = path.split("/").slice(0, -1).join("/");
                    console.log("[watcher] parent: ", parent);
                    // artist folder : songbookLocation/artistName
                    // artist profile pic: songbookLocation/artistName/profile.jpg

                    // song folder: songbookLocation/artistName/songName
                    // song file: songbookLocation/artistName/songName/songName.txt

                    // Check for folder type
                    if (result === "directory") {
                        // Based on how many levels deep the folder is (relative to songbookLocation)
                        const depth = path
                            .substring(songbookLocation.length - 1, path.length)
                            .split("/").length;
                        console.log("[watcher] folderDepth: ", depth);

                        // Get the artist name from the folder name on the first level

                        // Get depth difference between songbookLocation and path

                        const isArtist = depth === 2;
                        const isSong = depth === 3;

                        let artistName;
                        if (depth === 2) {
                            artistName = path.split("/").pop();
                        } else if (depth === 3) {
                            artistName = parent.split("/").pop();
                        }
                        console.log("[watcher] artistName: ", artistName);

                        if (depth === 2) {
                            // Check if artist folder exists
                            songbookArtists.set(
                                await loadArtistsFromSongbook()
                            );
                        } else if (depth === 3) {
                            // Only update if matches currently selected artist
                            if (
                                artistName === get(songbookSelectedArtist).name
                            ) {
                                currentSongProjects.set(
                                    await loadSongProjectsForArtist(artistName)
                                );
                            }
                        }
                    }
                }
            }
        },
        { recursive: true }
    );

    return startWatching;
}

export async function startWatchingPlaylistsFolder() {
    const settings = get(userSettings);
    const playlistsLocation = settings.playlistsLocation;

    const startWatching = await watchImmediate(
        playlistsLocation,
        async (event) => {
            console.log("[watcher] songbook: ", event);
            if (typeof event.type === "object") {
                // Handle files only
                for (const path of event.paths) {
                    const result = isFileOrDirectory(path);
                    const parent = path.split("/").slice(0, -1).join("/");
                    console.log("[watcher] parent: ", parent);
                    // expected structure: playlistsLocation/playlistName.m3u

                    // Check for file
                    if (result === "file") {
                        const playlistFileName = path.split("/").pop();
                        console.log(
                            "[watcher] playlistName: ",
                            playlistFileName
                        );

                        // We can't quite rely on the WatchEventKind to distinguish access/modify/remove,
                        // so let's just re-scan the folder
                        await scanPlaylists();
                    }
                }
            }
        },
        { recursive: true }
    );

    return startWatching;
}
