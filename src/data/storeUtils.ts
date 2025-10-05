import { writable, type Writable } from "svelte/store";
import { mergeByKeyDeep } from "../utils/ArrayUtils";

export interface PersistedValue<T> {
    __version: number;
    value: T;
}

export interface PersistentOptions<T> {
    version: number;
    matchKey?: string;
    migrate?: (oldValue: unknown) => T | null;
}

/**
 * A writable store that:
 * - Persists to localStorage
 * - Supports deep merges and migrations
 * - Provides a `.reset()` method
 */
export interface PersistentWritable<T> extends Writable<T> {
    reset: () => void;
}
/**
 * Writable store that auto-persists to localStorage,
 * supports merging arrays of objects and upgrading old persisted formats.
 */
export function persistentWritable<T>(
    initial: T,
    key: string,
    options: PersistentOptions<T> = { version: 1 },
): PersistentWritable<T> {
    const { version, matchKey, migrate } = options;

    const val = writable(initial, (set) => {
        const persistedStr = localStorage.getItem(key);
        if (!persistedStr) {
            set(initial);
            return;
        }

        try {
            const parsed = JSON.parse(persistedStr);

            // 🧩 Case 1: legacy primitive value (string, number, boolean, etc.)
            if (
                parsed === null ||
                typeof parsed !== "object" ||
                Array.isArray(parsed)
            ) {
                console.warn(
                    `[persistentWritable] Legacy primitive value detected for "${key}".`,
                );
                handleUpgrade(
                    parsed,
                    null,
                    initial,
                    version,
                    migrate,
                    set,
                    key,
                );
                return;
            }

            // 🧩 Case 2: new format with version
            if ("__version" in parsed && "value" in parsed) {
                const persisted = parsed as PersistedValue<T>;

                if (persisted.__version !== version) {
                    console.warn(
                        `[persistentWritable] Version mismatch for "${key}": ${persisted.__version} → ${version}`,
                    );
                    handleUpgrade(
                        persisted.value,
                        persisted.__version,
                        initial,
                        version,
                        migrate,
                        set,
                        key,
                    );
                    return;
                }

                // Same version → merge normally
                const valToSet = mergeValues(
                    initial,
                    persisted.value,
                    matchKey,
                );
                set(valToSet);
                return;
            }

            // 🧩 Case 3: old unversioned object (no __version)
            console.warn(
                `[persistentWritable] Unversioned data detected for "${key}".`,
            );
            handleUpgrade(
                parsed,
                undefined,
                initial,
                version,
                migrate,
                set,
                key,
            );
        } catch (err) {
            console.warn(`Failed to parse persisted store "${key}":`, err);
            set(initial);
        }
    });

    val.subscribe((val) => {
        const toStore: PersistedValue<T> = {
            __version: version,
            value: val,
        };
        localStorage.setItem(key, JSON.stringify(toStore));
    });

    const { set } = val;

    function reset() {
        console.log("Resetting field to default values: ", initial);
        set(initial);
    }

    return { ...val, reset };
}

/**
 * Handles upgrades or resets when the version or type changes.
 */
function handleUpgrade<T>(
    oldVal: unknown,
    oldVersion: number | null | undefined,
    initial: T,
    newVersion: number,
    migrate: ((oldValue: unknown) => T | null) | undefined,
    set: (v: T) => void,
    key: string,
) {
    if (migrate) {
        const migrated = migrate(oldVal);
        if (migrated !== null) {
            console.info(
                `[persistentWritable] Migrated "${key}" successfully.`,
            );
            set(migrated);
            localStorage.setItem(
                key,
                JSON.stringify({ __version: newVersion, value: migrated }),
            );
            return;
        }
    }
    console.warn(`[persistentWritable] Resetting "${key}" to defaults.`);
    set(initial);
    localStorage.setItem(
        key,
        JSON.stringify({ __version: newVersion, value: initial }),
    );
}

/**
 * Merges arrays and objects intelligently.
 */
function mergeValues<T>(initial: T, persisted: any, matchKey?: string): T {
    if (initial && initial.constructor === Object) {
        Object.assign(initial as any, persisted);
        return initial;
    }
    if (
        Array.isArray(initial) &&
        Array.isArray(persisted) &&
        typeof initial[0] === "object"
    ) {
        return mergeByKeyDeep(persisted, initial, matchKey) as T;
    }
    return persisted ?? initial;
}
