import type { QueryPartStruct } from "./QueryPart";

export enum QUERY_PARTS {
    BY_ARTIST = "by {artist}",
    RELEASED_BETWEEN = "released-between",
    RELEASED_AFTER = "released-after",
    TITLE_CONTAINS = "title-contains",
    LONGER_THAN = "longer-than",
    CONTAINS_GENRE = "contains-genre",
    RELEASED_IN = "released-in",
    FROM_COUNTRY = "from-country"
}

type QUERY_PART = keyof typeof QUERY_PARTS;

export function getQueryPart(queryPartName: QUERY_PART) {
    return BUILT_IN_QUERY_PARTS.find((q) => q.name === QUERY_PARTS[queryPartName]);
}

export const BUILT_IN_QUERY_PARTS: QueryPartStruct[] = [
    {
        dataType: "song",
        fieldKey: "artist",
        comparison: "is-equal",
        description: "by artist",
        example: "by Charlie Parker",
        prompt: "by {artist}",
        name: QUERY_PARTS.BY_ARTIST,
        inputRequired: {
            "artist": {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "year",
        comparison: "is-between",
        description: "released between",
        example: "released between 1950 and 1967",
        prompt: "released between {startYear} and {endYear}",
        name: QUERY_PARTS.RELEASED_BETWEEN,
        inputRequired: {
            "startYear": {
                defaultVal: 1940,
                isFieldKey: false,
                isRequired: true,
                type: "number"
            },
            "endYear": {
                defaultVal: 1960,
                isFieldKey: false,
                isRequired: true,
                type: "number"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "year",
        comparison: "is-greater-than",
        description: "released after",
        example: "released after 1950",
        prompt: "released after {startYear}",
        name: QUERY_PARTS.RELEASED_AFTER,
        inputRequired: {
            "startYear": {
                defaultVal: 1940,
                isFieldKey: false,
                isRequired: true,
                type: "number"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "title",
        comparison: "contains",
        description: "song title contains {text}",
        example: "song title contains love",
        prompt: "title contains {text}",
        name: QUERY_PARTS.TITLE_CONTAINS,
        inputRequired: {
            "text": {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "duration",
        comparison: "is-greater-than",
        description: "longer than",
        example: "longer than 04:00",
        prompt: "longer than {minutes}",
        name: QUERY_PARTS.LONGER_THAN,
        inputRequired: {
            "minutes": {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "number"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "genre",
        comparison: "contains",
        description: "contains genre",
        example: "contains 'disco'",
        prompt: "contains genre {genre}",
        name: QUERY_PARTS.CONTAINS_GENRE,
        inputRequired: {
            "genre": {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "year",
        comparison: "is-equal",
        description: "released in",
        example: "released in 1976",
        prompt: "released in {year}",
        name: QUERY_PARTS.RELEASED_IN,
        inputRequired: {
            "year": {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string"
            }
        }
    },
    {
        dataType: "song",
        fieldKey: "originCountry",
        comparison: "is-equal",
        description: "from country",
        example: "from Australia",
        prompt: "from {originCountry}",
        name: QUERY_PARTS.FROM_COUNTRY,
        inputRequired: {
            "originCountry": {
                defaultVal: "",
                isFieldKey: true,
                isRequired: true,
                type: "string"
            }
        }
    }
];
