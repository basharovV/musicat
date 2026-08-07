import type {
    ArtistFileItem,
    ArtistProject,
    ContentItem,
    SongProject
} from "../App";
import { getContentFileType } from "../utils/FileUtils";
import md5 from "md5";
import { invoke } from "@tauri-apps/api/core";
import { userSettings } from "./store";
import { get } from "svelte/store";
import { mkdir, remove, rename, writeTextFile } from "@tauri-apps/plugin-fs";
import fm from "front-matter";

// `readDir`/`readTextFile`/`exists` from `@tauri-apps/plugin-fs` block synchronously inside their
// Rust command handlers with no `spawn_blocking` (see the comment on `read_dir_blocking` et al. in
// `src-tauri/src/files.rs`) -- for a cloud-sync/placeholder-backed songbook or scrapbook folder
// (e.g. a ProtonDrive mount), that can stall the whole app, not just this call, since it occupies
// a shared Tauri command-dispatch worker thread for as long as the download takes. These three
// custom commands do the same `std::fs` calls via `spawn_blocking` instead, so a slow one can't
// starve everything else.
interface BlockingSafeDirEntry {
    name: string;
    isDirectory: boolean;
}
async function readDirBlocking(path: string): Promise<BlockingSafeDirEntry[]> {
    return await invoke<BlockingSafeDirEntry[]>("read_dir_blocking", { path });
}
async function existsBlocking(path: string): Promise<boolean> {
    return await invoke<boolean>("path_exists_blocking", { path });
}
async function readTextFileBlocking(path: string): Promise<string> {
    return await invoke<string>("read_text_file_blocking", { path });
}

export async function addScrapbookFile(filePath): Promise<ContentItem> {
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

    return await invoke<ContentItem>("add_scrapbook_item", { item: toAdd });
}

/**
 * On mount (or a rescan), reconciles the scrapbook table against the files
 * currently in `settings.scrapbookLocation`: adds newly-discovered files,
 * drops rows for files no longer present, and preserves any tags already on
 * an existing item — done in `sync_scrapbook_items` (Rust), mirroring what
 * this function used to do against Dexie directly.
 */
