export const ENCODINGS = [
    "IBM866",
    "ISO-8859-2",
    "ISO-8859-3",
    "ISO-8859-4",
    "ISO-8859-5",
    "ISO-8859-6",
    "ISO-8859-7",
    "ISO-8859-8",
    "ISO-8859-10",
    "ISO-8859-13",
    "ISO-8859-14",
    "ISO-8859-15",
    "ISO-8859-16",
    "KOI8-R",
    "KOI8-U",
    "macintosh",
    "windows-874",
    "windows-1250",
    "windows-1251",
    "windows-1252",
    "windows-1253",
    "windows-1254",
    "windows-1255",
    "windows-1256",
    "windows-1257",
    "windows-1258",
    "x-mac-cyrillic"
];

export function decodeLegacy(encodedString, encoding) {
    const decoder = new TextDecoder(encoding);
    return decoder.decode(
        new Uint8Array([...encodedString].map((c) => c.charCodeAt(0)))
    );
}

export function encodeUtf8(decodedString) {
    const encoder = new TextEncoder();
    return encoder.encode(decodedString);
}