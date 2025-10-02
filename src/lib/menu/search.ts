import { openUrl } from "@tauri-apps/plugin-opener";
import type { Song } from "../../App";
import { isWikiOpen, wikiArtist } from "../../data/store";

export function searchArtistOnWikiPanel(song: Song) {
    wikiArtist.set(song.artist);
    isWikiOpen.set(true);
}

export function searchArtistOnWikipedia(song: Song) {
    const query = encodeURIComponent(song.artist);
    openUrl(`https://en.wikipedia.org/wiki/${query}`);
}

export function searchArtistOnYouTube(song: Song) {
    const query = encodeURIComponent(song.artist);
    openUrl(`https://www.youtube.com/results?search_query=${query}`);
}

export function searchArtworkOnBrave(data: { title: string; artist: string }) {
    const query = encodeURIComponent(`${data.artist} - ${data.title}`);
    openUrl(`https://search.brave.com/images?q=${query}`);
}

export function searchChords(song: Song) {
    const query = encodeURIComponent(
        song.artist + " " + song.title + " chords",
    );
    openUrl(`https://duckduckgo.com/?q=${query}`);
}

export function searchLyrics(song: Song) {
    const query = encodeURIComponent(
        song.artist + " " + song.title + " lyrics",
    );
    openUrl(`https://duckduckgo.com/?q=${query}`);
}

export function searchSongOnYouTube(song: Song) {
    const query = encodeURIComponent(song.title);
    openUrl(`https://www.youtube.com/results?search_query=${query}`);
}
