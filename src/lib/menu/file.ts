import { openPath } from "@tauri-apps/plugin-opener";
import type { Song } from "../../App";

export function openInFinder(song: Song) {
    const query = song.path.replace(song.file, "");
    openPath(query);
}
