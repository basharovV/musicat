import type { ArtistFileItem, ContentItem } from "../App";
import { getContentFileType } from "../utils/FileUtils";
import md5 from "md5";
import { db } from "./db";
import { userSettings } from "./store";
import { get } from "svelte/store";
import { readDir } from "@tauri-apps/plugin-fs";

export async function addScrapbookFile(filePath) {
    console.log("adding item", filePath);
    const contentFileType = getContentFileType(filePath);
    console.log("type", contentFileType);
    const filename = filePath.split("/")?.pop() ?? "";
    const toAdd: ArtistFileItem = {
        id: md5(filePath),
        name: filename,
        tags: [],
        type: "file",
        fileType: contentFileType,
        path: filePath
    };

    await db.scrapbook.put(toAdd);
}

/**
 * On mount, load the scrapbook contents using the location in settings
 *
 */
export async function scanScrapbook() {
    const folderItems: ContentItem[] = [];
    const settings = get(userSettings);
    const itemsToDelete = [];
    const dbItems = await db.scrapbook.toArray();
    if (settings.scrapbookLocation) {
        try {
            let entries;
            try {
                entries = await readDir(settings.scrapbookLocation); 
            } catch (err) {
                throw new Error("Scrapbook location not found");
            }
            for (const entry of entries) {
                const filePath = settings.scrapbookLocation + "/" + entry.name;
                console.log("adding item", filePath);
                const contentFileType = getContentFileType(filePath);
                console.log("type", contentFileType);
                const filename = filePath.split("/")?.pop() ?? "";
                const toAdd: ArtistFileItem = {
                    id: md5(filePath),
                    name: filename,
                    tags: [],
                    type: "file",
                    fileType: contentFileType,
                    path: filePath
                };
                folderItems.push(toAdd);

            }

            // Process items to delete
            for (const item of dbItems) {
                if (!folderItems.find((i) => i.id === item.id)) {
                    itemsToDelete.push(item);
                }
            }

            if (itemsToDelete.length > 0) {
                await db.scrapbook.bulkDelete(itemsToDelete.map((i) => i.id));
            }


            await db
                .transaction("rw", db.scrapbook, async () => {
                    const itemsToRestore = await db.scrapbook.bulkGet(
                        folderItems.map((i) => i.id)
                    );
                    await db.scrapbook.bulkPut(folderItems);
                    await db.scrapbook.bulkUpdate(
                        itemsToRestore.filter((i) => i !== undefined).map((i) => ({
                            key: i.id,
                            changes: {
                                tags: i.tags
                            }
                        }))
                    );
                })
                .catch("BulkError", (err) => {
                    // Transaction Failed
                    console.error(err.stack);
                })
                .then(() => {
                    console.log("Transaction completed");
                });
        } catch (err) {
            console.error(err);
            throw err;
        }
    }
}
