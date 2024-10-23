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
                data: []
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
    const newArray = [...array];
    const [removedElement] = newArray.splice(fromIndex, 1);
    newArray.splice(
        toIndex > fromIndex ? toIndex - 1 : toIndex,
        0,
        removedElement
    );
    return newArray;
}

export function dedupe(array: string[]) {
    return [...new Set(array)];
}
