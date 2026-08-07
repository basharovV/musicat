import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { exists, watchImmediate } from "@tauri-apps/plugin-fs";
import md5 from "md5";
import { get } from "svelte/store";
import type { Song } from "../App";
import { isAudioFile, isFileOrDirectory } from "../utils/FileUtils";
import { importPaths } from "./LibraryUtils";
import { queryAllTracks } from "./query";
import {
    bottomBarNotification,
    currentSongProjects,
    isFolderWatchUpdate,
    libraryUpdater,
    scrapbookItems,
    songbookArtists,
    songbookSelectedArtist,
    userSettings,
} from "./store";
import {
    addScrapbookFile,
    loadArtistsFromSongbook,
    loadSongProjectsForArtist,
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
                            const song = await invoke<Song | null>(
                                "get_track_by_id",
                                { id: filehash },
                            );

                            console.log(
                                "[Folder watcher] File exists",
                                fileExists,
                            );

                            // New file
                            if (!song && fileExists) {
                                console.log("[Folder watcher] File added");
                                bottomBarNotification.set({
                                    text: "Folder watcher: File added - updating library...",
                                    timeout: 2000,
                                });
                                await importPaths([path], true);
                            }
                            // Deletion
                            else if (!fileExists && song) {
                                console.log("[Folder watcher] File deleted");
                                bottomBarNotification.set({
                                    text: "Folder watcher: File deleted - updating library...",
                                    timeout: 2000,
                                });
                                await invoke("delete_tracks", {
                                    trackIds: [filehash],
                                });
                                libraryUpdater.update((n) => n + 1);
                            }
                        }
                    } else if (result === "directory") {
                        const pathExists = await exists(path);
                        if (!pathExists) {
                            console.log("[Folder watcher] Folder deleted");
                            bottomBarNotification.set({
                                text: "Folder watcher: Folder deleted - updating library...",
                                timeout: 2000,
                            });
                            // Deleted folder - delete all songs from db
                            // (delete_tracks also removes any album left with
                            // no tracks as a result, so no separate album
                            // deletion step is needed here)
                            const allTracks = await queryAllTracks({});
                            const songsToDelete = allTracks.filter((s) =>
                                s.path.startsWith(path),
                            );
                            await invoke("delete_tracks", {
                                trackIds: songsToDelete.map((s) => s.id),
                            });
                            libraryUpdater.update((n) => n + 1);
                        } else {
                            console.log("[Folder watcher] Folder added");
                            bottomBarNotification.set({
                                text: "Folder watcher: Folder added - updating library...",
                                timeout: 2000,
                            });
                            // await importPaths([path], true);
                        }
                    }
                }
            }
            isFolderWatchUpdate.set(false);
        },
        { recursive: true },
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
                            // Not `exists()` from @tauri-apps/plugin-fs -- that blocks
                            // synchronously with no spawn_blocking (see files.rs), and this
                            // watcher fires on every fs event in a folder that may be
                            // cloud-sync-backed.
                            fileExists = await invoke<boolean>("path_exists_blocking", { path });
                        } catch (err) {
                            console.error("err in fileexists");
                        }
                        const filehash = md5(path);
                        const song = get(scrapbookItems).find(
                            (i) => i.id === filehash,
                        );

                        // New file
                        if (fileExists && !song) {
                            bottomBarNotification.set({
                                text: "Folder watcher: File added - updating library...",
                                timeout: 2000,
                            });
                            const added = await addScrapbookFile(path);
                            scrapbookItems.update((items) => [...items, added]);
                        }
                        // Deletion
                        else if (!fileExists && song) {
                            bottomBarNotification.set({
                                text: "Folder watcher: File deleted - updating library...",
                                timeout: 2000,
                            });
                            await invoke("delete_scrapbook_item", { id: filehash });
                            scrapbookItems.update((items) =>
                                items.filter((i) => i.id !== filehash),
                            );
                        }
                    }
                }
            }
        },
        { recursive: true },
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
                                await loadArtistsFromSongbook(),
                            );
                        } else if (depth === 3) {
                            // Only update if matches currently selected artist
                            const selectedArtist = get(songbookSelectedArtist);
                            if (artistName === selectedArtist?.name) {
                                currentSongProjects.set(
                                    await loadSongProjectsForArtist(selectedArtist),
                                );
                            }
                        }
                    }
                }
            }
        },
        { recursive: true },
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
                    if (result === "file" && path.endsWith(".m3u")) {
                        const playlistFileName = path.split("/").pop();
                        console.log(
                            "[watcher] playlistName: ",
                            playlistFileName,
                        );

                        // We can't quite rely on the WatchEventKind to distinguish access/modify/remove,
                        // so let's just re-scan the folder. This also tracks
                        // any playlist we've never seen before, establishing
                        // its baseline hash before the divergence check below
                        // can run against it.
                        await scanPlaylists();

                        // Deletes leave nothing to hash -- check_playlist_divergence
                        // reads the file itself and would just error, and
                        // scanPlaylists()/delete_playlist already handles
                        // deletions via the folder rescan.
                        const fileStillExists = await exists(path);
                        if (fileStillExists) {
                            try {
                                await invoke("check_playlist_divergence", {
                                    playlistId: md5(path),
                                    filePath: path,
                                });
                            } catch (e) {
                                console.error(
                                    "[watcher] Error checking playlist divergence",
                                    e,
                                );
                            }
                        }
                    }
                }
            }
        },
        { recursive: true },
    );

    return startWatching;
}

/**
 * Listens for the Rust-side `playlist_diverged` event (emitted by
 * `check_playlist_divergence` above) and prompts the user to reimport via
 * the bottom bar. Call once at startup, alongside `startWatchingPlaylistsFolder`.
 */
export async function listenForPlaylistDivergence() {
    return listen<{ playlistId: string }>(
        "playlist_diverged",
        ({ payload: { playlistId } }) => {
            bottomBarNotification.set({
                text: "A playlist file changed outside Musicat.",
                action: {
                    label: "Reimport",
                    onClick: async () => {
                        try {
                            await invoke("reimport_playlist_from_file", {
                                playlistId,
                            });
                            bottomBarNotification.set({
                                text: "Playlist reimported",
                                timeout: 2000,
                            });
                        } catch (e) {
                            console.error(
                                "[watcher] Error reimporting playlist",
                                e,
                            );
                            bottomBarNotification.set({
                                text: "Couldn't reimport playlist",
                                timeout: 2000,
                            });
                        }
                    },
                },
            });
        },
    );
}
