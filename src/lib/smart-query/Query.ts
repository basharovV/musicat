import { db } from "../../data/db";
import { isSmartQueryValid, smartQueryUpdater } from "../../data/store";
import type { UserQueryPart } from "./UserQueryPart";
import { uniqBy } from "lodash-es";
export default class SmartQuery {
    parts: UserQueryPart[] = [];

    userInput: string = "";

    addPart(userQueryPart: UserQueryPart) {
        this.parts.push(userQueryPart);
        this.validate();
    }

    removePart(partIdx) {
        console.log('parts: ', this.parts.length);
        this.parts.splice(partIdx, 1);
        this.parts = this.parts;
        this.validate();
    }

    validate() {
        const isValid = this.parts.every((p) => p.isValid);
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
                        const result = currentPart.run(song);
                        return result;
                    });
                    console.log("filtered", filtered);

                    return filtered;
                }
                return results;
            });
        }, Promise.resolve(db.songs.toArray()));
    }
}
