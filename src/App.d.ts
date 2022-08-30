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
