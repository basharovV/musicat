import { open } from "@tauri-apps/api/dialog";
import { BaseDirectory, readDir } from "@tauri-apps/api/fs";
import { appDir } from "@tauri-apps/api/path";
import "iconify-icon";
import md5 from "md5";
import { db } from "./db";

// TODO metadata
import * as musicMetadata from "music-metadata-browser";
import { songsJustAdded } from "./store";
import { get } from "svelte/store";

let songs: Song[] = [];

export async function addSong(filePath: string, fileName: string) {
  if (!filePath.match(/\.(mp3|ogg|flac|aac|wav)$/)) {
    return;
  }
  const metadata = await musicMetadata.fetchFromUrl(
    window.__TAURI__.tauri.convertFileSrc(filePath)
  );
  console.log(metadata);
  const fileHash = md5(filePath);

  try {
    const songToAdd = {
      id: fileHash,
      path: filePath,
      file: fileName,
      title: metadata.common.title || fileName || "",
      artist: metadata.common.artist || "",
      album: metadata.common.album || "",
      year: metadata.common.year || 0,
      genre: metadata.common.genre || [],
    };
    await db.songs.add(songToAdd, "id");
    songsJustAdded.update((songs) => {
      songs.push(songToAdd);
      return songs;
    });
  } catch (e) {
    console.error(e);
  }
}

function processEntries(entries) {
  for (const entry of entries) {
    console.log(`Entry: ${entry.name}`);
    if (entry.children) {
      processEntries(entry.children.filter((e) => !e.name.startsWith(".")));
    } else {
      addSong(entry.path, entry.name);
    }
  }
}

export async function addFolder(folderPath) {
  const entries = await readDir(folderPath, {
    dir: BaseDirectory.App,
    recursive: true,
  });
  processEntries(entries.filter((e) => !e.name.startsWith(".")));
}

export async function openTauriImportDialog() {
  // Open a selection dialog for directories
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: await appDir(),
  });
  if (Array.isArray(selected)) {
    // user selected multiple directories
  } else if (selected === null) {
    // user cancelled the selection
  } else {
    console.log("selected", selected);
    // user selected a single directory
    addFolder(selected);
  }
}
