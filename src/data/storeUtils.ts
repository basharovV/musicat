import { writable, type Writable } from "svelte/store";

/**
 * Writable store that auto-persists to localStorage,
 * when you just need a simple persist mechanism without extra logic.
 *
 * @param initial Initial value
 * @param key Key to persist
 * @returns Writable object
 */
export function persistentWritable<T>(initial: T, key: string): Writable<T> {
    const val = writable(initial, (set) => {
        let persisted = localStorage.getItem(key);
        if (persisted) {
            if (!!initial && initial.constructor === Object) {
                // Merge default and persisted values (persisted overrides)
                // This is useful when adding new fields
                Object.assign(initial, JSON.parse(persisted));
            }
            set(initial);
        }
    });
    val.subscribe((val) => {
        localStorage.setItem(key, JSON.stringify(val));
    });
    return val;
}
