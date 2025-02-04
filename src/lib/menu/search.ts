import { open } from "@tauri-apps/plugin-shell";
import type { Song } from "../../App";
import { isWikiOpen, wikiArtist } from "../../data/store";

export function searchArtistOnWikiPanel(song: Song) {
    wikiArtist.set(song.artist);
    isWikiOpen.set(true);
}

export function searchArtistOnWikipedia(song: Song) {
    const query = encodeURIComponent(song.artist);
    open(`https://en.wikipedia.org/wiki/${query}`);
}

export function searchArtistOnYouTube(song: Song) {
    const query = encodeURIComponent(song.artist);
    open(`https://www.youtube.com/results?search_query=${query}`);
}

export function searchArtworkOnBrave(data: { title: string; artist: string }) {
    const query = encodeURIComponent(`${data.artist} - ${data.title}`);
    open(`https://search.brave.com/images?q=${query}`);
}

export function searchChords(song: Song) {
    const query = encodeURIComponent(
        song.artist + " " + song.title + " chords",
    );
    open(`https://duckduckgo.com/?q=${query}`);
}

export function searchLyrics(song: Song) {
    const query = encodeURIComponent(
        song.artist + " " + song.title + " lyrics",
    );
    open(`https://duckduckgo.com/?q=${query}`);
}

export function searchSongOnYouTube(song: Song) {
    const query = encodeURIComponent(song.title);
    open(`https://www.youtube.com/results?search_query=${query}`);
}
