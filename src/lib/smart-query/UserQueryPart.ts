import type {
    InputType,
    QueryPartInput,
    QueryPartStruct,
    QueryPartStructWithValues,
} from "./QueryPart";
import "../string.extensions";
import { TRACK_FIELD_KIND, type FilterNode } from "../../data/query";

function instanceOfQueryPartStructWithValues(
    object: any,
): object is QueryPartStructWithValues {
    return typeof object === "object" && "values" in object;
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
        this.queryPart = queryPart;
        this.fieldKey = queryPart.fieldKey;

        if (instanceOfQueryPartStructWithValues(queryPart)) {
            this.queryPartWithValues = queryPart;

            Object.entries(queryPart.inputRequired).forEach((p) => {
                this.userInputs[p[0]] = {
                    ...p[1],
                    value: queryPart.values[p[0]],
                };
            });
        } else {
            // Prepare inputs, with null value
            Object.entries(queryPart.inputRequired).forEach((p) => {
                this.userInputs[p[0]] = { ...p[1], value: null };
            });
        }
    }

    /**
     * Coerces a raw input value to what the field's SQL column actually
     * holds (see `TRACK_FIELD_KIND`) rather than trusting the query part's
     * declared `type`, which is inconsistent in `QueryParts.ts` (e.g.
     * RELEASED_IN declares its year input as `type: "string"`).
     */
    private coerceValue(raw: InputType): string | number | boolean {
        const kind = TRACK_FIELD_KIND[this.fieldKey] ?? "text";
        if (kind === "int") {
            return Math.trunc(Number(raw));
        }
        if (kind === "real") {
            // "duration" is the only real-valued field. The UI collects it
            // in minutes (see QueryParts.ts's LONGER_THAN) but the column is
            // seconds -- convert here rather than at every call site.
            return this.fieldKey === "duration" ? Number(raw) * 60 : Number(raw);
        }
        return String(raw);
    }

    /**
     * Converts this part into a `QuerySpec` filter node, evaluated by the
     * database instead of per-song in JS.
     */
    toFilterNode(): FilterNode | null {
        const comparison = this.queryPartWithValues
            ? this.queryPartWithValues.comparison
            : this.queryPart.comparison;

        if (comparison === "not-empty") {
            return { kind: "notEmpty", field: this.fieldKey };
        }

        if (Object.values(this.userInputs).length === 0) return null;

        const rhs1 = this.queryPartWithValues
            ? this.queryPartWithValues.values[
                  Object.keys(this.queryPartWithValues.inputRequired)[0]
              ]
            : Object.values(this.userInputs)[0].value;
        const rhs2 = this.queryPartWithValues
            ? this.queryPartWithValues.values[
                  Object.keys(this.queryPartWithValues.inputRequired)[1]
              ]
            : Object.values(this.userInputs).length > 1
              ? Object.values(this.userInputs)[1].value
              : null;

        switch (comparison) {
            case "is-equal":
                return { kind: "cmp", field: this.fieldKey, op: "eq", value: this.coerceValue(rhs1) };
            case "contains":
                return { kind: "contains", field: this.fieldKey, value: String(rhs1) };
            case "is-greater-than":
                return { kind: "cmp", field: this.fieldKey, op: "gt", value: this.coerceValue(rhs1) };
            case "is-less-than":
                return { kind: "cmp", field: this.fieldKey, op: "lt", value: this.coerceValue(rhs1) };
            case "is-between":
                return {
                    kind: "between",
                    field: this.fieldKey,
                    min: this.coerceValue(rhs1),
                    max: this.coerceValue(rhs2),
                };
            default:
                return null;
        }
    }
}
