import { invoke } from "@tauri-apps/api/core";
import type { Album, Song } from "../App";

/**
 * TS mirror of `db::query`'s `QuerySpec`/`FilterNode`/`Page<T>` (Rust) --
 * the generic filter/sort/pagination request every query command shares.
 */
export type CmpOp = "eq" | "ne" | "gt" | "gte" | "lt" | "lte";

export type FilterNode =
    | { kind: "and"; nodes: FilterNode[] }
    | { kind: "or"; nodes: FilterNode[] }
    | { kind: "cmp"; field: string; op: CmpOp; value: string | number | boolean }
    | {
          kind: "between";
          field: string;
          min: string | number | boolean;
          max: string | number | boolean;
      }
    | { kind: "contains"; field: string; value: string }
    | { kind: "notEmpty"; field: string };

export interface SortKey {
    field: string;
    descending?: boolean;
}

export interface PageRequest {
    cursor?: string | null;
    offset?: number;
    limit?: number;
}

export interface QuerySpec {
    filter?: FilterNode | null;
    sort?: SortKey | null;
    page?: PageRequest;
}

export interface Page<T> {
    items: T[];
    nextCursor: string | null;
    hasMore: boolean;
}

/**
 * The value type each whitelisted track field expects (mirrors
 * `TRACK_FIELDS` in `db::query`) -- used to coerce a smart-query part's raw
 * (always-string-from-input) value into the right JSON type before it's sent
 * over, since Rust's whitelist rejects a type mismatch rather than
 * coercing it.
 */
export const TRACK_FIELD_KIND: Record<string, "text" | "int" | "real"> = {
    title: "text",
    artist: "text",
    album: "text",
    albumArtist: "text",
    composer: "text",
    genre: "text",
    originCountry: "text",
    tags: "text",
    year: "int",
    trackNumber: "int",
    dateAdded: "int",
    playCount: "int",
    duration: "real",
};

const MAX_PAGE_SIZE = 1000;

/**
 * Runs `query_tracks`, following `nextCursor` until every matching row has
 * been fetched. `query_tracks` pages (1000 rows max per call) rather than
 * returning everything at once, but existing UI (library table, smart query
 * results) still expects a single fully-loaded array -- introducing real
 * paginated/virtualized fetching in the UI is separate follow-up work, not
 * part of swapping the storage backend.
 */
export async function queryAllTracks(spec: QuerySpec): Promise<Song[]> {
    const results: Song[] = [];
    let cursor: string | null | undefined = spec.page?.cursor ?? null;
    while (true) {
        const page = await invoke<Page<Song>>("query_tracks", {
            spec: { ...spec, page: { ...spec.page, cursor, limit: MAX_PAGE_SIZE } },
        });
        results.push(...page.items);
        if (!page.hasMore || !page.nextCursor) {
            break;
        }
        cursor = page.nextCursor;
    }
    return results;
}

/** Same idea as `queryAllTracks`, but for `query_albums` (offset-paginated). */
export async function queryAllAlbums(spec: QuerySpec): Promise<Album[]> {
    const results: Album[] = [];
    let offset = spec.page?.offset ?? 0;
    while (true) {
        const page = await invoke<Page<Album>>("query_albums", {
            spec: { ...spec, page: { ...spec.page, offset, limit: MAX_PAGE_SIZE } },
        });
        results.push(...page.items);
        if (!page.hasMore) {
            break;
        }
        offset += page.items.length;
    }
    return results;
}

/**
 * Tracks belonging to an album, in track order. `Album.tracksIds` isn't
 * populated from SQLite (see `db::models::row_to_album` -- `tracks.albumId`
 * is the source of truth for this relationship now), so this queries by
 * that instead of reading a denormalized id list off the album.
 */
export async function queryAlbumTracks(albumId: string): Promise<Song[]> {
    return queryAllTracks({
        filter: { kind: "cmp", field: "albumId", op: "eq", value: albumId },
        sort: { field: "trackNumber" },
    });
}
