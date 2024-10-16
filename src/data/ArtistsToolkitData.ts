import type {
    ArtistFileItem,
    ArtistProject,
    ContentItem,
    SongProject
} from "../App";
import { getContentFileType } from "../utils/FileUtils";
import md5 from "md5";
import { db } from "./db";
import { userSettings } from "./store";
import { get } from "svelte/store";
import {
    type DirEntry,
    exists,
    mkdir,
    readDir,
    readTextFile,
    remove,
    rename,
    writeTextFile
} from "@tauri-apps/plugin-fs";
import fm from "front-matter";
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
            let entries: DirEntry[];
            try {
                entries = await readDir(settings.scrapbookLocation);
            } catch (err) {
                throw new Error("Scrapbook location not found");
            }
            for (const entry of entries.filter(
                (f) => !f.name.startsWith(".")
            )) {
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
                        itemsToRestore
                            .filter((i) => i !== undefined)
                            .map((i) => ({
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

export async function loadArtistsFromSongbook() {
    const songbookLocation = get(userSettings).songbookLocation;
    if (!songbookLocation) return;

    try {
        const dirContents = await readDir(songbookLocation);
        const artistFolders = dirContents.filter((item) => item.isDirectory);
        const artists: ArtistProject[] = artistFolders.map((folder) => ({
            name: folder.name,
            members: [],
            profilePhoto: null
        }));
        return artists;
    } catch (error) {
        console.error("Error loading artists from songbook location:", error);
    }
}

export async function loadSongProjectsForArtist(artistName: string) {
    const songbookLocation = get(userSettings).songbookLocation;
    if (!songbookLocation) return;

    try {
        const dirContents = await readDir(songbookLocation + "/" + artistName);
        const songFolders = dirContents.filter((item) => item.isDirectory);
        const songs: string[] = songFolders.map((folder) => folder.name);
        songs.sort();
        return songs;
    } catch (error) {
        console.error("Error loading songs from songbook location:", error);
    }
}
function parseFileContent(fileContent: string): {
    frontmatter: string | null;
    chordmark: string;
} {
    // Regular expression to match frontmatter enclosed in '---'
    const frontmatterRegex = /^---\n([\s\S]*?)\n---\n/;

    // Try to match the frontmatter
    const match = fileContent.match(frontmatterRegex);

    if (match) {
        // Extract frontmatter and the remaining chordmark content
        const frontmatter = match[1];
        const chordmark = fileContent.substring(match.index! + match[0].length);
        return { frontmatter, chordmark };
    } else {
        // No frontmatter found, so the whole file content is chordmark
        return { frontmatter: null, chordmark: fileContent };
    }
}

async function readSongbookFile(songFilePath: string) {
    try {
        let metaFileExists = await exists(songFilePath);

        if (!metaFileExists) {
            // Create the file
            await writeTextFile(songFilePath, "");
        }

        let songFileContents = "";

        try {
            songFileContents = await readTextFile(songFilePath);
            console.log("songFileContents", songFileContents);
            return songFileContents;
        } catch (err) {
            console.error(err);
        }
    } catch (error) {
        console.error("Error loading song from songbook location:", error);
    }
}

async function extractFrontmatterAndChordmark(songFileContents: string) {
    // Use front-matter to extract the metadata. We can use the bodyBegin as the line where the chordmark begins

    const frontmatterRegex = /^(---\s*\n[\s\S]*?\n---\s*)/;

    // Try to match the frontmatter
    const match = songFileContents.match(frontmatterRegex);
    let chordmark = "";
    let frontmatter;
    console.log("match", match);
    if (match) {
        // Extract frontmatter and the remaining chordmark content
        frontmatter = fm(match[1]);
        chordmark = songFileContents.substring(match.index! + match[0].length);
    } else {
        // No frontmatter found, so the whole file content is chordmark
        chordmark = songFileContents;
    }

    return { frontmatter, chordmark };
}

async function insertLyricsToSong(songFileContents: string, lyrics: string) {
    const frontmatterRegex = /^(---\s*\n[\s\S]*?\n---\s*)/;
    // Try to match the frontmatter
    const match = songFileContents.match(frontmatterRegex);
    let frontmatter;
    // console.log("match", match);
    if (match) {
        // Extract frontmatter and the remaining chordmark content
        frontmatter = fm(match[1]);
        return match[1].trim() + `\n\n${lyrics.trim()}`;
    } else {
        // No frontmatter found, so the whole file content is chordmark
        return lyrics;
    }
}

export async function loadSongProject(artistName: string, songName: string) {
    const songbookLocation = get(userSettings).songbookLocation;
    if (!songbookLocation) return;

    try {
        const songFilePath = `${songbookLocation}/${artistName}/${songName}/${songName}.txt`;
        let songProject: SongProject = {
            songFilepath: songFilePath,
            title: songName,
            artist: artistName,
            musicComposedBy: [],
            lyricsWrittenBy: [],
            recordings: [],
            otherContentItems: []
        };

        const songFileContents = await readSongbookFile(songFilePath);
        const { frontmatter, chordmark } =
            await extractFrontmatterAndChordmark(songFileContents);

        console.log("frontmatter", frontmatter);
        if (frontmatter?.attributes) {
            songProject.album = frontmatter.attributes.album;
            songProject.musicComposedBy = frontmatter.attributes.composer
                ? [frontmatter.attributes.composer]
                : [];
            songProject.lyricsWrittenBy = frontmatter.attributes.lyricist
                ? [frontmatter.attributes.lyricist]
                : [];
            songProject.key = frontmatter.attributes.key;
            songProject.bpm = frontmatter.attributes.bpm;
        }

        if (chordmark?.length) {
            songProject.lyrics = chordmark;
        }
        return songProject;
    } catch (error) {
        console.error("Error loading song from songbook location:", error);
    }
}

export async function writeChordMarkToSongProject(
    songProject: SongProject,
    lyrics: string
) {
    // ChordMark goes after frontmatter, so we need to know which line to write from

    const songFileContents = await readSongbookFile(songProject.songFilepath);
    const newContents = await insertLyricsToSong(songFileContents, lyrics);

    await writeTextFile(songProject.songFilepath, newContents);
}

export async function createSongProject(artistName: string, songName: string) {
    const songbookLocation = get(userSettings).songbookLocation;
    const songPath = `${songbookLocation}/${artistName}/${songName}`;
    const songPathExists = await exists(songPath);
    if (!songPathExists) {
        await mkdir(songPath, { recursive: true });
    }
    const songFilePath = `${songbookLocation}/${artistName}/${songName}/${songName}.txt`;

    await writeTextFile(
        songFilePath,
        `---\ncomposer: ${artistName}\nlyricist: ${artistName}\n---`,
        {
            create: true
        }
    );
}

export async function renameSongProject(
    artistName: string,
    oldSongName: string,
    newSongName: string
) {
    const songbookLocation = get(userSettings).songbookLocation;
    const oldSongFilePath = `${songbookLocation}/${artistName}/${oldSongName}/${oldSongName}.txt`;
    const newSongFilePath = `${songbookLocation}/${artistName}/${oldSongName}/${newSongName}.txt`;
    await rename(oldSongFilePath, newSongFilePath);
    const oldSongPath = `${songbookLocation}/${artistName}/${oldSongName}`;
    const newSongPath = `${songbookLocation}/${artistName}/${newSongName}`;
    await rename(oldSongPath, newSongPath);
}

export async function saveFrontmatterToSongProject(songProject: SongProject) {
    const songFileContents = await readSongbookFile(songProject.songFilepath);
    const attributes = {
        composer: songProject.musicComposedBy?.length
            ? songProject.musicComposedBy[0]
            : null,
        lyricist: songProject.lyricsWrittenBy?.length
            ? songProject.lyricsWrittenBy[0]
            : null,
        album: songProject.album,
        bpm: songProject.bpm,
        key: songProject.key
    };

    // Create frontmatter string
    const newFrontmatter = `---\n${Object.entries(attributes)
        .filter(([key, value]) => value)
        .map(([key, value]) => `${key}: ${value}`)
        .join("\n")}\n---\n`;

    const { chordmark } =
        await extractFrontmatterAndChordmark(songFileContents);
    const newContents = newFrontmatter + chordmark;
    await writeTextFile(songProject.songFilepath, newContents);
}
export async function deleteSongProject(artistName: string, songName: string) {
    const songbookLocation = get(userSettings).songbookLocation;
    const songPath = `${songbookLocation}/${artistName}/${songName}`;
    await remove(songPath, { recursive: true });
}