export async function scanScrapbook(): Promise<ContentItem[]> {
    const settings = get(userSettings);
    if (!settings.scrapbookLocation) return [];

    const folderItems: ArtistFileItem[] = [];
    try {
        let entries: BlockingSafeDirEntry[];
        try {
            entries = await readDirBlocking(settings.scrapbookLocation);
        } catch (err) {
            throw new Error("Scrapbook location not found");
        }
        for (const entry of entries.filter((f) => !f.name.startsWith("."))) {
            // Plain string concatenation, not path.join -- that's an IPC round-trip per call,
            // and this runs once per file in the scrapbook folder.
            const filePath = `${settings.scrapbookLocation}/${entry.name}`;
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

        return await invoke<ContentItem[]>("sync_scrapbook_items", { discovered: folderItems });
    } catch (err) {
        console.error(err);
        throw err;
    }
}

/**
 * Reconciles `artist_projects` against the songbook's artist folders
 * (`sync_artist_projects`, Rust) and returns the merged list -- unlike the
 * bare folder scan this used to be, each `ArtistProject` now carries a real
 * `id` and its `members`, since the DB row is created (if missing) as part
 * of this call.
 */
export async function loadArtistsFromSongbook(): Promise<ArtistProject[] | undefined> {
    const songbookLocation = get(userSettings).songbookLocation;
    if (!songbookLocation) return;

    try {
        const dirContents = await readDirBlocking(songbookLocation);
        const artistFolders = dirContents.filter((item) => item.isDirectory);
        const discoveredNames = artistFolders.map((folder) => folder.name);
        return await invoke<ArtistProject[]>("sync_artist_projects", { discoveredNames });
    } catch (error) {
        console.error("Error loading artists from songbook location:", error);
    }
}

/**
 * Reconciles `song_projects` for this artist against its song folders
 * (`sync_song_projects_for_artist`, Rust), and returns the titles -- same
 * contract as before this change, folder scan plus DB reconciliation instead
 * of a bare folder scan.
 */
export async function loadSongProjectsForArtist(
    artist: ArtistProject
): Promise<string[] | undefined> {
    const songbookLocation = get(userSettings).songbookLocation;
    if (!songbookLocation || artist?.id == null) return;

    try {
        const artistPath = `${songbookLocation}/${artist.name}`;
        const dirContents = await readDirBlocking(artistPath);
        const songFolders = dirContents.filter((item) => item.isDirectory);
        // Plain string concatenation (matching the rest of this file, e.g.
        // loadSongProject/createSongProject) rather than path.join -- that's an IPC round-trip
        // per call, and this used to fire one per song folder concurrently via Promise.all,
        // adding avoidable IPC load on top of the readDir above.
        const discovered: [string, string][] = songFolders.map((folder) => [
            folder.name,
            `${artistPath}/${folder.name}/${folder.name}.txt`
        ]);
        const titles = await invoke<string[]>("sync_song_projects_for_artist", {
            artistId: artist.id,
            discovered
        });
        titles.sort();
        return titles;
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
        let metaFileExists = await existsBlocking(songFilePath);

        if (!metaFileExists) {
            // Create the file
            await writeTextFile(songFilePath, "");
        }

        let songFileContents = "";

        try {
            songFileContents = await readTextFileBlocking(songFilePath);
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

/**
 * Loads a song project: parses the `.txt` file (frontmatter + lyrics remain
 * file-sourced, unchanged), then find-or-creates the corresponding
 * `song_projects` row (`ensure_song_project`, Rust) and merges in the fields
 * only SQLite holds -- `id`, `recordings`, `otherContentItems`, `songId`.
 * The find-or-create (rather than a plain lookup) matters right after
 * `createSongProject`, before any list-level `loadSongProjectsForArtist`
 * scan has had a chance to reconcile the new folder.
 */
export async function loadSongProject(
    artist: ArtistProject,
    songName: string
): Promise<SongProject | undefined> {
    const songbookLocation = get(userSettings).songbookLocation;
    if (!songbookLocation || artist?.id == null) return;

    try {
        const songFilePath = `${songbookLocation}/${artist.name}/${songName}/${songName}.txt`;
        let songProject: SongProject = {
            songFilepath: songFilePath,
            title: songName,
            artist: artist.name,
            musicComposedBy: [],
            lyricsWrittenBy: [],
            recordings: [],
            otherContentItems: []
        };

        const songFileContents = await readSongbookFile(songFilePath);
        const { frontmatter, chordmark } = await extractFrontmatterAndChordmark(
            songFileContents
        );

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

        const dbRow = await invoke<any>("ensure_song_project", {
            artistId: artist.id,
            title: songName,
            songFilepath: songFilePath
        });
        songProject.id = dbRow.id;
        songProject.artistId = dbRow.artistId;
        songProject.recordings = dbRow.recordings ?? [];
        songProject.otherContentItems = dbRow.otherContentItems ?? [];
        songProject.songId = dbRow.songId ?? undefined;

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
    const songPathExists = await existsBlocking(songPath);
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

/**
 * Renames a song project's folder/file, then updates its `song_projects` row
 * in place (`rename_song_project`, Rust) so its `recordings`/
 * `otherContentItems` stay associated with the same row instead of being
 * orphaned under a row for the old title.
 */
export async function renameSongProject(
    songProject: SongProject,
    newSongName: string
) {
    const songbookLocation = get(userSettings).songbookLocation;
    const artistName = songProject.artist;
    const oldSongName = songProject.title;
    const oldSongFilePath = `${songbookLocation}/${artistName}/${oldSongName}/${oldSongName}.txt`;
    const newSongFilePath = `${songbookLocation}/${artistName}/${oldSongName}/${newSongName}.txt`;
    await rename(oldSongFilePath, newSongFilePath);
    const oldSongPath = `${songbookLocation}/${artistName}/${oldSongName}`;
    const newSongPath = `${songbookLocation}/${artistName}/${newSongName}`;
    await rename(oldSongPath, newSongPath);

    if (songProject.id != null) {
        const finalSongFilePath = `${songbookLocation}/${artistName}/${newSongName}/${newSongName}.txt`;
        await invoke("rename_song_project", {
            id: songProject.id,
            newTitle: newSongName,
            newSongFilepath: finalSongFilePath
        });
    }
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

    const { chordmark } = await extractFrontmatterAndChordmark(
        songFileContents
    );
    const newContents = newFrontmatter + chordmark;
    await writeTextFile(songProject.songFilepath, newContents);

    await saveSongProject(songProject);
}

export async function deleteSongProject(artistName: string, songName: string) {
    const songbookLocation = get(userSettings).songbookLocation;
    const songPath = `${songbookLocation}/${artistName}/${songName}`;
    await remove(songPath, { recursive: true });
    // The song_projects row is dropped lazily, the next time
    // loadSongProjectsForArtist reconciles this artist's folders.
}

/**
 * Persists the DB-owned fields of a song project (`recordings`,
 * `otherContentItems`, plus the frontmatter-mirrored fields already in
 * `songProject`) via `update_song_project` (Rust) -- a full replace, same
 * semantics `db.songProjects.put(songProjectClone)` used to have, just
 * actually reaching a row that gets read back now. Requires `songProject.id`
 * (set by `loadSongProject`/`ensure_song_project`).
 */
export async function saveSongProject(songProject: SongProject): Promise<void> {
    if (songProject.id == null) {
        console.error("saveSongProject called before the song project was tracked (no id)");
        return;
    }
    await invoke("update_song_project", {
        id: songProject.id,
        update: {
            album: songProject.album ?? null,
            key: songProject.key ?? null,
            bpm: songProject.bpm ?? null,
            musicComposedBy: songProject.musicComposedBy ?? [],
            lyricsWrittenBy: songProject.lyricsWrittenBy ?? [],
            artworkFilePath: songProject.artworkFilePath ?? null,
            lyrics: songProject.lyrics ?? null,
            recordings: songProject.recordings ?? [],
            otherContentItems: songProject.otherContentItems ?? [],
            songId: songProject.songId ?? null
        }
    });
}
