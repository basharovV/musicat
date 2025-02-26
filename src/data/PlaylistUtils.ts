import {
    exists,
    mkdir,
    readDir,
    readTextFile,
    rename,
    writeTextFile,
} from "@tauri-apps/plugin-fs";
import md5 from "md5";
import { get } from "svelte/store";
import type { StaticPlaylistFile, Song, DynamicPlaylistFile } from "../App";
import { db } from "./db";
import {
    selectedPlaylistFile,
    userDynamicPlaylists,
    userStaticPlaylists,
    userSettings,
} from "./store";
import { moveArrayElement } from "../utils/ArrayUtils";
import { invoke } from "@tauri-apps/api/core";
import { path } from "@tauri-apps/api";
import YAML from "yaml";
import Ajv from "ajv";
import dynoPLSchema from "@zokugun/dynopl/lib/dynopl.schema.json";
import type { Playlist as DynamicPlaylist } from "@zokugun/dynopl";
import SmartQuery from "../lib/smart-query/Query";
import { kebabCase } from "lodash-es";

interface M3UTrack {
    duration: number; // Duration in seconds, -1 if unknown
    title: string; // Track title and artist
    path: string; // Path to the media file (local or URL)
}

interface M3U {
    tracks: M3UTrack[]; // List of tracks
}

const VALID_DYNOPL_OPERATORS = {
    contains: ["genre", "tags", "title"],
    inTheRange: ["year"],
    is: ["artist", "albumArtist", "composer", "originCountry", "year"],
    gt: ["duration", "year"],
};

const AJV = new Ajv({
    allErrors: true,
    allowUnionTypes: true,
    useDefaults: true,
    removeAdditional: true,
});
const validateDynoPLSchema = AJV.compile<DynamicPlaylist>(dynoPLSchema);

const NUMBER_FIELDS = ["duration", "trackNumber", "compilation", "year"];

export async function addSongsToStaticPlaylist(
    playlistFile: StaticPlaylistFile,
    songs: Song[],
) {
    const existingSongs = await loadStaticPlaylist(playlistFile);
    await writeStaticPlaylist(playlistFile, [...existingSongs, ...songs]);
}

export function composeComparator(
    key: string,
    descending: boolean,
    eqComparator,
) {
    if (NUMBER_FIELDS.includes(key)) {
        return (a, b) => {
            const d = a[key] - b[key];

            if (d === 0) {
                return eqComparator(a, b);
            } else if (descending) {
                return -d;
            } else {
                return d;
            }
        };
    } else {
        return (a, b) => {
            const d = a[key].localeCompare(b[key]);

            if (d === 0) {
                return eqComparator(a, b);
            } else if (descending) {
                return -d;
            } else {
                return d;
            }
        };
    }
}

export async function createNewStaticPlaylistFile(
    title: string,
    songs: Song[] = [],
) {
    await writeStaticPlaylist(
        {
            path: await path.join(
                get(userSettings).playlistsLocation,
                `${title}.m3u`,
            ),
            title,
        },
        songs,
    );
    await scanPlaylists();
}

export async function deletePlaylistFile(playlistFile: StaticPlaylistFile) {
    await invoke("delete_files", {
        event: {
            files: [playlistFile.path],
        },
    });
    await scanPlaylists();
}

export async function deleteSongsFromStaticPlaylist(
    playlistFile: StaticPlaylistFile,
    songs: Song[],
): Promise<Song[]> {
    const playlist = await loadStaticPlaylist(playlistFile);
    const newPlaylist = playlist.filter(
        (ps) => !songs.find((s) => s.id === ps.id),
    );
    await writeStaticPlaylist(playlistFile, newPlaylist);
    return loadStaticPlaylist(playlistFile);
}

export function getComparator(key: string, descending: boolean) {
    if (NUMBER_FIELDS.includes(key)) {
        if (descending) {
            return (a, b) => b[key] - a[key];
        } else {
            return (a, b) => a[key] - b[key];
        }
    } else {
        if (descending) {
            return (a, b) => b[key].localeCompare(a[key]);
        } else {
            return (a, b) => a[key].localeCompare(b[key]);
        }
    }
}

export async function insertSongsToStaticPlaylist(
    playlistFile: StaticPlaylistFile,
    songs: Song[],
    index: number,
) {
    const playlist = await loadStaticPlaylist(playlistFile);
    const newPlaylist = [...playlist];
    newPlaylist.splice(index, 0, ...songs);
    await writeStaticPlaylist(playlistFile, newPlaylist);
}

export async function loadDynamicPlaylist(
    filepath: string,
): Promise<DynamicPlaylistFile | null> {
    const ext = await path.extname(filepath);

    if (ext === "json" || ext === "jdp") {
        const content = await readTextFile(filepath);
        const parsed = JSON.parse(content);

        if (validateDynoPL(parsed)) {
            let title = parsed.name;
            if (typeof title !== "string" || title.length === 0) {
                title = delExt(await path.basename(filepath));

                if (ext === "json") {
                    title = delExt(title);
                }
            }

            return {
                title,
                path: filepath,
                schema: parsed,
            };
        }
    } else if (ext === "yaml" || ext === "yml" || ext === "ydp") {
        const content = await readTextFile(filepath);
        const parsed = YAML.parse(content);

        if (validateDynoPL(parsed)) {
            let title = parsed.name;
            if (typeof title !== "string" || title.length === 0) {
                title = delExt(await path.basename(filepath));

                if (ext === "yaml" || ext === "yml") {
                    title = delExt(title);
                }
            }

            return {
                title,
                path: filepath,
                schema: parsed,
            };
        }
    }

    return null;
}

