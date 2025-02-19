export function reverseHex(hex: string): string {
    const m = /(\w\w)(\w\w)(\w\w)/.exec(hex);

    if (m) {
        return `${m[3]}${m[2]}${m[1]}`;
    } else {
        return "";
    }
}
