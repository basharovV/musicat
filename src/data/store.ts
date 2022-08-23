import { writable, type Writable } from "svelte/store";

interface Query {
  orderBy: string;
  reverse: boolean;
  query: string;
}

export const query: Writable<Query> = writable({
  orderBy: "artist",
  reverse: false,
  query: ''
});



export const isPlaying = writable(false);
export const currentSong: Writable<Song> = writable(null);
export const currentSongIdx = writable(0);
export const isDraggingFiles = writable(false);
export const queriedSongs: Writable<Song[]> = writable([]);
export const songsJustAdded: Writable<Song[]> = writable([]);
export const songJustAdded = writable(false);
export const rightClickedTrack: Writable<Song> = writable(null);
export const playerTime = writable(0);
export const seekTime = writable(0);
export const volume = writable(1);
export const isInfoPopupOpen = writable(false);
