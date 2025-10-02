import { type Event } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { get } from "svelte/store";

import { path } from "@tauri-apps/api";
import { appConfigDir, appDataDir } from "@tauri-apps/api/path";
import { CACHE_DIR, deleteCacheDirectory } from "../data/Cacher";
import { deleteDatabase, exportDatabase, importDatabase } from "../data/db";
import { openTauriImportDialog } from "../data/LibraryUtils";
import {
    isCompactView,
    isFindFocused,
    isLyricsOpen,
    isQueueOpen,
    popupOpen,
    queue,
    shouldFocusFind,
    uiView,
} from "../data/store";

import toast from "svelte-french-toast";
import { openPath } from "@tauri-apps/plugin-opener";

const appWindow = getCurrentWebviewWindow();

export function startMenuListener() {
    appWindow.listen("menu", async ({ event, payload }) => {
        console.log("menu", event);
        switch (payload) {
            case "about":
                console.log("about");
                popupOpen.set("info");
                break;
            case "settings":
                console.log("settings");
                popupOpen.set("settings");
                break;
            case "find":
                console.log("find");
                shouldFocusFind.set({
                    target: "search",
                    action: get(isFindFocused) ? "unfocus" : "focus",
                });
                break;
            case "import":
                openTauriImportDialog();
                break;
            case "queue":
                if (get(isCompactView)) {
                    uiView.set("queue");
                }
                isQueueOpen.set(!get(isQueueOpen));
                break;
            case "albums":
                uiView.set("albums");
                break;
            case "library":
                uiView.set("library");
                break;
            case "lyrics":
                isLyricsOpen.set(!get(isLyricsOpen));
                break;
            case "prune":
                uiView.set("prune");
                break;
            // DevTools
            case "clear-data":
                console.log("clear-data");
                await deleteDatabase();
                await deleteCacheDirectory();
                queue.set([]);
                break;
            case "import-db":
                console.log("import-db");
                importDatabase();
                break;
            case "export-db":
                console.log("export-db");
                exportDatabase();
                break;
            case "open-cache":
                try {
                    const dir = await appDataDir();
                    console.log("dir", dir);
                    openPath(await path.join(dir, CACHE_DIR));
                } catch (err) {
                    console.error(err);
                }
                break;
            case "open-config":
                try {
                    const dir = await appConfigDir();
                    openPath(dir);
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
                },
            );
        } catch (e) {
            console.error(e);
            reject(e);
        }
    });
}
export async function startErrorListener() {
    appWindow.listen("error", (event) => {
        toast.error(event.payload);
    });
}
