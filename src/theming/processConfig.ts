/**
 * Recursively process configuration object from nested to kebab-case
 * @name processConfig
 * @param {object} obj
 *
 */
export default function processConfig(obj) {
    const result = {};

    const recurse = (obj, current) => {
        for (let prop in obj) {
            const value = obj[prop];
            const key = (current ? `${current}-${prop}` : prop)
                .replace(/-_$/g, "")
                // .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
                .replace(/([A-Z])([A-Z])(?=[a-z])/g, "$1-$2")
                .toLowerCase();

            if (value && typeof value === "object") {
                recurse(value, key);
            } else {
                result[key] = value;
            }
        }
    };

    recurse(obj);

    return result;
}
