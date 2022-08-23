import type { Event } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";
import { db } from "../data/db";

export function startMenuListener() {
  appWindow.listen("menu", ({ event, payload }) => {
    switch (payload) {
      case "clear-db":
        console.log("clear-db");
        db.songs.clear();
        break;
    }
  });
}

export async function listenForFileDrop(): Promise<Event<any>> {
  return new Promise(async (resolve, reject) => {
    try {
    const unlisten = await appWindow.listen("tauri://file-drop", (event) => {
      console.log("event", event);
      resolve(event);
      unlisten();
    });
  } catch(e) {
    console.error(e);
    reject(e);
  }
  });
}
