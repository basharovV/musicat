import type { DynamicPlaylistFile, Song } from "src/App";
import { db } from "../../data/db";
import { isSmartQueryValid, smartQueryUpdater } from "../../data/store";
import type {
    FieldKey,
    QueryPartStructWithValues,
    SavedSmartQuery,
} from "./QueryPart";
import { UserQueryPart } from "./UserQueryPart";
import { snakeCase, upperFirst } from "lodash-es";
import type { Playlist as DynamicPlaylist } from "@zokugun/dynopl";
import { v7 as uuidv7 } from "uuid";

export default class SmartQuery {
    parts: UserQueryPart[] = [];

    id: number = null;
    name: string = null;
    userInput: string = "";

    static fromDynoPL(playlist: DynamicPlaylistFile): void {
        const queryParts: QueryPartStructWithValues[] = [];

        for (const data of playlist.schema.all) {
            if (data.contains) {
                const key = Object.keys(data.gt)[0];
                const pathKey =
                    key === "title"
                        ? "titleContains"
                        : `contains${upperFirst(key)}`;
                const valueKey = key === "title" ? "text" : key;
                const part: QueryPartStructWithValues = {
                    dataType: "song",
                    fieldKey: key as FieldKey,
                    comparison: "contains",
                    description: `smartPlaylists.builder.parts.${pathKey}.title`,
                    example: `smartPlaylists.builder.parts.${pathKey}.example`,
                    prompt:
                        key === "title"
                            ? "title contains {text}"
                            : key === "genre"
                              ? "contains genre {genre}"
                              : "contains tag {tags}",
                    name:
                        key === "title"
                            ? "title-contains"
                            : key === "genre"
                              ? "contains-genre"
                              : "contains-tag",
                    inputRequired: {
                        [valueKey]: {
                            defaultVal: "",
                            isFieldKey: true,
                            isRequired: true,
                            type: "string",
                        },
                    },
                    values: {
                        [valueKey]: data.contains[key],
                    },
                };

                queryParts.push(part);
            } else if (data.inTheRange) {
                const part: QueryPartStructWithValues = {
                    dataType: "song",
                    fieldKey: "year",
                    comparison: "is-between",
                    description:
                        "smartPlaylists.builder.parts.releasedBetween.title",
                    example:
                        "smartPlaylists.builder.parts.releasedBetween.example",
                    prompt: "released between {startYear} and {endYear}",
                    name: "released-between",
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
                    values: {
                        startYear: data.inTheRange[0],
                        endYear: data.inTheRange[1],
                    },
                };

                queryParts.push(part);
            } else if (data.is) {
                const key = Object.keys(data.is)[0];
                const ckey = key === "artistCountry" ? "originCountry" : key;
                const byKey = `by${upperFirst(ckey)}`;
                const part: QueryPartStructWithValues = {
                    comparison: "is-equal",
                    dataType: "song",
                    description: `smartPlaylists.builder.parts.${byKey}.title`,
                    example: `smartPlaylists.builder.parts.${byKey}.example`,
                    fieldKey: ckey as FieldKey,
                    inputRequired: {
                        [ckey]: {
                            defaultVal: "",
                            isFieldKey: true,
                            isRequired: true,
                            type: "string",
                        },
                    },
                    name: `by {${ckey}}`,
                    prompt: `by ${snakeCase(ckey).replaceAll("_", " ")} {${ckey}}`,
                    values: {
                        [ckey]: data.is[key],
                    },
                };

                queryParts.push(part);
            } else if (data.gt) {
                const key = Object.keys(data.gt)[0];
                const pathKey = key === "year" ? "releasedAfter" : "longerThan";
                const valueKey = key === "year" ? "startYear" : "minutes";
                const part: QueryPartStructWithValues = {
                    dataType: "song",
                    fieldKey: key as FieldKey,
                    comparison: "is-greater-than",
                    description: `smartPlaylists.builder.parts.${pathKey}.title`,
                    example: `smartPlaylists.builder.parts.${pathKey}.example`,
                    prompt:
                        key === "year"
                            ? "released after {startYear}"
                            : "longer than {minutes}",
                    name: key === "year" ? "released-after" : "longer-than",
                    inputRequired: {
                        [valueKey]: {
                            defaultVal: key === "year" ? 1940 : 0,
                            isFieldKey: false,
                            isRequired: true,
                            type: "number",
                        },
                    },
                    values: {
                        [valueKey]: data.gt[key],
                    },
                };

                queryParts.push(part);
            }
        }

        const query = new SmartQuery({
            name: playlist.title,
            queryParts,
        });

        playlist.query = query;
    }

