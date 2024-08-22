import { db } from "./data/db";
// import { bottomBarNotification } from "./data/store";
self.window = self;

import type {
    Album,
    ToImport
} from "./App";

export async function handleImport(toImport: ToImport) {
    await db
        .transaction("rw", db.songs, async () => {
            await db.songs
                .bulkPut(
                    toImport.songs.map((s) => {
                        const { artwork, ...rest } = s;
                        return rest;
                    })
                )
                .then(function (lastKey) {
                    console.log(`
                        Done putting ${toImport.songs.length} songs`);
                })
                .catch("BulkError", function (e) {
                    // Explicitly catching the bulkAdd() operation makes those successful
                    // additions commit despite that there were errors.
                    console.error(
                        "Some raindrops did not succeed. However, " +
                            e.failures.length +
                            " raindrops was added successfully"
                    );
                });

            // TODO: Remove
            // const albumsToPut: { [key: string]: Album } = toImport.songs.reduce(
            //     (albums: { [key: string]: Album }, song) => {
            //         const albumPath = song.path.replace(`/${song.file}`, "");

            //         let id = md5(`${albumPath} - ${song.album}`.toLowerCase());
            //         console.log("album path", song.album, id);
            //         if (albums[id] !== undefined) {
            //             albums[id].tracksIds.push(song.id);
            //             albums[id].lossless =
            //                 albums[id].lossless && song.fileInfo.lossless;
            //         } else {
            //             albums[id] = {
            //                 id,
            //                 title: song.album,
            //                 artist: song.artist,
            //                 genre: song.genre,
            //                 path: song.path.replace(`/${song.file}`, ""),
            //                 year: song.year,
            //                 tracksIds: [song.id],
            //                 lossless: song.fileInfo.lossless
            //             };
            //         }
            //         return albums;
            //     },
            //     {}
            // );

            // await db.albums.bulkPut(Object.values(albumsToPut));
        })
        .catch("BulkError", (err) => {
            //
            // Transaction Failed
            //

            console.error(err.stack);
        })
        .then(() => {
            console.log("Transaction completed");
        });
}

async function bulkAlbumPut(albumsToPut: { [key: string]: Album }) {
    await db.albums
        .bulkPut(Object.values(albumsToPut))
        .catch("BulkError", (err) => {
            // Explicitly catching the bulkAdd() operation makes those successful
            // additions commit despite that there were errors.
            console.error(
                "Some album writes did not succeed. However, " +
                    err.failures.length +
                    " albums was added successfully"
            );
        });
}

async function handleMessage(e) {
    switch (e.data.function) {
        case "handleImport":
            await handleImport(e.data.toImport);
            postMessage({
                event: "handleImportDone",
                progress: e.data.toImport.progress,
                done: e.data.toImport.done
            });
            break;
        case "bulkAlbumPut":
            await bulkAlbumPut(e.data.toImport.albums);
            postMessage({
                event: "bulkAlbumPutDone",
                progress: e.data.toImport.progress,
                done: e.data.toImport.done
            });
            break;
        default:
            break;
    }
}

onmessage = (e) => {
    console.log("Message received from main script", e.data);
    handleMessage(e);
    console.log("Posting message back to main script");
};
