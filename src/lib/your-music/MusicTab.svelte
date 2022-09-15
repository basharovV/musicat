<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";
    import { audioDir } from "@tauri-apps/api/path";
    import type { SongProject, SongProjectRecording } from "src/App";
    import { db } from "../../data/db";
    import { getSongFromFile } from "../../data/LibraryImporter";
    import AudioPlayer from "../AudioPlayer";

    export let recordings: SongProjectRecording[];
    export let songProject: SongProject;
    $: isProject = songProject?.id !== undefined;

    export async function openImportDialog() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: await audioDir()
        });
        if (Array.isArray(selected)) {
            // user selected multiple files
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single directory
            addRecording(selected);
        }
    }

    async function addRecording(filePath) {
        const filename = filePath.split("/").pop();
        const song = await getSongFromFile(filePath, filename);
        if (!recordings) recordings = [];
        recordings.push({
            recordingType: "master",
            song
        });
        await db.songProjects.update(songProject.id, {
            recordings
        });
        recordings = recordings;
    }

    function playSong(recording) {
        AudioPlayer.playSong(recording.song);
    }

    async function createProject() {
        await db.songProjects.add(songProject);
    }
</script>

<container>
    {#if recordings}
        {#each recordings as recording}
            <div
                class="recording"
                on:click={() => {
                    playSong(recording);
                }}
            >
                <iconify-icon icon="fe:play" />
                <p>{recording.song.file}</p>
            </div>
        {/each}
    {/if}
    {#if isProject}
        <button on:click={openImportDialog}>Add recording</button>
    {:else}
        <p>
            This song is in your library but doesn't have a project. To attach
            more music files, lyrics and other media, create a project:
        </p>

        <button on:click={createProject}>Create a project</button>
    {/if}
</container>

<style lang="scss">
    container {
        display: block;
        padding: 3em 2em;

        .recording {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 10px;
            border-radius: 4px;
            padding: 0.2em 0.5em;
            margin: 0.5em 0;
            p {
                margin: 0;
            }
            &:hover {
                background-color: #7c787823;
            }
        }

        button {
            margin-top: 1em;
        }
    }
</style>
