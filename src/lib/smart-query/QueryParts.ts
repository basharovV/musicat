import type { QueryPartStruct } from "./QueryPart";

export enum QUERY_PARTS {
    BY_ALBUM_ARTIST = "by {albumArtist}",
    BY_ARTIST = "by {artist}",
    BY_COMPOSER = "by-composer",
    CONTAINS_GENRE = "contains-genre",
    CONTAINS_TAG = "contains-tag",
    FROM_COUNTRY = "from-country",
    LONGER_THAN = "longer-than",
    RELEASED_AFTER = "released-after",
    RELEASED_BETWEEN = "released-between",
    RELEASED_IN = "released-in",
    TITLE_CONTAINS = "title-contains",
}

type QUERY_PART = keyof typeof QUERY_PARTS;

export function getQueryPart(queryPartName: QUERY_PART) {
    return BUILT_IN_QUERY_PARTS.find(
        (q) => q.name === QUERY_PARTS[queryPartName],
    );
}

export const BUILT_IN_QUERY_PARTS: QueryPartStruct[] = [
    {
        dataType: "song",
        fieldKey: "artist",
        comparison: "is-equal",
        description: "smartPlaylists.builder.parts.byArtist.title",
        example: "smartPlaylists.builder.parts.byArtist.example",
        prompt: "by {artist}",
        name: QUERY_PARTS.BY_ARTIST,
        inputRequired: {
            artist: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "composer",
        comparison: "is-equal",
        description: "smartPlaylists.builder.parts.byComposer.title",
        example: "smartPlaylists.builder.parts.byComposer.example",
        prompt: "by composer: {composer}",
        name: QUERY_PARTS.BY_COMPOSER,
        inputRequired: {
            composer: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "albumArtist",
        comparison: "is-equal",
        description: "smartPlaylists.builder.parts.byAlbumArtist.title",
        example: "smartPlaylists.builder.parts.byAlbumArtist.example",
        prompt: "by album artist {albumArtist}",
        name: QUERY_PARTS.BY_ALBUM_ARTIST,
        inputRequired: {
            albumArtist: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "year",
        comparison: "is-between",
        description: "smartPlaylists.builder.parts.releasedBetween.title",
        example: "smartPlaylists.builder.parts.releasedBetween.example",
        prompt: "released between {startYear} and {endYear}",
        name: QUERY_PARTS.RELEASED_BETWEEN,
        inputRequired: {
            startYear: {
                defaultVal: 1940,
                isFieldKey: false,
                isRequired: true,
                type: "number",
            },
            endYear: {
                defaultVal: 1960,
                isFieldKey: false,
                isRequired: true,
                type: "number",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "year",
        comparison: "is-greater-than",
        description: "smartPlaylists.builder.parts.releasedAfter.title",
        example: "smartPlaylists.builder.parts.releasedAfter.example",
        prompt: "released after {startYear}",
        name: QUERY_PARTS.RELEASED_AFTER,
        inputRequired: {
            startYear: {
                defaultVal: 1940,
                isFieldKey: false,
                isRequired: true,
                type: "number",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "year",
        comparison: "is-equal",
        description: "smartPlaylists.builder.parts.releasedIn.title",
        example: "smartPlaylists.builder.parts.releasedIn.example",
        prompt: "released in {year}",
        name: QUERY_PARTS.RELEASED_IN,
        inputRequired: {
            year: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "title",
        comparison: "contains",
        description: "smartPlaylists.builder.parts.titleContains.title",
        example: "smartPlaylists.builder.parts.titleContains.example",
        prompt: "title contains {text}",
        name: QUERY_PARTS.TITLE_CONTAINS,
        inputRequired: {
            text: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "duration",
        comparison: "is-greater-than",
        description: "smartPlaylists.builder.parts.longerThan.title",
        example: "smartPlaylists.builder.parts.longerThan.example",
        prompt: "longer than {minutes}",
        name: QUERY_PARTS.LONGER_THAN,
        inputRequired: {
            minutes: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "number",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "genre",
        comparison: "contains",
        description: "smartPlaylists.builder.parts.containsGenre.title",
        example: "smartPlaylists.builder.parts.containsGenre.example",
        prompt: "contains genre {genre}",
        name: QUERY_PARTS.CONTAINS_GENRE,
        inputRequired: {
            genre: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "originCountry",
        comparison: "is-equal",
        description: "smartPlaylists.builder.parts.fromCountry.title",
        example: "smartPlaylists.builder.parts.fromCountry.example",
        prompt: "from {originCountry}",
        name: QUERY_PARTS.FROM_COUNTRY,
        inputRequired: {
            originCountry: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
    {
        dataType: "song",
        fieldKey: "tags",
        comparison: "contains",
        description: "smartPlaylists.builder.parts.containsTag.title",
        example: "smartPlaylists.builder.parts.containsTag.example",
        prompt: "contains tag {tags}",
        name: QUERY_PARTS.CONTAINS_TAG,
        inputRequired: {
            tags: {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string",
            },
        },
    },
];
