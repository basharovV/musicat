import type { TagType } from "src/App";

function inverse(obj) {
    var retobj = {};
    for (var key in obj) {
        retobj[obj[key]] = key;
    }
    return retobj;
}

interface TagFieldMap {
    title: string;
    artist: string;
    album: string;
    albumArtist?: string;
    composer?: string;
    performer?: string;
    genre: string;
    date: string;
    copyright?: string;
    publisher?: string;
    trackNumber: string;
    license?: string;
    location?: string;
    isrc?: string;
    bpm?: string;
}

const genericToVorbisMap: TagFieldMap = {
    title: "TITLE",
    artist: "ARTIST",
    album: "ALBUM",
    albumArtist: "ALBUMARTIST",
    composer: "COMPOSER",
    genre: "GENRE",
    date: "DATE",
    trackNumber: "TRACKNUMBER",
    copyright: "COPYRIGHT",
    publisher: "PUBLISHER",
    performer: "PERFORMER",
    license: "LICENSE",
    location: "LOCATION",
    isrc: "ISRC",
    bpm: "BPM"
};

const vorbisToGenericMap = inverse(genericToVorbisMap);

/*
ID3v1
-------

*/
const genericToId3v1Map: TagFieldMap = {
    title: "title",
    artist: "artist",
    album: "album",
    genre: "genre",
    date: "year",
    trackNumber: "track"
};

const id3v1ToGenericMap = inverse(genericToId3v1Map);

/*
ID3v2.2
-------
Three character ids
*/
const genericToId3v22Map: TagFieldMap = {
    title: "TT2",
    artist: "TPE1",
    album: "TAL",
    albumArtist: "TP2",
    composer: "TCM",
    genre: "TCO",
    date: "TYE",
    trackNumber: "TRK",
    copyright: "TCR",
    publisher: "TPB",
    isrc: "TRC",
    bpm: "TBP"
};

const id3v22ToGenericMap = inverse(genericToId3v22Map);

const genericToId3v23Map: TagFieldMap = {
    title: "TIT2",
    artist: "TPE1",
    album: "TALB",
    albumArtist: "TPE2",
    composer: "TCOM",
    genre: "TCON",
    date: "TDAT",
    trackNumber: "TRCK",
    copyright: "TCOP",
    publisher: "TPUB",
    isrc: "TSRC",
    bpm: "TBPM"
};

const id3v23ToGenericMap = inverse(genericToId3v23Map);

const genericToId3v24Map: TagFieldMap = {
    title: "TIT2",
    artist: "TPE1",
    album: "TALB",
    albumArtist: "TPE2",
    composer: "TCOM",
    genre: "TCON",
    date: "TDRC",
    trackNumber: "TRCK",
    copyright: "TCOP",
    publisher: "TPUB",
    isrc: "TSRC",
    bpm: "TBPM"
};

const id3v24ToGenericMap = inverse(genericToId3v24Map);

function getMapForTagType(
    tagType: TagType,
    fromGeneric: boolean = true
): object | null {
    switch (tagType) {
        case "vorbis":
            return fromGeneric ? genericToVorbisMap : vorbisToGenericMap;
        case "ID3v1":
            return fromGeneric ? genericToId3v1Map : id3v1ToGenericMap;
        case "ID3v2.2":
            return fromGeneric ? genericToId3v22Map : id3v22ToGenericMap;
        case "ID3v2.3":
            return fromGeneric ? genericToId3v23Map : id3v23ToGenericMap;
        case "ID3v2.4":
            return fromGeneric ? genericToId3v24Map : id3v24ToGenericMap;
        default:
            return null;
    }
}

/**
 * If we don't know the tagType from metadata, or the metadata is empty,
 * we can infer the type from the file codec.
 * @param codec
 */
const codecToTagTypeMap = {
    "FLAC": "Vorbis",
    "MPEG": "ID3v2.4",
    "MPEG 1 Layer 3": "ID3v2.4"
};

function getTagTypeFromCodec(codec) {
    if (!codec) return null;
    return codecToTagTypeMap[codec];
}

export { getMapForTagType, getTagTypeFromCodec };
