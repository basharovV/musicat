import { readTextFile } from "@tauri-apps/api/fs";
import type { ContentFileType } from "src/App";

export function isAudioFile(filename: string): boolean {
    return filename.match(/\.(mp3|ogg|aac|flac|wav)$/) !== null;
}
export function isVideoFile(filename: string): boolean {
    return filename.match(/\.(mov|mp4|webm|mkv|avi)$/) !== null;
}
export function isImageFile(filename: string): boolean {
    return filename.match(/\.(jpg|png|webp)$/) !== null;
}
export function isTextFile(filename: string): boolean {
    return filename.match(/\.(txt|rtf|md)$/) !== null;
}

export function getContentFileType(filename: string): ContentFileType {
    const extensionMatches = filename.match(/\.[0-9a-z]+$/i);
    const extension = extensionMatches ? extensionMatches[0] : "unsupported";
    if (isAudioFile(filename)) {
        return {
            type: "audio",
            extension
        };
    } else if (isVideoFile(filename)) {
        return {
            type: "video",
            extension
        };
    } else if (isImageFile(filename)) {
        return {
            type: "image",
            extension
        };
    } else if (isTextFile(filename)) {
        return {
            type: "txt",
            extension
        };
    }
    return {
        type: "unsupported",
        extension
    };
}
