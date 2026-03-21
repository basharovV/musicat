import { get } from "svelte/store";
import { userSettings } from "./store";
import { invoke } from "@tauri-apps/api/core";
import type { GetLyricsResponse, Song, SyncedLyrics } from "../App";
import { fetch } from "@tauri-apps/plugin-http";
import { exists, readTextFile } from "@tauri-apps/plugin-fs";

export async function getLyrics(song: Song) {
    let result = {
        lyrics: null,
        syncedLyrics: null,
        writers: [],
        source: null, // "genius" | "lrclib", "local"
    };

    // First try and find lyrics locally
    // i.e file with the same name as the song in the same folder

    let lrcFilePath = song.path.replace(
        /\.(mp3|wav|flac|m4a|ogg|aac)$/i,
        ".lrc",
    );
    try {
        // Check if file exists
        if (await exists(lrcFilePath)) {
            console.log("lrc file exists");
            const lrcFile = await readTextFile(lrcFilePath);
            result.syncedLyrics = parseLCRLyrics(lrcFile);
            result.source = "local";
        }
    } catch (err) {}

    const songTitle = song.title;
    const artist = song.artist;
    // Otherwise try to fetch synced lyrics
    if (!result.syncedLyrics) {
        const synced = await getSyncedLyrics(songTitle, artist);

        if (synced) {
            result.syncedLyrics = parseLCRLyrics(synced);
            result.source = "lrclib";
        }
    }

    // Otherwise try and fetch lyrics from Genius
    if (!result.syncedLyrics) {
        const { geniusPage, writers } = await getGeniusURLandWriters(
            songTitle,
            artist,
        );
        result.writers = writers;
        if (geniusPage) {
            const lyricsResult = await invoke<GetLyricsResponse>("get_lyrics", {
                event: {
                    url: geniusPage,
                },
            });
            result.lyrics = lyricsResult?.lyrics;
            result.source = "genius";
        }
    }

    return result;
}

export async function getGeniusURLandWriters(
    songTitle: string,
    artist: string,
) {
    // Otherwise try and fetch lyrics from Genius

    let geniusPage;
    let writers = [];

    const settings = get(userSettings);
    try {
        if (!settings.geniusApiKey) {
            throw new Error("API key not set");
        }

        const query = encodeURIComponent(`${songTitle} ${artist}`);

        const geniusResult = await await fetch(
            `https://api.genius.com/search?q=${query}`,
            {
                method: "GET",
                headers: {
                    Accept: "application/json",
                    Authorization: `Bearer ${settings.geniusApiKey}`,
                },
            },
        );
        const geniusData = await geniusResult.json();
        console.log("result", geniusResult);
        if (!geniusResult.ok) {
            throw new Error("Genius API: " + JSON.stringify(geniusResult));
        }
        const hits = geniusData?.response?.hits;
        const hit = hits?.find(
            (h) =>
                artist.toLowerCase() === h?.result?.artist_names?.toLowerCase(),
        );
        if (hit) {
            geniusPage = hit.result?.url;
            let songId = hit.result?.id;

            const songResult = await fetch(
                `https://api.genius.com/songs/${songId}`,
                {
                    method: "GET",
                    headers: {
                        Accept: "application/json",
                        Authorization: `Bearer ${settings.geniusApiKey}`,
                    },
                },
            );
            writers = (
                await songResult.json()
            )?.response?.song?.writer_artists?.map((w) => w?.name);
        }
        console.log("geniusPage", geniusPage);
    } catch (err) {
        console.error("Error fetching from Genius" + err);
        throw new Error("Error: " + err);
    }
    return {
        geniusPage,
        writers,
    };
}

/**
 * Get synced lyrics from LRCLIB API
 */
export async function getSyncedLyrics(songTitle: string, artist: string) {
    let result;
    try {
        const results = await fetch(
            `https://lrclib.net/api/search?track_name=${songTitle}&artist_name=${artist}`,
        );
        const matches = await results.json();
        if (matches.length > 0) {
            result = matches[0]?.syncedLyrics;
        }

        console.log("[synced lyrics] result", result);
    } catch (err) {
        console.error("Error fetching lyrics" + err);
    }
    return result;
}

/**
 * Parse LCRLIB lyrics into a @type {SyncedLyrics} format
 * @param lyrics Lyrics string
 */
function parseLCRLyrics(lyrics: string) {
    const lines = lyrics.split("\n");
    console.log("lines", lines);
    let syncedLyrics: SyncedLyrics[] = [];
    // eg. [00:02:59] May I have your attention please
    const regex = /\[(\d{2}:\d{2}.\d{2})\](.*)/;
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const match = line.match(regex);
        if (match) {
            const time = match[1];
            const text = match[2].trim();
            // Convert the time (minutes, seconds, millis) to seconds
            const [minutes, seconds] = time.split(":");
            const timestamp = parseInt(minutes) * 60 + parseFloat(seconds);
            syncedLyrics.push({ timestamp, lyricLine: text });
        }
    }
    return syncedLyrics;
}
