import type { ContentFileType } from "src/App";

export function isFileOrDirectory(path) {
    const lastChar = path.slice(-1);

    // If the path ends with a slash, assume it's a directory
    if (lastChar === "/" || lastChar === "\\") {
        return "directory";
    }

    // If there's a dot in the last part of the path, assume it's a file
    if (
        (path.split("/").pop() !== path &&
            path.split("/").pop().includes(".")) ||
        (path.split(/\\/).pop() !== path &&
            path.split(/\\/).pop().includes("."))
    ) {
        return "file";
    }

    return "directory";
}

export function isAudioFile(filename: string): boolean {
    return filename.match(/\.(mp3|ogg|aac|flac|wav|m4a)$/i) !== null;
}

export function isCueFile(filename: string): boolean {
    return filename.match(/\.(cue)$/i) !== null;
}

export function isVideoFile(filename: string): boolean {
    return filename.match(/\.(mov|mp4|webm|mkv|avi)$/i) !== null;
}
export function isImageFile(filename: string): boolean {
    return filename.match(/\.(jpg|png|webp)$/i) !== null;
}
export function isTextFile(filename: string): boolean {
    return filename.match(/\.(txt|rtf|md)$/i) !== null;
}

export function getImageFormat(extension: string): string {
    switch (extension) {
        case "jpg":
        case "jpeg":
            return "image/jpeg";
        case "png":
            return "image/png";
        case "webp":
            return "image/webp";
        default:
            return null;
    }
}

export function getImageExtension(mimeType: string): string {
    switch (mimeType) {
        case "image/jpeg":
            return "jpg";
        case "image/png":
            return "png";
        case "image/webp":
            return "webp";
        default:
            return null;
    }
}

export function getContentFileType(filename: string): ContentFileType {
    const extensionMatches = filename.match(/\.[0-9a-z]+$/i);
    const extension = extensionMatches ? extensionMatches[0] : "unsupported";
    if (isAudioFile(filename)) {
        return {
            type: "audio",
            extension,
        };
    } else if (isVideoFile(filename)) {
        return {
            type: "video",
            extension,
        };
    } else if (isImageFile(filename)) {
        return {
            type: "image",
            extension,
        };
    } else if (isTextFile(filename)) {
        return {
            type: "txt",
            extension,
        };
    }
    return {
        type: "unsupported",
        extension,
    };
}
