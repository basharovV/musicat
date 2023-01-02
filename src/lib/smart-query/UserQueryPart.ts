import type {
    InputType,
    QueryPartInput,
    QueryPartStruct,
    QueryPartStructWithValues
} from "./QueryPart";
import "../string.extensions";
import type { Song } from "src/App";

function instanceOfQueryPartStructWithValues(
    object: any
): object is QueryPartStructWithValues {
    return typeof object === 'object' && "values" in object;
}

export class UserQueryPart {
    // Not saved variant (builder mode)
    queryPart: QueryPartStruct;
    withValues = false;
    // Saved variant (already in db, just run it)
    queryPartWithValues: QueryPartStructWithValues;

    userInputs: {
        [key: string]: QueryPartInput<InputType>;
    } = {};
    fieldKey;

    /**
     * Can we run this query part?
     */
    get isValid() {
        if (this.queryPartWithValues) return true;
        return Object.values(this.userInputs).every((input) => {
            if (input.isRequired) {
                return (
                    input?.value !== undefined &&
                    input?.value !== null &&
                    String(input.value)?.length > 0
                );
            }
            return true;
        });
    }

    constructor(queryPart: QueryPartStruct | QueryPartStructWithValues) {
        if (instanceOfQueryPartStructWithValues(queryPart)) {
            this.queryPartWithValues = queryPart;
        } else {
            this.queryPart = queryPart;

            // Prepare inputs, with null value
            Object.entries(queryPart.inputRequired).forEach((p) => {
                this.userInputs[p[0]] = { ...p[1], value: null };
            });
        }

        this.fieldKey = queryPart.fieldKey;
    }

    run(song: Song): boolean {
        const lhs = String(song[this.fieldKey]).trim().toLowerCase();
        const rhs1 = String(
            this.queryPartWithValues
                ? this.queryPartWithValues.values[
                      Object.keys(this.queryPartWithValues.inputRequired)[0]
                  ]
                : Object.values(this.userInputs)[0].value
        ).toLowerCase();
        const rhs2 = String(
            this.queryPartWithValues
                ? this.queryPartWithValues.values[
                      Object.keys(this.queryPartWithValues.inputRequired)[1]
                  ]
                : Object.values(this.userInputs).length > 1
                ? Object.values(this.userInputs)[1].value
                : null
        ).toLowerCase();
        
        const comparison = this.queryPartWithValues ? this.queryPartWithValues.comparison : this.queryPart.comparison
        
        switch (comparison) {
            case "is-equal":
                return lhs === rhs1;
            case "contains":
                return lhs.includes(rhs1);
            case "is-greater-than":
                return lhs.localeCompare(rhs1) === 1;
            case "is-less-than":
                return lhs.localeCompare(rhs1) === -1;
            case "is-between":
                try {
                    return (
                        lhs.localeCompare(rhs1) === 1 &&
                        lhs.localeCompare(rhs2) === -1
                    );
                } catch (err) {
                    console.error("Unable to parse as number: ", rhs1, rhs2);
                    return false;
                }
        }
    }
}
