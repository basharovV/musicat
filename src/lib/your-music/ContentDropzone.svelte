<script lang="ts">
    /*
  Would be nice if Tauri supported dropping onto specific elements
  https://github.com/probablykasper/svelte-tauri-filedrop/issues/2
  */
    import md5 from "md5";
    import { invoke } from "@tauri-apps/api/core";
    import type { Event, UnlistenFn } from "@tauri-apps/api/event";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import type { ArtistFileItem, Song, SongProject } from "src/App";
    import { db } from "../../data/db";
    import { isDraggingExternalFiles, userSettings } from "../../data/store";
    import { getContentFileType } from "../../utils/FileUtils";
    import { copyFile } from "@tauri-apps/plugin-fs";
    const appWindow = getCurrentWebviewWindow();

    export let songProject: SongProject;
    let highlightedOption: "add-to-scrapbook" | "add-to-song-project";

    let filedropEvent: Event<any>;
    let unlisten: UnlistenFn;
    async function onDrop(event) {
        filedropEvent = event;
        if (!filedropEvent) return;
        console.log("ondrop", filedropEvent);
        for (const entry of filedropEvent?.payload) {
            // ignore directory at first
            const file = entry.split("/").pop();
            const fileType = getContentFileType(file);
            const contentItem: ArtistFileItem = {
                id: md5(entry),
                name: file,
                tags: [],
                type: "file",
                fileType,
            };
            if (highlightedOption === "add-to-scrapbook") {
                // await db.scrapbook.add(contentItem);
                copyFile(entry, `${$userSettings.scrapbookLocation}/${file}`);
            } else if (
                highlightedOption === "add-to-song-project" &&
                songProject
            ) {
                switch (fileType.type) {
                    case "audio":
                        const song = await invoke<Song>("get_song_metadata", {
                            event: {
                                path: entry,
                                isImport: false,
                                includeFolderArtwork: false,
                            },
                        });
                        if (song) {
                            await db.songProjects.update(songProject, {
                                recordings: [
                                    ...songProject.recordings,
                                    {
                                        recordingType: "master",
                                        song,
                                    },
                                ],
                            });
                        }

                        break;
                    case "video":
                        await db.songProjects.update(songProject, {
                            otherContentItems: songProject.otherContentItems
                                ? [
                                      ...songProject.otherContentItems,
                                      contentItem,
                                  ]
                                : [contentItem],
                        });
                        break;
                }
            }
        }
        unlisten();
        isDraggingExternalFiles.set(false);
    }
    async function startDragListener() {
        if (!unlisten) {
            unlisten = await appWindow.once("tauri://file-drop", onDrop);
        }
    }

    startDragListener();

    function onDragLeave(e) {
        e.preventDefault();
        isDraggingExternalFiles.set(false);
        console.log("drag leave content");
    }
</script>

<div class="dropzone">
    <div
        class="scrapbook"
        class:highlighted={highlightedOption === "add-to-scrapbook"}
        on:dragenter={() => {
            highlightedOption = "add-to-scrapbook";
        }}
    >
        <h2>Add to scrapbook</h2>
        <p>you can attach it to a song later</p>
    </div>
    {#if songProject?.id}
        <div
            class="song"
            class:highlighted={highlightedOption === "add-to-song-project"}
            on:dragenter={() => {
                highlightedOption = "add-to-song-project";
            }}
        >
            <h2>Add to {songProject?.title}</h2>
        </div>
    {/if}
</div>

<style lang="scss">
    .dropzone {
        display: flex;
        align-items: center;
        justify-content: center;
        position: absolute;
        width: 80%;
        height: 80%;
        left: 0;
        right: 0;
        top: 0;
        bottom: 0;
        margin: auto;
        background-color: #242626cc;
        z-index: 20;
        border: 3px dashed rgb(211, 57, 57);
        border-radius: 5px;
        opacity: 1;
        transition: opacity 0.2s ease-in-out;
        /* pointer-events: auto; */
        * {
            /* pointer-events: none; */
        }
        div {
            height: 300px;
            padding: 3em;
            border: 1px solid white;
            background-color: black;
            pointer-events: visibleStroke;
            /* * {
                pointer-events: none;
            } */
            &.highlighted {
                border: 1px solid red;
            }
        }
    }
</style>
