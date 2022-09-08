import type { InputType, QueryPartInput, QueryPartStruct } from "./QueryPart";
import "../string.extensions";

export class UserQueryPart {
    queryPart: QueryPartStruct;
    userInputs: {
        [key: string]: QueryPartInput<InputType>;
    } = {};
    fieldKey;

    /**
     * Can we run this query part?
     */
    get isValid() {
        return Object.values(this.userInputs).every((input) => {
            if (input.isRequired) {
                return String(input.value)?.length > 0;
            }
            return true;
        });
    }

    constructor(queryPart: QueryPartStruct) {
        this.queryPart = queryPart;
        this.fieldKey = queryPart.fieldKey;

        // Prepare inputs, with null value
        Object.entries(queryPart.inputRequired).forEach((p) => {
            this.userInputs[p[0]] = { ...p[1], value: null };
        });
    }
    
    run(song: Song): boolean {
        const lhs = String(song[this.fieldKey]).trim().toLowerCase();
        const rhs1 = String(Object.values(this.userInputs)[0].value)
            .toLowerCase();
        const rhs2 =
            Object.values(this.userInputs).length > 1
                ? String(Object.values(this.userInputs)[1].value)
                      .toLowerCase()
                : null;

        switch (this.queryPart.comparison) {
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

    /** This simply validates the input, it does not run the condition! run() does that. */
    validate() {
        switch (this.queryPart.comparison) {
            case "is-equal":
            case "contains":
            case "is-greater-than":
            case "is-less-than":
                this.isValid = lhs?.length > 0 && this.rhs1?.length > 0;
                break;
            case "is-between":
                this.isValid =
                    lhs?.length > 0 &&
                    this.rhs1?.length > 0 &&
                    this.rhs2?.length > 0;
                break;
        }
    }

    toString() {
        const str = `${
            this.rhs2
                ? this.queryPart.description.format(this.rhs1, this.rhs2)
                : this.queryPart.description.format(this.rhs1)
        }`;
        console.log("str", str);
        return str;
    }
}
