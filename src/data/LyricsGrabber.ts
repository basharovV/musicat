import { get } from "svelte/store";
import { userSettings } from "./store";
import { invoke } from "@tauri-apps/api";
import type { GetLyricsResponse } from "../App";
import { fetch } from "@tauri-apps/api/http";

export async function getLyrics(songTitle: string, artist: string) {
    let geniusPage;
    const settings = get(userSettings);
    try {
        if (!settings.geniusApiKey) {
            throw new Error("API key not set");
        }

        const query = encodeURIComponent(`${songTitle} ${artist}`);

        const geniusResult = await fetch(
            `https://api.genius.com/search?q=${query}`,
            {
                method: "GET",
                headers: {
                    "Accept": "application/json",
                    "Authorization": `Bearer ${settings.geniusApiKey}`
                }
            }
        );
        if (!geniusResult.ok) {
            throw new Error("Genius API: " + JSON.stringify(geniusResult.data))
        }
        const hits = geniusResult.data?.response?.hits;
        if (hits?.length) {
            geniusPage = hits[0]?.result?.url;
        }
        console.log("geniusPage", geniusPage);
    } catch (err) {
        console.error("Error fetching from Genius" + err);
        throw new Error("Error: " + err);
    }
    if (geniusPage) {
        const result = await invoke<GetLyricsResponse>("get_lyrics", {
            event: {
                url: geniusPage
            }
        });
        return result?.lyrics;
    }
    return null;
}
