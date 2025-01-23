export const HEADER_HEIGHT = 22;
export const PADDING = 14;
export const ROW_HEIGHT = 26;

export async function getAlbumDetailsHeight(length: number): Promise<number> {
    return PADDING + 300 + HEADER_HEIGHT + length * ROW_HEIGHT + 8 + PADDING;
}
