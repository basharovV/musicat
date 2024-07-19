import type { IACollection, IAFile, IAItem } from "../App";

export async function getIACollections(page: number) {
    const response = await fetch(
        `https://archive.org/advancedsearch.php?fl%5B%5D=identifier&q=collection%3Aaudio_music%20AND%20mediatype%3A(collection)&fl%5B%5D=collection_size&fl%5B%5D=collection_files_count&fl%5B%5D=description&fl%5B%5D=title&rows=20&mediatype=collection&page=${page}&output=json&sort=week%20desc`
    );
    const responseJson = await response.json();
    console.log("response", responseJson);
    const collection: IACollection[] = responseJson.response.docs.map((d) => ({
        id: d.identifier,
        title: d.title,
        description: d.description,
        size: d.collection_size,
        filesCount: d.collection_files_count
    }));
    return collection;
}

export async function getIACollection(collectionId, page) {
    const response = await fetch(
        `https://archive.org/advancedsearch.php?q=collection%3A(${collectionId})%20AND%20mediatype%3A(audio)&fl%5B%5D=identifier&fl%5B%5D=title&sort%5B%5D&sort%5B%5D&sort%5B%5D&rows=50&page=${page}&output=json`
    );
    const responseJson = await response.json();
    console.log("response", responseJson);
    const collection: IAItem[] = responseJson.response.docs.map((d) => ({
        id: d.identifier,
        title: d.title
    }));
    return collection;
}

export async function getIAItem(itemId) {
    const response = await fetch(`https://archive.org/metadata/${itemId}`);
    const responseJson = await response.json();
    const original = findOriginal(responseJson);

    const item: IAItem = {
        id: itemId,
        title: responseJson.metadata.title,
        server1: responseJson.d1,
        server2: responseJson.d2,
        dir: responseJson.dir,
        original: original,
        // Match against types in IAFormat type
        files: responseJson.files
            .filter((f) => isValidFormat(f) && f.source !== "original")
            .map((f) => mapFile(f, responseJson)),
        date: responseJson.metadata?.date,
        performer: responseJson.roles?.performer,
        writer: responseJson.roles?.writer
    };
    return item;
}

function isValidFormat(file: any) {
    return mapFormat(file)?.match(/(mp3|flac)/i);
}

function findOriginal(response): IAFile {
    for (let file of response.files) {
        if (file.source === "original" && mapFormat(file) === "mp3") {
            return mapFile(file, response);
        }
    }

    for (let file of response.files) {
        if (file.source === "original" && mapFormat(file) === "flac") {
            return mapFile(file, response);
        }
    }
    return undefined;
}

function mapFile(f, responseJson) {
    let supportedFormat = mapFormat(f);
    f.format = supportedFormat ? supportedFormat : undefined;
    f.duration = timestampToSeconds(f["length"]);
    console.log("duration", f.duration);
    f.previewSrc = `https://${responseJson.d1}/${responseJson.dir}/${f.name}`;
    f.itemId = responseJson.metadata.identifier;
    return f;
}

function timestampToSeconds(timestamp) {
    console.log("timestamp", timestamp);
    if (!timestamp || timestamp?.length === 0) return undefined;
    // Split the timestamp into minutes and seconds
    const parts = timestamp.split(":");
    let minutes = 0;
    let seconds = 0;
    if (parts.length !== 2) {
        if (!isNaN(Number(timestamp))) {
            seconds = Number(timestamp);
        }
    } else {
        minutes = parseInt(parts[0], 10);
        seconds = parseInt(parts[1], 10);
    }

    if (isNaN(minutes) || isNaN(seconds)) {
        throw new Error("Invalid number in timestamp.");
    }

    // Convert the minutes to seconds and add the seconds
    return minutes * 60 + seconds;
}

function mapFormat(file) {
    if (file.format?.match(/(mp3)/i)) {
        return "mp3";
    } else if (file.format?.match(/(flac)/i)) {
        return "flac";
    }
    return undefined;
}
