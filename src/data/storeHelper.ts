import type { Song } from "src/App";
import { current, isShuffleEnabled, queue, queueDuration, shuffledQueue } from "./store";
import AudioPlayer from "../lib/player/AudioPlayer";
import { get } from "svelte/store";

export function findQueueIndex({id}: Song): number {
    return get(queue).findIndex((song) => song.id === id);
}

export function findQueueIndexes(songs: Song[]): number[] {
    const q = get(queue);

    return songs.map(({id}) => q.findIndex((song) => song.id === id));
}

export function setQueue(newQueue: Song[], nextSong: Song | number | boolean = null): void {
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
            current.set({ song: null, index: 0, position: 0 });
            queue.set(newQueue);
            queueDuration.set(newDuration);
        }
    } else if (nextSong) {
        const nextIndex = get(queue).indexOf(nextSong);

        if (nextIndex !== -1) {
            current.set({ song: nextSong, index: nextIndex, position: 0 });
            queue.set(newQueue);
        } else {
            current.set({ song: null, index: 0, position: 0 });
            queue.set(newQueue);
            queueDuration.set(newDuration);

            nextSong = null;
        }
    } else {
        current.set({ song: null, index: 0, position: 0 });
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
        const newShuffledQueue = doAction(oldShuffledQueue, shuffledQueueValues)
        shuffledQueue.set(newShuffledQueue || oldShuffledQueue);
    }

    AudioPlayer.shouldPlay = false;

    const oldQueue = get(queue);
    const newQueue = doAction(oldQueue, queueValues)
    queue.set(newQueue || oldQueue);
}
