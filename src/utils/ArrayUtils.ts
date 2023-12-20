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
