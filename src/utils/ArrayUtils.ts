export function shuffleArray(array) {
    const copy = [...array];
    for (let i = copy.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [copy[i], copy[j]] = [copy[j], copy[i]];
    }
    return copy;
}

export const groupBy = function (xs, key) {
    return xs.reduce(function (rv, x) {
        if (rv[x[key]] === undefined) {
            rv[x[key]] = {
                data: [],
            };
        }
        (rv[x[key]].data = rv[x[key]].data || []).push(x);
        return rv;
    }, {});
};

export function swapArrayElements(array, indexA, indexB) {
    const newArray = [...array]; // Create a shallow copy to avoid modifying the original array
    const temp = newArray[indexA];
    newArray[indexA] = newArray[indexB];
    newArray[indexB] = temp;
    console.log("newArray", newArray);
    return newArray;
}

export function moveArrayElement(array, fromIndex, toIndex) {
    console.log("moving element", fromIndex, toIndex);
    const newArray = [...array];
    const [removedElement] = newArray.splice(fromIndex, 1);
    newArray.splice(toIndex, 0, removedElement);
    return newArray;
}

export function dedupe(array: string[]) {
    return [...new Set(array)];
}

/**
 * Deeply merges two arrays of objects by a shared key.
 * Nested fields from `base` are preserved unless overridden in `updates`.
 *
 * @param base - The original array of objects
 * @param updates - The array with updated or new objects
 * @param key - The property key to match on (e.g. 'id')
 * @returns A new merged array
 */
export function mergeByKeyDeep<T extends Array<any>>(
    base: T,
    updates: T,
    key: string,
): T {
    const updateMap = new Map(updates.map((item) => [item[key], item]));

    const deepMerge = (target: any, source: any): any => {
        if (Array.isArray(target) && Array.isArray(source)) {
            return source; // replace arrays — could also merge if you prefer
        } else if (isObject(target) && isObject(source)) {
            const result: Record<string, any> = { ...target };
            for (const [k, v] of Object.entries(source)) {
                result[k] = k in target ? deepMerge(target[k], v) : v;
            }
            return result;
        }
        return source;
    };

    const isObject = (obj: any): obj is Record<string, any> =>
        obj && typeof obj === "object" && !Array.isArray(obj);

    const merged = base.map((item) => {
        const updated = updateMap.get(item[key]);
        return updated ? deepMerge(item, updated) : item;
    });

    // Include new items not present in base
    for (const [k, v] of updateMap.entries()) {
        if (!base.some((item) => item[key] === k)) {
            merged.push(v);
        }
    }

    return merged;
}
