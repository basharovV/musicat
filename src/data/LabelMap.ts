import type { TagType } from "src/App";

function inverse(obj) {
    const retobj = {};
    for (const key in obj) {
        const value = obj[key];
        if (Array.isArray(value)) {
            for (const val of value) {
                if (retobj[val]) {
                    if (Array.isArray(retobj[val])) {
                        retobj[val].push(key);
                    } else {
                        retobj[val] = [retobj[val], key];
                    }
                } else {
                    retobj[val] = key;
                }
            }
        } else {
            if (retobj[value]) {
                if (Array.isArray(retobj[value])) {
                    retobj[value].push(key);
                } else {
                    retobj[value] = [retobj[value], key];
                }
            } else {
                retobj[value] = key;
            }
        }
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
    year: string;
    copyright?: string;
    publisher?: string;
    trackNumber: string;
    trackTotal?: string[];
    license?: string;
    isrc?: string;
    bpm?: string;
    compilation?: string;
    discNumber?: string;
    discTotal?: string[];
    encodingTool?: string;
}

const genericToVorbisMap: TagFieldMap = {
    title: "TITLE",
    artist: "ARTIST",
    album: "ALBUM",
    albumArtist: "ALBUMARTIST",
    composer: "COMPOSER",
    genre: "GENRE",
    year: "DATE",
    compilation: "COMPILATION",
    trackNumber: "TRACKNUMBER",
    trackTotal: ["TRACKTOTAL", "TOTALTRACKS"],
    discNumber: "DISCNUMBER",
    discTotal: ["DISCTOTAL", "TOTALDISCS"],
    copyright: "COPYRIGHT",
    publisher: "PUBLISHER",
    performer: "PERFORMER",
    license: "LICENSE",
    isrc: "ISRC",
    bpm: "BPM",
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
    year: "year",
    trackNumber: "track",
};

const id3v1ToGenericMap = inverse(genericToId3v1Map);

/*
ID3v2.2
-------
Three character ids
*/
const genericToId3v22Map: TagFieldMap = {
    title: "TT2",
    artist: "TP1",
    album: "TAL",
    albumArtist: "TP2",
    composer: "TCM",
    genre: "TCO",
    year: "TYE",
    compilation: "TCP",
    trackNumber: "TRK",
    trackTotal: ["TRK"],
    discNumber: "TPA",
    discTotal: ["TPA"],
    copyright: "TCR",
    publisher: "TPB",
    isrc: "TRC",
    bpm: "TBP",
};

const id3v22ToGenericMap = inverse(genericToId3v22Map);

/*
ID3v2.3
-------

*/
const genericToId3v23Map: TagFieldMap = {
    title: "TIT2",
    artist: "TPE1",
    album: "TALB",
    albumArtist: "TPE2",
    composer: "TCOM",
    genre: "TCON",
    year: "TDAT",
    compilation: "TCMP",
    trackNumber: "TRCK",
    trackTotal: ["TRCK"],
    discNumber: "TPOS",
    discTotal: ["TPOS"],
    copyright: "TCOP",
    publisher: "TPUB",
    isrc: "TSRC",
    bpm: "TBPM",
};

const id3v23ToGenericMap = inverse(genericToId3v23Map);

/*
ID3v2.4
-------

*/
const genericToId3v24Map: TagFieldMap = {
    title: "TIT2",
    artist: "TPE1",
    album: "TALB",
    albumArtist: "TPE2",
    composer: "TCOM",
    genre: "TCON",
    year: "TDRC",
    compilation: "TCMP",
    trackNumber: "TRCK",
    trackTotal: ["TRCK"],
    discNumber: "TPOS",
    discTotal: ["TPOS"],
    copyright: "TCOP",
    publisher: "TPUB",
    isrc: "TSRC",
    bpm: "TBPM",
};

const id3v24ToGenericMap = inverse(genericToId3v24Map);

/*
iTunes
-------

Reference: https://github.com/sergiomb2/libmp4v2/wiki/iTunesMetadata
*/
const genericToiTunesMap: TagFieldMap = {
    title: "©nam",
    artist: "©ART",
    album: "©alb",
    albumArtist: "aART",
    composer: "©wrt",
    genre: "gnre",
    year: "©day",
    compilation: "cpil",
    trackNumber: "trkn",
    trackTotal: ["trkn"],
    discNumber: "disk",
    discTotal: ["disk"],
    copyright: "cprt",
    encodingTool: "©too",
};

const iTunesToGenericMap = inverse(genericToiTunesMap);

function getMapForTagType(
    tagType: TagType,
    fromGeneric: boolean = true,
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
        case "iTunes":
            return fromGeneric ? genericToiTunesMap : iTunesToGenericMap;
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
    FLAC: "Vorbis",
    MPEG: "ID3v2.4",
    "MPEG 1 Layer 3": "ID3v2.4",
};

function getTagTypeFromCodec(codec) {
    if (!codec) return null;
    return codecToTagTypeMap[codec];
}

export { getMapForTagType, getTagTypeFromCodec };
