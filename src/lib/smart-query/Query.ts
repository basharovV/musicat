import type { Song } from "src/App";
import { db } from "../../data/db";
import { isSmartQueryValid, smartQueryUpdater } from "../../data/store";
import type { SavedSmartQuery } from "./QueryPart";
import { UserQueryPart } from "./UserQueryPart";
export default class SmartQuery {
    parts: UserQueryPart[] = [];

    id: number = null;
    name: string = null;
    userInput: string = "";

    static async loadWithUQI(queryId: string): Promise<SmartQuery> {
        // Run the query from the user-built blocks
        const queryName = Number(queryId.substring(5));
        const savedQuery = await db.smartQueries.get(queryName);
        return new SmartQuery(savedQuery);
    }

    constructor(savedQuery?: SavedSmartQuery) {
        if (savedQuery) {
            this.id = savedQuery.id;
            this.parts = savedQuery.queryParts.map((p) => new UserQueryPart(p));
            this.name = savedQuery.name;
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
        return this.name !== null && this.name.length > 0;
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

    async save() {
        if (this.id) {
            await db.smartQueries.update(this.id, {
                name: this.name,
                queryParts: this.parts.map((p) => ({
                    ...p.queryPart,
                    values: Object.entries(p.userInputs).reduce(
                        (obj, current) => {
                            obj[current[0]] = current[1].value;
                            return obj;
                        },
                        {},
                    ),
                })),
            });

            return this.id;
        } else {
            return await db.smartQueries.put({
                name: this.name,
                queryParts: this.parts.map((p) => ({
                    ...p.queryPart,
                    values: Object.entries(p.userInputs).reduce(
                        (obj, current) => {
                            obj[current[0]] = current[1].value;
                            return obj;
                        },
                        {},
                    ),
                })),
            });
        }
    }
}
