import { get } from "svelte/store";
import { userSettings } from "./store";
import { invoke } from "@tauri-apps/api/core";
import type { GetLyricsResponse } from "../App";
import { fetch } from "@tauri-apps/plugin-http";

export async function getLyrics(songTitle: string, artist: string) {
    let result = {
        lyrics: null,
        writers: []
    };
    let geniusPage;

    const settings = get(userSettings);
    try {
        if (!settings.geniusApiKey) {
            throw new Error("API key not set");
        }

        const query = encodeURIComponent(`${songTitle} ${artist}`);

        const geniusResult = await (
            await fetch(`https://api.genius.com/search?q=${query}`, {
                method: "GET",
                headers: {
                    "Accept": "application/json",
                    "Authorization": `Bearer ${settings.geniusApiKey}`
                }
            })
        );
        const geniusData = await geniusResult.json();
        console.log("result", geniusResult);
        if (!geniusResult.ok) {
            throw new Error("Genius API: " + JSON.stringify(geniusResult));
        }
        const hits = geniusData?.response?.hits;
        if (
            hits?.filter(
                (h) =>
                    artist.toLowerCase() ===
                    h?.result?.artist_names?.toLowerCase()
            ).length
        ) {
            geniusPage = hits[0]?.result?.url;
            let songId = hits[0]?.result?.id;

            const songResult = await fetch(
                `https://api.genius.com/songs/${songId}`,
                {
                    method: "GET",
                    headers: {
                        "Accept": "application/json",
                        "Authorization": `Bearer ${settings.geniusApiKey}`
                    }
                }
            );
            result.writers =
                songResult.data?.response?.song?.writer_artists?.map(
                    (w) => w?.name
                );
        }
        console.log("geniusPage", geniusPage);
    } catch (err) {
        console.error("Error fetching from Genius" + err);
        throw new Error("Error: " + err);
    }
    if (geniusPage) {
        const lyricsResult = await invoke<GetLyricsResponse>("get_lyrics", {
            event: {
                url: geniusPage
            }
        });
        result.lyrics = lyricsResult?.lyrics;
    }
    return result;
}
