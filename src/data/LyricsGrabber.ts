import { get } from "svelte/store";
import { userSettings } from "./store";
import { invoke } from "@tauri-apps/api/core";
import type { GetLyricsResponse, SyncedLyrics } from "../App";
import { fetch } from "@tauri-apps/plugin-http";

export async function getLyrics(songTitle: string, artist: string) {
    let result = {
        lyrics: null,
        syncedLyrics: null,
        writers: [],
    };

    // First try to fetch synced lyrics
    // (and later still use Genius to fetch writers)
    const synced = await getSyncedLyrics(songTitle, artist);

    if (synced) {
        result.syncedLyrics = parseLCRLyrics(synced);
        console.log("synced lyrics", result.syncedLyrics);
    }

    let geniusPage;

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
            result.writers = (
                await songResult.json()
            )?.response?.song?.writer_artists?.map((w) => w?.name);
        }
        console.log("geniusPage", geniusPage);
    } catch (err) {
        console.error("Error fetching from Genius" + err);
        throw new Error("Error: " + err);
    }
    if (!result.syncedLyrics && geniusPage) {
        const lyricsResult = await invoke<GetLyricsResponse>("get_lyrics", {
            event: {
                url: geniusPage,
            },
        });
        result.lyrics = lyricsResult?.lyrics;
    }
    return result;
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