export async function loadStaticPlaylist(
    playlistFile: StaticPlaylistFile,
): Promise<Song[]> {
    const fileContents = await readTextFile(playlistFile.path);
    let playlist: M3U;
    try {
        playlist = parseStaticPlaylist(fileContents);
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

export async function renameStaticPlaylist(
    playlistFile: StaticPlaylistFile,
    newTitle: string,
) {
    console.log("renameStaticPlaylist", playlistFile, newTitle);
    const newPath = playlistFile.path.replace(playlistFile.title, newTitle);
    console.log(`Renaming ${playlistFile.path} to ${newPath}`);
    await rename(playlistFile.path, newPath);
    await scanPlaylists();
    selectedPlaylistFile.set({
        path: newPath,
        title: newTitle,
    });
}

export async function reorderSongsInStaticPlaylist(
    playlistFile: StaticPlaylistFile,
    fromIdx: number,
    toIdx: number,
) {
    let playlist = await loadStaticPlaylist(playlistFile);
    playlist = moveArrayElement(playlist, fromIdx, toIdx);
    await writeStaticPlaylist(playlistFile, playlist);
}

export async function scanPlaylists() {
    console.log("[M3U] Scanning playlists...");
    const playlistsLocation = get(userSettings).playlistsLocation;
    // Read directory (or create if non existent)
    const locationExists = await exists(playlistsLocation);
    if (!locationExists) {
        await mkdir(playlistsLocation);
    }

    for (const smartQuery of await db.smartQueries.toArray()) {
        const name = kebabCase(smartQuery.name);
        const filepath = await path.join(
            playlistsLocation,
            `${name}.dynopl.yaml`,
        );
        const schema = SmartQuery.toDynoPL(smartQuery);
        const playlist: DynamicPlaylistFile = {
            title: smartQuery.name,
            path: filepath,
            schema,
        };

        await writeDynamicPlaylist(playlist);

        await db.smartQueries.delete(smartQuery.id);
    }

    const dynamicFiles: DynamicPlaylistFile[] = [];
    const staticFiles: StaticPlaylistFile[] = [];
    const entries = await readDir(playlistsLocation);

    for (const entry of entries) {
        if (entry.isFile) {
            if (entry.name.endsWith(".m3u")) {
                staticFiles.push({
                    title: delExt(entry.name),
                    path: await path.join(playlistsLocation, entry.name),
                });
            } else if (
                entry.name.endsWith(".dynopl.yaml") ||
                entry.name.endsWith(".dynopl.yml") ||
                entry.name.endsWith(".dynopl.json") ||
                entry.name.endsWith(".ydp") ||
                entry.name.endsWith(".jdp")
            ) {
                const playlist = await loadDynamicPlaylist(
                    await path.join(playlistsLocation, entry.name),
                );

                if (playlist) {
                    dynamicFiles.push(playlist);
                }
            }
        }
    }

    userDynamicPlaylists.set(
        dynamicFiles.sort((a, b) => a.title.localeCompare(b.title)),
    );
    userStaticPlaylists.set(
        staticFiles.sort((a, b) => a.title.localeCompare(b.title)),
    );

    console.log("[DynoPL]: Playlists: ", dynamicFiles);
    console.log("[M3U]: Playlists: ", staticFiles);
}

export async function writeDynamicPlaylist(playlistFile: DynamicPlaylistFile) {
    playlistFile.schema.name = playlistFile.title;

    let content = null;

    const ext = await path.extname(playlistFile.path);

    if (ext === "json" || ext === "jdp") {
        content = JSON.stringify(playlistFile.schema, null, "\t");
    } else if (ext === "yaml" || ext === "yml" || ext === "ydp") {
        content = YAML.stringify(playlistFile.schema);
    }

    if (content) {
        // Write to file using fs, overwrite
        await writeTextFile(playlistFile.path, content);
    }
}

export async function writeStaticPlaylist(
    playlistFile: StaticPlaylistFile,
    songs: Song[],
) {
    const playlist: M3U = {
        tracks: songs.map((s) => ({
            title: `${s.artist} - ${s.title}`,
            path: s.path,
            duration: s.fileInfo.duration,
        })),
    };

    const m3u = toStaticPlaylist(playlist);

    // Write to file using fs, overwrite
    await writeTextFile(playlistFile.path, m3u);
}

function delExt(filename: string): string {
    const index = filename.lastIndexOf(".");
    if (index > 0) {
        return filename.substring(0, index);
    } else {
        return filename;
    }
}

function parseStaticPlaylist(contents: string): M3U {
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
                title: title.trim(),
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
function toStaticPlaylist(m3u: M3U): string {
    const lines: string[] = ["#EXTM3U"];

    for (const track of m3u.tracks) {
        lines.push(`#EXTINF:${track.duration},${track.title}`);
        lines.push(track.path);
    }

    return lines.join("\n");
}

function validateDynoPL(data): boolean {
    // if (!validateDynoPLSchema(data)) {
    //     console.log(validateDynoPLSchema.errors);
    //     return false;
    // }

    if (!data.all) {
        return false;
    }

    for (var rule of data.all) {
        const operator = Object.keys(rule)[0];
        const keys = VALID_DYNOPL_OPERATORS[operator];

        if (!keys) {
            return false;
        }

        for (const key of Object.keys(rule[operator])) {
            if (!keys.includes(key)) {
                return false;
            }
        }
    }

    return true;
}
