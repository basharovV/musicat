import { open } from "@tauri-apps/plugin-shell";
import type { Song } from "../../App";

export function openInFinder(song: Song) {
    const query = song.path.replace(song.file, "");
    open(query);
}
