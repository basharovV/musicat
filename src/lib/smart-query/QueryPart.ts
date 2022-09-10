import type { Comparison, DataType } from "src/App";

/**
 * At the moment we just create query parts per field.
 * This can expand to more complex queries
 */
type FieldKey = "title" | "artist" | "album" | "year" | "genre" | "duration";

export type InputType = string | number | boolean;
export type InputTypeStr = "string" | "number" | "boolean";

interface QueryPart {
    queryPart: QueryPartStruct;
    userInputs: {
        [key: string]: QueryPartInput<InputType>;
    };
    fieldKey;
}
interface QueryPartInputStruct<T extends InputType> {
    defaultVal: T;
    isRequired: boolean;
    isFieldKey: boolean; // Is this input for the query part field eg. album or genre
    type: InputTypeStr;
}

export interface QueryPartInput<T extends InputType>
    extends QueryPartInputStruct<T> {
    value: T;
}

export interface QueryPartStruct {
    /**
     * The type of entry - at the moment just Song
     */
    dataType: DataType;
    /**
     * The field to check
     */
    fieldKey: FieldKey;
    /**
     * The comparison/condition to run on this field
     */
    comparison: Comparison;
    /**
     * Query part name eg. released
     * If this is null, we contruct a generic one eg. "where 'Year' is 1969"
     */
    name: string;
    /**
     * The prompt the user sees during input
     * eg. released in 1968
     */
    prompt: string;
    /**
     * An example to show the user
     */
    example: string;
    /**
     * The description the user sees after input
     * Supports substitions by using {}, or {1}, {2}, etc
     */
    description: string;
    /**
     * Input map required
     */
    inputRequired: {
        [key: string]: QueryPartInputStruct<InputType>;
    };
}
export interface QueryPartStructWithValues extends QueryPartStruct {
    values: {
        [key: string]: any;
    };
}

export interface SavedSmartQuery {
    name: string;
    queryParts: QueryPartStructWithValues[];
}
