/**
 * Helper function to determine whether input is an Object
 * @param {object} obj
 * @returns {boolean}
 */
export default function isObject(obj) {
    return Object.prototype.toString.call(obj) === "[object Object]";
}
