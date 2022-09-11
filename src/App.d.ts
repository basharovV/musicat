import type { UserQueryPart } from "./lib/smart-query/UserQueryPart";

interface MetadataEntry {
    /** Musicat's cross-file tag identifier */
    genericId: string?;
    /** The original tag id (from Vorbis / IDv3 / etc) */
    id: string;
    /** The tag's value */
    value: string;
}

interface ImportStatus {
    totalTracks: number;
    importedTracks: number;
    isImporting: boolean;
    currentFolder: string;
}

interface Song {
    id: string;
    path: string;
    file: string;
    title: string;
    artist: string;
    album: string;
    year: number;
    genre: string[];
    trackNumber: number;
    duration: string;
    metadata: MetadataEntry[];
    fileInfo: any;
}

type TagType =
    | "vorbis"
    | "ID3v1"
    | "ID3v2.2"
    | "ID3v2.3"
    | "ID3v2.4"
    | "APEv2"
    | "asf"
    | "iTunes"
    | "exif"
    | "matroska";

interface ArtworkSrc {
    src: any;
    format: string;
    size?: {
        width: number;
        height: number;
    };
}

interface UserSettings {
    albumArtworkFilenames: string[];
    miniPlayerLocation: MiniPlayerLocation;
}

type MiniPlayerLocation =
    | "bottom-left"
    | "bottom-right"
    | "top-left"
    | "top-right";

type DataType = "song";

type Comparison =
    | "is-equal"
    | "is-greater-than"
    | "is-less-than"
    | "is-between"
    | "contains";

type SidebarItem = "library" | "smart-query" | "your-music";
