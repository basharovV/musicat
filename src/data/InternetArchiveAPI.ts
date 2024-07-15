import type { IACollection, IAItem } from "../App";

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
    const { previewSrc, duration } = findPreviewSrc(responseJson);
    const item: IAItem = {
        id: itemId,
        title: responseJson.metadata.title,
        server1: responseJson.d1,
        server2: responseJson.d2,
        dir: responseJson.dir,
        previewSrc,
        duration,
        // Match against types in IAFormat type
        files:
            responseJson.files
                ?.map((f) => {
                    let supportedFormat = mapFormat(f);
                    f.format = supportedFormat ? supportedFormat : undefined;
                    return f;
                })
                .filter((f) => f.format) ?? [],
        date: responseJson.metadata?.date,
        performer: responseJson.roles?.performer,
        writer: responseJson.roles?.writer
    };
    return item;
}

function findPreviewSrc(response) {
    for (let file of response.files) {
        if (file.source === "original" && mapFormat(file) === "mp3") {
            return {
                previewSrc: `https://${response.d1}/${response.dir}/${file.name}`,
                duration: Number(file["length"])
            };
        }
    }
    
    for (let file of response.files) {
        if (file.source === "original" && mapFormat(file) === "flac") {
            return {
                previewSrc: `https://${response.d1}/${response.dir}/${file.name}`,
                duration: Number(file["length"])
            };
        }
    }
    return undefined;
}

function mapFormat(file) {
    if (file.format.match(/(mp3)/i)) {
        return "mp3";
    } else if (file.format.match(/(flac)/i)) {
        return "flac";
    }
    return undefined;
}
