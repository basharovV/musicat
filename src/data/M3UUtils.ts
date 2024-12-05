import {
    exists,
    mkdir,
    readDir,
    readTextFile,
    rename,
    writeTextFile
} from "@tauri-apps/plugin-fs";
import md5 from "md5";
import { get } from "svelte/store";
import type { PlaylistFile, Song } from "../App";
import { db } from "./db";
import { selectedPlaylistFile, userPlaylists, userSettings } from "./store";
import { moveArrayElement } from "../utils/ArrayUtils";
import { invoke } from "@tauri-apps/api/core";

interface M3UTrack {
    duration: number; // Duration in seconds, -1 if unknown
    title: string; // Track title and artist
    path: string; // Path to the media file (local or URL)
}

interface M3U {
    tracks: M3UTrack[]; // List of tracks
}

function parse(contents: string): M3U {
    const lines = contents.split(/\r?\n/).filter((line) => line.trim() !== "");
    const tracks: M3UTrack[] = [];
    let currentTrack: Partial<M3UTrack> = {};

    for (const line of lines) {
        if (line.trim().startsWith("#EXTINF:")) {
            // Parse EXTINF line
            const [, duration, title] =
                line.match(/#EXTINF:([+-]?\d+(?:\.\d+)?),(.+)/) || [];
            currentTrack = {
                duration: parseInt(duration, 10),
                title: title.trim()
            };
        } else if (!line.trim().startsWith("#")) {
            // Parse media file path
            if (currentTrack) {
                currentTrack.path = line.trim();
                tracks.push(currentTrack as M3UTrack);
                currentTrack = {}; // Reset for the next track
            }
        }
    }

    return { tracks };
}

/**
 * Converts an M3U object into an M3U playlist string.
 * @param m3u The M3U object to convert.
 * @returns The M3U playlist as a string.
 */
function write(m3u: M3U): string {
    const lines: string[] = ["#EXTM3U"];

    for (const track of m3u.tracks) {
        lines.push(`#EXTINF:${track.duration},${track.title}`);
        lines.push(track.path);
    }

    return lines.join("\n");
}

export async function scanPlaylists() {
    console.log("[M3U] Scanning playlists...");
    const playlistsLocation = get(userSettings).playlistsLocation;
    // Read directory (or create if non existent)
    const locationExists = await exists(playlistsLocation);
    if (!locationExists) {
        await mkdir(playlistsLocation);
    }
    const entries = await readDir(playlistsLocation);
    const m3uFiles: PlaylistFile[] = [];
    for (const entry of entries) {
        if (entry.isFile && entry.name.endsWith(".m3u")) {
            m3uFiles.push({
                title: entry.name.split(".m3u")[0],
                path: playlistsLocation + "/" + entry.name
            });
        }
    }
    userPlaylists.set(m3uFiles);
    console.log("[M3U]: Playlists: ", m3uFiles);
}

export async function parsePlaylist(
    playlistFile: PlaylistFile
): Promise<Song[]> {
    const fileContents = await readTextFile(playlistFile.path);
    let playlist: M3U;
    try {
        playlist = parse(fileContents);
    } catch (e) {
        console.error(e);
        return [];
    }

    console.log("M3U: parsed playlist", playlist);

    // Do a db query to get all those songs
    const songIds = playlist.tracks.map((c) => md5(c.path));
    try {
        const songs = await db.songs.bulkGet(songIds);
        console.log("M3U: got songs", songs);
        return songs;
    } catch (e) {
        console.error(e);
        return [];
    }
}

export async function addSongsToPlaylists(
    playlistFile: PlaylistFile,
    songs: Song[]
) {
    const existingSongs = await parsePlaylist(playlistFile);
    await writePlaylist(playlistFile, [...existingSongs, ...songs]);
}

export async function createNewPlaylistFile(title: string) {
    await writePlaylist(
        {
            path: get(userSettings).playlistsLocation + "/" + title + ".m3u",
            title
        },
        []
    );
    await scanPlaylists();
}

export async function renamePlaylist(
    playlistFile: PlaylistFile,
    newTitle: string
) {
    console.log("renamePlaylist", playlistFile, newTitle);
    const newPath = playlistFile.path.replace(playlistFile.title, newTitle);
    console.log(`Renaming ${playlistFile.path} to ${newPath}`);
    await rename(playlistFile.path, newPath);
    await scanPlaylists();
    selectedPlaylistFile.set({
        path: newPath,
        title: newTitle
    });
}

export async function deletePlaylistFile(playlistFile: PlaylistFile) {
    await invoke("delete_files", {
        event: {
            files: [playlistFile.path]
        }
    });
    await scanPlaylists();
}

export async function reorderSongsInPlaylist(
    playlistFile: PlaylistFile,
    fromIdx: number,
    toIdx: number
) {
    let playlist = await parsePlaylist(playlistFile);
    playlist = moveArrayElement(playlist, fromIdx, toIdx);
    await writePlaylist(playlistFile, playlist);
}

export async function deleteSongsFromPlaylist(playlistFile: PlaylistFile, songs: Song[]): Promise<Song[]> {
    const playlist = await parsePlaylist(playlistFile);
    const newPlaylist = playlist.filter((ps) => !songs.find(s => s.id === ps.id));
    await writePlaylist(playlistFile, newPlaylist);
    return parsePlaylist(playlistFile);
}

export async function writePlaylist(playlistFile: PlaylistFile, songs: Song[]) {
    const playlistsLocation = get(userSettings).playlistsLocation;

    const playlist: M3U = {
        tracks: songs.map((s) => ({
            title: `${s.artist} - ${s.title}`,
            path: s.path,
            duration: s.fileInfo.duration
        }))
    };

    const m3u = write(playlist);

    // Write to file using fs, overwrite
    await writeTextFile(playlistFile.path, m3u);
}
