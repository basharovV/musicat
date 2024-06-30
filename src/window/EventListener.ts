import type { Event } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";
import { get } from "svelte/store";

import {
    importStatus,
    isFindFocused,
    isInfoPopupOpen,
    isQueueOpen,
    isSettingsOpen,
    isTrackInfoPopupOpen,
    shouldFocusFind,
    uiView
} from "../data/store";
import { db } from "../data/db";
import type { ToImport } from "../App";
import { CACHE_DIR, deleteCacheDirectory } from "../data/Cacher";
import { open } from "@tauri-apps/api/shell";
import { appDataDir, dataDir } from "@tauri-apps/api/path";

export function startMenuListener() {
    appWindow.listen("menu", async ({ event, payload }) => {
        console.log("menu", event);
        switch (payload) {
            case "about":
                console.log("about");
                isInfoPopupOpen.set(true);
                isSettingsOpen.set(false);
                isTrackInfoPopupOpen.set(false);
                break;
            case "settings":
                console.log("settings");
                isSettingsOpen.set(true);
                isInfoPopupOpen.set(false);
                isTrackInfoPopupOpen.set(false);
                break;
            case "find":
                console.log("find");
                shouldFocusFind.set({
                    target: "search",
                    action: get(isFindFocused) ? "unfocus" : "focus"
                });
                break;
            case "queue":
                isQueueOpen.set(!get(isQueueOpen));
                break;
            case "albums":
                uiView.set("albums");
                break;
            case "library":
                uiView.set("library");
                break;
            // DevTools
            case "clear-db":
                console.log("clear-db");
                await db.songs.clear();
                await db.albums.clear();
                await db.smartQueries.clear();
                await db.songProjects.clear();
                await db.artistProjects.clear();
                await db.scrapbook.clear();
                await db.playlists.clear();
                await db.delete();
                await deleteCacheDirectory();
                break;
            case "open-cache":
                try {
                    const dir = await appDataDir();
                    console.log("dir", dir);
                    open(`${dir}${CACHE_DIR}`);
                } catch (err) {
                    console.error(err);
                }
                break;
        }
    });
}

export async function listenForFileDrop(): Promise<Event<any>> {
    return new Promise(async (resolve, reject) => {
        try {
            const unlisten = await appWindow.listen(
                "tauri://file-drop",
                (event) => {
                    console.log("event", event);
                    resolve(event);
                    unlisten();
                }
            );
        } catch (e) {
            console.error(e);
            reject(e);
        }
    });
}
