export function isAudioFile(filename: string): boolean {
    return filename.match(/\.(mp3|ogg|flac|wav)$/) !== null;
}