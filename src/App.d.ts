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
    viewModel?: {
        isFirstArtist?: boolean;
        isFirstAlbum?: boolean;
        index: number; // When viewed in a song slice, we need the actual index of this song in the list,
        viewId?: string; // Either the song ID, or a playlist id (to allow for duplicates in a keyed each)
        timeSinceAdded?: string; // eg 5m ago
    };
    // Returned from lofty but only written to db for albums for better grid loading performance
    artwork?: {
        data: number[];
        format: string;
        src?: string;
    };
    // User-specific
    playCount: number;
    originCountry?: string;
    songProjectId?: number; // Link to project id
    isFavourite: boolean;
    markers: Marker[];
    dateAdded?: number; // unix timestamp
    tags: string[];
}

interface Album {
    id: string; // Hash of artist + album name
    title: string; // We store the title in lower case for indexed case insensitive searches
    displayTitle?: string; // The display title with actual case
    artist: string;
    year: number;
    genre: string[];
    tracksIds: string[];
    path: string;
    artwork?: ArtworkSrc;
    lossless: boolean;
}

interface Playlist {
    id?: string; // increments automatically
    title: string;
    path: string;
    tracks: string[];
}

interface PlaylistFile {
    path: string;
    title: string; // the filename
}

/**
 * Represents a song/track that's in progress.
 */
interface SongProject {
    songFilepath?: string; // path to the song file containing lyrics and metadata (used for populating this object)
    title: string; // Needs to have at least this, everything else is optional
    artist: string;
    album?: string;
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
    id?: number;
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
    isArtistsToolkitEnabled: boolean;
    scrapbookLocation?: string;
    songbookLocation?: string;
    downloadLocation: string;
    playlistsLocation: string;
    theme: string;
    outputDevice?: string;
    followSystemOutput: boolean;
    geniusApiKey?: string;
    discogsApiKey?: string;
}

interface UIPreferences {
    albumsViewShowSingles: boolean;
    albumsViewShowInfo: boolean;
    albumsViewSortBy: AlbumsSortBy;
    albumsViewGridSize: number;
}

type AlbumsSortBy = "title" | "artist" | "year";

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

type UiView =
    | "library"
    | "favourites"
    | "smart-query"
    | "your-music"
    | "albums"
    | "playlists"
    | "map"
    | "analytics"
    | "internet-archive"
    | "prune" 
    | "to-delete"

type ArtistContentItem = ArtistFileItem | ArtistLinkItem;

interface ContentFileType {
    type: "audio" | "video" | "txt" | "image" | "unsupported";
    extension: string;
}

interface ContentItem {
    id: string; // Hash of filepath
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
    albums: Album[];
    progress: number; // After reaching 100, progress can jump back to 0 and start again (eg. processing tracks then albums)
    done: boolean;
    error?: string;
}

interface ToImportAlbums {
    albums: Album[];
    progress: number;
    done: boolean;
    error?: string;
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

interface GetHTMLResponse {
    html?: string;
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
        trackDescriptions: [{ numberChannels: number }];
        shouldLoop: boolean;
        totalSamples: number;
        loopStartSample: number;
        inputSampleRate: number;
        outputSampleRate: number;
    };
}

interface StreamStatus {
    isOpen: boolean;
}

interface StreamInfo {
    bytesReceived: number;
    receiveRate: number; // in KB/s
    bufferedSamples: number;
    playedSamples: number;
    timestamp: number; // in s
    sampleIdx: number;
}

interface SongChangeEvent {
    songId: string;
}
interface Waveform {
    data: number[];
}

interface WaveformPlayerState {
    data: number[];
    songId: string;
    loopEnabled: boolean;
    loopStartPos: number;
    loopEndPos: number;
    markers: Marker[];
}

interface Marker {
    pos: number; // seconds
    title: string;
}

type PlaylistType = "library" | "album" | "playlist" | "country";

type ArrowFocus = "library" | "queue";

type QueueMode = "library" | "custom";

interface IACollection {
    id: string;
    filesCount: number;
    size: number; // in bytes
    title: string;
    description: string;
}

interface IAFile {
    name: string;
    source: string;
    mtime: string;
    itemId: string;
    duration: number; // in seconds
    previewSrc: string;
    size: string;
    md5: string;
    crc32: string;
    sha1: string;
    format: IAFormat;
    rotation: string;
    matrix_number: string;
    publisher: string;
    creator: string;
    title: string;
    track: string;
    album: string;
    collectionCatalogNumber?: string;
    downloadLocation?: string;
}

type IAFormat = "flac" | "mp3";

interface IAItem {
    id: string;
    title: string;
    server1: string;
    server2: string;
    dir: string;
    files: IAFile[];
    date?: number;
    performer?: string;
    writer?: string;
    original: IAFile;
}

interface AudioDevice {
    name: string;
}

interface AudioDevices {
    devices: AudioDevice[];
    default: AudioDevice;
}

type PopupType = "info" | "track-info" | "settings";
