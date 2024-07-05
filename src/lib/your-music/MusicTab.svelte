<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";
    import { audioDir } from "@tauri-apps/api/path";
    import type { Song, SongProject, SongProjectRecording } from "src/App";
    import { currentSong, songDetailsUpdater } from "../../data/store";
    import { db } from "../../data/db";
    import AudioPlayer from "../player/AudioPlayer";
    import { fly } from "svelte/transition";
    import { quadInOut, quadOut } from "svelte/easing";
    import { flip } from "svelte/animate";
    import Icon from "../ui/Icon.svelte";

    export let recordings: SongProjectRecording[];
    export let songProject: SongProject;
    export let song: Song;
    export let addRecording;
    export let deleteRecording;
    export let showDropPlaceholder = false;
    export let onSelectSong;

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

    function playSong(recording) {
        AudioPlayer.playSong(recording.song);
    }

    async function createProject() {
        const songProjectId = (await db.songProjects.add(
            songProject
        )) as number;
        if (song) {
            song.songProjectId = songProjectId;
            await db.songs.put(song);
            onSelectSong && onSelectSong(song);
        }
    }
</script>

<container>
    {#if recordings}
        {#each recordings as recording, idx (idx)}
            <div
                in:fly={{ duration: 150, easing: quadOut }}
                class:playing={$currentSong?.file === recording.song.file}
                class="recording"
                on:click={() => {
                    playSong(recording);
                }}
            >
                <Icon icon="fe:play" />
                <p>{recording.song.file}</p>
                {#if isProject}
                    <div class="delete">
                        <Icon
                            icon="ant-design:delete-outlined"
                            onClick|preventDefault|stopPropagation={() => {
                                deleteRecording(idx);
                            }}
                        />
                    </div>
                {/if}
            </div>
        {/each}

        {#if showDropPlaceholder}
            <div class="recording drop-placeholder">
                <Icon icon="fe:play" />
                <p>Drop files here</p>
            </div>
        {/if}
    {/if}
    {#if isProject}
        <button on:click={openImportDialog}>Add recordings</button>
        <p class="prompt">
            Or drop a file here.<br /> You can also drag files in here from the Scrapbook.
        </p>
    {:else}
        <p>
            This song is in your library but doesn't have a project. To attach
            more music files, lyrics and other media, create a project:
        </p>

        <button on:click={createProject}>Create a project</button>
    {/if}
</container>

<style lang="scss">
    $playing_text_color: #00ddff;
    $selected_color: #5123dd;
    container {
        display: block;
        padding: 3em 2em;

        .recording {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
            border-radius: 4px;
            padding: 0.2em 0.5em;
            margin: 0.5em 0;

            &.drop-placeholder {
                border: 1px dashed white;
            }

            p {
                margin: 0;
                text-align: left;
                flex-grow: 2;
                cursor: default;
            }

            .delete {
                visibility: hidden;
            }

            &.playing {
                background: $selected_color;
                color: $playing_text_color;
                &:hover {
                    background: $selected_color;
                    color: $playing_text_color;
                }
            }
            &:hover {
                background-color: #7c787823;

                .delete {
                    visibility: visible;
                    padding: 5px;
                    z-index: 2;
                    &:hover {
                        color: rgb(224, 72, 72);
                        background-color: rgba(0, 0, 0, 0.457);
                    }
                }
            }
        }

        .prompt {
            opacity: 0.5;
        }

        button {
            margin-top: 1em;
        }
    }
</style>
