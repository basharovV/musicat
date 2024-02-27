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
    currentSong: string;
    backgroundImport: boolean;
    status: string;
    percent: number;
}

interface BottomBarNotification {
    text?: string;
    timeout?: number;
}

interface FileInfo {
    duration: number;
    overallBitrate: number;
    audioBitrate: number;
    sampleRate: number;
    bitDepth: number;
    channels: number;
    lossless: boolean;
    tagType: string;
    codec: string;
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
    trackNumber: number;
    duration: string;
    metadata: MetadataEntry[];
    fileInfo: FileInfo;
    originCountry?: string;
    songProjectId?: number; // Link to project id
    isFavourite: boolean;
    viewModel?: {
        isFirstArtist: boolean;
        isFirstAlbum: boolean;
        index: number; // When viewed in a song slice, we need the actual index of this song in the list,
        viewId: string; // Either the song ID, or a playlist id (to allow for duplicates in a keyed each)
    };
    playCount: number;
    // Returned from lofty but only written to db for albums for better grid loading performance
    artwork?: {
        data: number[];
        format: string;
    };
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
    lossless: boolean;
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
    geniusApiKey?: string;
    isArtistsToolkitEnabled: boolean;
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
    | "favourites"
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
    numberOfArtists: number;
    artists: string[]; // first few,
    albums: { path: string; album: string; artist: string }[]; // first few as well
}

interface ToImport {
    songs: Song[];
    progress: number;
}
interface ReadMetadataStatus {
    progress: number;
}

type ArtworkFolderFilename = "folder.jpg" | "cover.jpg" | "artwork.jpg";

interface LookForArtResult {
    artworkSrc?: string;
    artworkFormat: string;
    artworkFilenameMatch: string;
}

interface ActionEvent {
    target: string;
    action: "focus" | "unfocus";
}

type LLM = "gpt-3.5-turbo" | "gpt-4" | "ollama";

type Compression = "lossy" | "lossless" | "both";

interface LastPlayedInfo {
    songId?: string;
    position: number; //seconds;
}

interface AddOriginCountryStatus {
    percent: number;
}

interface GetLyricsResponse {
    lyrics?: string;
}

interface CurrentSongLyrics {
    songId: string;
    lyrics?: string;
    writers?: string[];
}

interface GetFileSizeResponse {
    fileSize: nuumber;
}

interface AudioSourceNodeOptions {
    numberOfInputs: number;
    numberOfOutputs: number;
    outputChannelCount: number;
    processorOptions: {
        initialSamples: Float32Array[][];
        trackStates: Array<boolean>;
        trackDescriptions: [{numberChannels: number}];
        shouldLoop: boolean;
        totalSamples: number;
        loopStartSample: number;
        inputSampleRate: number;
        outputSampleRate: number;
    }
}


interface StreamStatus {
    isOpen: boolean;
}

interface StreamInfo {
    bytesReceived: number,
    receiveRate: number, // in KB/s
    bufferedSamples: number,
    playedSamples: number,
    timestamp: number // in s
    
}