import type { Album } from "../../App";
import { db } from "../../data/db";

export const HEADER_HEIGHT = 22;
export const PADDING = 14;
export const ROW_HEIGHT = 26;

export async function getAlbumDetailsHeight(album: Album): Promise<number> {
    var tracks = await db.songs
        .where("id")
        .anyOf(album.tracksIds)
        .toArray();

    tracks.sort((a, b) => {
        return a.trackNumber - b.trackNumber;
    });

    const height = PADDING + 300 + HEADER_HEIGHT + tracks.length * ROW_HEIGHT + 8 + PADDING;

    return height;
}
