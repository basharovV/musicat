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
    backgroundImport: boolean;
}

interface Song {
    /**
     * A hash of the filepath
     */
    id: string;
    path: string;
    file: string;
    title: string;
    artist: string;
    album: string;
    year: number;
    genre: string[];
    composer: string[];
    originCountry?: string;
    trackNumber: number;
    duration: string;
    metadata: MetadataEntry[];
    fileInfo: any;
    songProjectId?: number; // Link to project id
    isFavourite: boolean;
    viewModel?: {
        isFirstArtist: boolean;
        isFirstAlbum: boolean;
    },
    playCount: number
}

interface Album {
    id: string; // Hash of artist + album name
    title: string;
    artist: string;
    year: number;
    genre: string[];
    trackCount: number;
    tracksIds: string[];
    duration: string;
    path: string;
    artwork?: ArtworkSrc;
}

interface Playlist {
    id?: number; // increments automatically
    title: string;
    tracks: string[];
}

/**
 * Represents a song/track that's in progress.
 */
interface SongProject {
    id?: number; // incremental id
    title: string; // Needs to have at least this, everything else is optional
    artist: string;
    album: string;
    key?: string;
    bpm?: number;
    musicComposedBy: string[]; // Multiple people
    lyricsWrittenBy: string[]; // Multiple people
    artworkFilePath?: string;
    lyrics?: string;
    recordings: SongProjectRecording[];
    otherContentItems: ArtistContentItem[];
    songId?: string; // If this exists - the song is added to the library (master audio)
}

interface ArtistProject {
    name: string;
    members: string[];
    profilePhoto?: string;
}

type RecordingType = "master" | "live" | "demo" | "rehearsal";

interface SongProjectRecording {
    recordingType: RecordingType;
    song: Song;
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
    foldersToWatch: string[];
    albumArtworkFilenames: string[];
    miniPlayerLocation: MiniPlayerLocation;
    aiFeaturesEnabled: boolean;
    llm: LLM;
    openAIApiKey?: string;
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

type SidebarItem =
    | "library"
    | "smart-query"
    | "your-music"
    | "albums"
    | "playlists"
    | "map"
    | "analytics";

type ArtistContentItem = ArtistFileItem | ArtistLinkItem;

interface ContentFileType {
    type: "audio" | "video" | "txt" | "image" | "unsupported";
    extension: string;
}

interface ContentItem {
    id?: number;
    type: "file" | "link";
    name: string; // Grab from filename
    tags: string[];
    songId?: string; // Could just use "artist-title"
}

export interface ArtistLinkItem extends ContentItem {
    type: "link";
    url: string;
    imageUrl?: string;
}

export interface ArtistFileItem extends ContentItem {
    type: "file";
    fileType: ContentFileType;
    path?: string;
}


export interface MenuItem {
    text: string;
    description?: string;
    source?: any;
    color?: string;
}

export interface MenuSection {
    items: MenuItem[];
    title: string;
}


interface MapTooltipData {
    countryName: string;
    emoji: string;
    numberOfArtists: number,
    artists: string[], // first few,
    albums: {album: string, artist: string}[] // first few as well
}


type LLM =
    | "gpt-3.5-turbo"
    | "gpt-4"
    | "ollama"
