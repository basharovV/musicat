import type { Album, PlaylistFile, Song } from "src/App";
import {
    current,
    draggedAlbum,
    draggedSongs,
    draggedSource,
    isShuffleEnabled,
    queue,
    queueDuration,
    shuffledQueue,
    type DragSource,
    draggedPlaylist,
} from "./store";
import AudioPlayer from "../lib/player/AudioPlayer";
import { get } from "svelte/store";
import { remove } from "lodash-es";

export function findQueueIndex({ id }: Song): number {
    return get(queue).findIndex((song) => song.id === id);
}

export function findQueueIndexes(songs: Song[]): number[] {
    const q = get(queue);

    return songs.map(({ id }) => q.findIndex((song) => song.id === id));
}

export function removeQueuedSongs(songs: string[]) {
    const olds = get(queue);
    const news = remove(olds, ({ id }) => !songs.includes(id));

    setQueue(news, false);
}

export function resetDraggedSongs() {
    if (get(draggedSongs).length) {
        draggedAlbum.set(null);
        draggedPlaylist.set(null);
        draggedSongs.set([]);
        draggedSource.set(null);
    }
}

export function setDraggedAlbum(
    album: Album,
    songs: Song[],
    source: DragSource,
) {
    draggedAlbum.set(album);
    draggedPlaylist.set(null);
    draggedSongs.set(songs);
    draggedSource.set(source);
}

export function setDraggedPlaylist(
    playlist: PlaylistFile,
    songs: Song[],
    source: DragSource,
) {
    draggedAlbum.set(null);
    draggedPlaylist.set(playlist);
    draggedSongs.set(songs);
    draggedSource.set(source);
}

export function setDraggedSongs(songs: Song[], source: DragSource) {
    draggedAlbum.set(null);
    draggedPlaylist.set(null);
    draggedSongs.set(songs);
    draggedSource.set(source);
}

export function setQueue(
    newQueue: Song[],
    nextSong: Song | number | boolean = null,
): void {
    const newDuration = newQueue.reduce((total, song) => {
        return total + song.fileInfo.duration;
    }, 0);

    if (typeof nextSong === "boolean") {
        AudioPlayer.shouldPlay = nextSong;

        queue.set(newQueue);
        queueDuration.set(newDuration);

        nextSong = null;
    } else if (typeof nextSong === "number") {
        const nextIndex = nextSong;
        nextSong = newQueue[nextIndex];

        if (nextSong) {
            current.set({ song: nextSong, index: nextIndex, position: 0 });
            queue.set(newQueue);
            queueDuration.set(newDuration);
        } else {
            const $current = get(current);
            if ($current.song) {
                // the current song isn't in the queue, set index to -1
                current.set({ ...$current, index: -1 });
            }
            queue.set(newQueue);
            queueDuration.set(newDuration);
        }
    } else if (nextSong) {
        const nextIndex = get(queue).indexOf(nextSong);

        current.set({ song: nextSong, index: nextIndex, position: 0 });
        queue.set(newQueue);
        queueDuration.set(newDuration);
    } else {
        const $current = get(current);
        if ($current.song) {
            // the current song isn't in the queue, set index to -1
            current.set({ ...$current, index: -1 });
        }
        queue.set(newQueue);
        queueDuration.set(newDuration);
    }

    if (nextSong) {
        AudioPlayer.playSong(nextSong as Song);
    }
}

// execute the same `doAction` function on both queues (`queue` and `shuffledQueue`)
export function updateQueues(queueValues, shuffledQueueValues, doAction): void {
    if (get(isShuffleEnabled)) {
        AudioPlayer.shouldShuffle = false;

        const oldShuffledQueue = get(shuffledQueue);
        const newShuffledQueue = doAction(
            oldShuffledQueue,
            shuffledQueueValues,
        );
        shuffledQueue.set(newShuffledQueue || oldShuffledQueue);
    }

    AudioPlayer.shouldPlay = false;

    const oldQueue = get(queue);
    const newQueue = doAction(oldQueue, queueValues);
    queue.set(newQueue || oldQueue);
}
