import type { Event } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";
import { get } from "svelte/store";

import {
    importStatus,
    isFindFocused,
    isInfoPopupOpen,
    isSettingsOpen,
    isTrackInfoPopupOpen,
    shouldFocusFind
} from "../data/store";
import { db } from "../data/db";
import type { ToImport } from "../App";
import { deleteCacheDirectory } from "../data/Cacher";

export function startMenuListener() {
    appWindow.listen("menu", async ({ event, payload }) => {
        console.log("menu", event);
        switch (payload) {
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