    static async loadWithUQI(queryId: `~usq:${string}`): Promise<SmartQuery> {
        // Run the query from the user-built blocks
        const queryName = Number(queryId.substring(5));
        const savedQuery = await db.smartQueries.get(queryName);
        return new SmartQuery(savedQuery);
    }

    static toDynoPL(savedQuery: SavedSmartQuery): DynamicPlaylist {
        const query = new SmartQuery(savedQuery);

        return query.toDynoPL();
    }

    constructor(savedQuery?: SavedSmartQuery) {
        if (savedQuery) {
            this.id = savedQuery.id;
            this.parts = savedQuery.queryParts.map((p) => new UserQueryPart(p));
            this.name = savedQuery?.name || null;
        }
    }

    addPart(userQueryPart: UserQueryPart) {
        this.parts.push(userQueryPart);
        this.validate();
    }

    removePart(partIdx) {
        console.log("parts: ", this.parts.length);
        this.parts.splice(partIdx, 1);
        this.parts = this.parts;
        this.validate();
    }

    get isNameSet() {
        return this.name?.length > 0;
    }

    get isEmpty() {
        return this.parts.filter((p) => p.isValid).length === 0;
    }

    reset() {
        this.parts = [];
        this.name = "";
        this.userInput = "";
    }

    validate() {
        const isValid =
            this.parts.length > 0 && this.parts.every((p) => p.isValid);
        isSmartQueryValid.set(isValid);
        smartQueryUpdater.update((n) => n + 1);

        // if (isValid) {
        //     // Set off a promise to run the query
        //     this.run().then((songs) => {
        //         console.log("got some songs!", songs);
        //     });
        // }
    }

    async run(): Promise<Song[]> {
        return this.parts.reduce(async (resultChain, currentPart) => {
            return resultChain.then((results) => {
                console.log("results", results);
                console.log("checking part", currentPart);
                if (currentPart.isValid) {
                    const filtered = results.filter((song) => {
                        try {
                            const result = currentPart.run(song);
                            return result;
                        } catch (err) {
                            console.error(err);
                        }
                    });
                    console.log("filtered", filtered);

                    return filtered;
                } else {
                    console.log("part " + currentPart.fieldKey + " is invalid");
                }
                return results;
            });
        }, Promise.resolve(db.songs.toArray()));
    }

    toDynoPL(playlist?: DynamicPlaylist): DynamicPlaylist {
        if (!playlist) {
            playlist = {
                id: uuidv7(),
            };
        }

        playlist.all = [];

        for (const part of this.parts) {
            const { fieldKey } = part.queryPart;

            if (part.queryPart.comparison === "contains") {
                const inputKey = Object.keys(part.queryPart.inputRequired)[0];

                playlist.all.push({
                    contains: {
                        [fieldKey]: part.userInputs[inputKey].value,
                    },
                });
            } else if (part.queryPart.comparison === "is-between") {
                playlist.all.push({
                    inTheRange: {
                        [fieldKey]: [
                            part.userInputs.startYear.value,
                            part.userInputs.endYear.value,
                        ],
                    },
                });
            } else if (part.queryPart.comparison === "is-equal") {
                const operand =
                    fieldKey === "originCountry" ? "artistCountry" : fieldKey;

                playlist.all.push({
                    is: {
                        [operand]: part.userInputs[fieldKey].value,
                    },
                });
            } else if (part.queryPart.comparison === "is-greater-than") {
                const inputKey = Object.keys(part.queryPart.inputRequired)[0];

                playlist.all.push({
                    gt: {
                        [fieldKey]: part.userInputs[inputKey].value,
                    },
                });
            }
        }

        return playlist;
    }
}
