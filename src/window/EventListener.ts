import { TauriEvent, type Event } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { get } from "svelte/store";

import {
    importStatus,
    isFindFocused,
    popupOpen,
    isLyricsOpen,
    isQueueOpen,
    shouldFocusFind,
    uiView,
    queue,
    isCompactView,
} from "../data/store";
import { db, deleteDatabase, exportDatabase, importDatabase } from "../data/db";
import type { ToImport } from "../App";
import { CACHE_DIR, deleteCacheDirectory } from "../data/Cacher";
import { open } from "@tauri-apps/plugin-shell";
import { appConfigDir, appDataDir, dataDir } from "@tauri-apps/api/path";
import { openTauriImportDialog } from "../data/LibraryUtils";
import { path } from "@tauri-apps/api";

import toast from "svelte-french-toast";

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
                    open(await path.join(dir, CACHE_DIR));
                } catch (err) {
                    console.error(err);
                }
                break;
            case "open-config":
                try {
                    const dir = await appConfigDir();
                    open(dir);
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

interface StemSeparationEvent {
    event: string;
    message: string;
}

export async function listenForStemSeparation() {
    const toastId = toast.loading("Loading...");

    await new Promise(async (resolve, reject) => {
        try {
            const unlisten = await appWindow.listen<StemSeparationEvent>(
                "stem-separation",
                (event) => {
                    switch (event.payload?.event) {
                        case "progress":
                            toast.loading(event.payload.message, {
                                id: toastId,
                            });
                            break;
                        case "complete":
                            toast.success(event.payload.message, {
                                id: toastId,
                            });
                            resolve(event.payload);
                            console.log("event", event);
                            unlisten();
                            break;
                        case "error":
                            toast.error(event.payload.message, {
                                id: toastId,
                            });
                            reject(event.payload);
                            console.log("event", event);
                            unlisten();
                            break;
                    }
                },
            );
        } catch (e) {
            console.error(e);
            reject(e);
        }
    });
}
