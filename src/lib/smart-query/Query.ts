import { invoke } from "@tauri-apps/api/core";
import type { Song } from "src/App";
import { queryAllTracks, type FilterNode } from "../../data/query";
import { isSmartQueryValid, smartQueryUpdater } from "../../data/store";
import type { SavedSmartQuery } from "./QueryPart";
import { UserQueryPart } from "./UserQueryPart";
export default class SmartQuery {
    parts: UserQueryPart[] = [];

    id: number = null;
    name: string = null;
    userInput: string = "";

    static async loadWithUQI(queryId: `~usq:${string}`): Promise<SmartQuery> {
        // Run the query from the user-built blocks
        const queryName = Number(queryId.substring(5));
        const saved = await invoke<{
            id: number;
            name: string;
            queryParts: SavedSmartQuery["queryParts"];
        } | null>("get_smart_query", { id: queryName });
        return new SmartQuery(
            saved ? { id: saved.id, name: saved.name, queryParts: saved.queryParts } : undefined,
        );
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
        const nodes: FilterNode[] = this.parts
            .filter((p) => p.isValid)
            .map((p) => p.toFilterNode())
            .filter((n): n is FilterNode => n !== null);

        if (nodes.length === 0) {
            return [];
        }

        return queryAllTracks({
            filter: nodes.length === 1 ? nodes[0] : { kind: "and", nodes },
        });
    }

    private serializedQueryParts(): SavedSmartQuery["queryParts"] {
        return this.parts.map((p) => ({
            ...p.queryPart,
            values: Object.entries(p.userInputs).reduce(
                (obj, current) => {
                    obj[current[0]] = current[1].value;
                    return obj;
                },
                {},
            ),
        }));
    }

    async save() {
        this.id = await invoke<number>("save_smart_query", {
            id: this.id ?? null,
            name: this.name,
            queryParts: this.serializedQueryParts(),
        });
        return this.id;
    }
}
