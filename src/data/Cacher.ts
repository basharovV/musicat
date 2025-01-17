// import {
//     BaseDirectory,
//     create,
//     remove,
//     writeFile
// } from "@tauri-apps/plugin-fs";
// import { appDataDir } from "@tauri-apps/api/path";

import { path } from "@tauri-apps/api";
import { appDataDir } from "@tauri-apps/api/path";
import { exists, remove } from "@tauri-apps/plugin-fs";

// // function createDir(dir: string, options: object = {}): Promise<unknown> {
// //     return invokeTauriCommand({
// //         __tauriModule: "Fs",
// //         message: {
// //             cmd: "createDir",
// //             path: dir,
// //             options: options
// //         }
// //     });
// // }

// // function writeImage(
// //     path: string,
// //     data: Uint8Array,
// //     options: object = {}
// // ): Promise<unknown> {
// //     return invokeTauriCommand({
// //         __tauriModule: "Fs",
// //         message: {
// //             cmd: "writeFile",
// //             path: path,
// //             contents: data,
// //             options: options
// //         }
// //     });
// // }
export const CACHE_DIR =
    process.env.NODE_ENV === "development" ? "cache-dev" : "cache";

// /**
//  * Cache an image from a given URL to the specified cache directory.
//  *
//  * @param {string} imageUrl - The URL of the image to be cached.
//  * @returns {Promise<void>} A promise that resolves when the image is successfully cached.
//  * @throws {Error} If there's an error fetching, creating the cache directory, or writing the image data.
//  */
// export async function cacheArtwork(
//     imageData: Uint8Array,
//     albumId: string,
//     format: string
// ) {
//     try {
//         const dataDir = await appDataDir();

//         const imagePath = await getImagePath(dataDir, albumId, format);

//         await createCacheDirectory(dataDir);
//         await writeImageDataToCache(imagePath, imageData);

//         console.log("Image cached successfully:", imagePath);
//         return imagePath;
//     } catch (error) {
//         console.error("Error caching image:", error);
//     }
//     return null;
// }

// /**
//  * Generates the full path to the cache directory.
//  *
//  * @param {string} dataDir - The app data directory
//  * @param {string} imageName - The name of the image.
//  * @returns {string} The full path to the cache directory.
//  */
// const getImagePath = async (
//     dataDir: string,
//     imageName: string,
//     format: string
// ) => {
//     if (format === "image/jpeg") {
//         return `${dataDir}${CACHE_DIR}/${imageName}.jpg`;
//     } else if (format === "image/png") {
//         return `${dataDir}${CACHE_DIR}/${imageName}.png`;
//     }
//     return `${dataDir}${CACHE_DIR}/${imageName}`;
// };

// /**
//  * Creates the cache directory if it doesn't exist.
//  *
//  * @returns {Promise<void>} A promise that resolves when the cache directory is created.
//  */
// const createCacheDirectory = async (dataDir) => {
//     try {
//         await create(`${dataDir}${CACHE_DIR}`);
//     } catch (error) {
//         throw new Error("Error creating cache directory: " + error);
//     }
// };

/**
 * Deletes the cache directory if it exists
 *
 * @returns {Promise<void>} A promise that resolves when the cache directory is deleted.
 */
export const deleteCacheDirectory = async () => {
    try {
        const dataDir = await appDataDir();
        const toDelete = await path.join(dataDir, CACHE_DIR);
        if (await exists(toDelete)) {
            await remove(await path.join(dataDir, CACHE_DIR), {
                recursive: true,
            });
        }
    } catch (error) {
        throw new Error("Error deleting cache directory: " + error);
    }
};

// /**
//  * Writes image data to the cache directory.
//  *
//  * @param {string} imagePath - The path to the image file.
//  * @param {ArrayBuffer} imageData - The image data to write.
//  * @returns {Promise<void>} A promise that resolves when the image data is written to the cache.
//  */
// const writeImageDataToCache = async (
//     imagePath: string,
//     imageData: ArrayBuffer
// ) => {
//     try {
//         await writeBinaryFile(imagePath, new Uint8Array(imageData), {
//             dir: BaseDirectory.AppLocalData
//         });
//     } catch (error) {
//         throw new Error("Error writing image data to cache: " + error);
//     }
// };
