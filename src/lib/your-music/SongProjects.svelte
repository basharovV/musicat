<script lang="ts">
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import type { ArtistProject } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import {
        createSongProject,
        loadSongProject
    } from "../../data/ArtistsToolkitData";
    import { startWatchingSongbookArtistsFolder } from "../../data/FolderWatcher";
    import { userSettings } from "../../data/store";

    export let songProjects: string[];
    export let selectedSongProject;
    export let onSelectSongProject: Function;
    export let artist: ArtistProject;
    let newSongProjectTitle = "";

    async function onCreateSongProject() {
        await createSongProject(artist.name, newSongProjectTitle);
        const createdProject = await loadSongProject(
            artist.name,
            newSongProjectTitle
        );
        selectedSongProject = createdProject;
        onSelectSongProject(createdProject);
        newSongProjectTitle = "";
    }

    let unlistenFolderWatch: UnlistenFn;
    onMount(async () => {
        userSettings.subscribe(async (_) => {
            unlistenFolderWatch && unlistenFolderWatch();
            unlistenFolderWatch = await startWatchingSongbookArtistsFolder(
                artist.name
            );
        });
    });

    onDestroy(() => {
        unlistenFolderWatch && unlistenFolderWatch();
    });
</script>

<container>
    {#if songProjects}
        <ul>
            {#each songProjects as song (song)}
                <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                <li
                    class="song"
                    class:selected={selectedSongProject === song}
                    on:click={() => {
                        selectedSongProject = song;
                        onSelectSongProject(song);
                    }}
                >
                    <p>{song}</p>
                </li>
            {/each}
        </ul>
    {:else}{/if}
    <div class="add">
        <p>+</p>
        <form on:submit|preventDefault={onCreateSongProject}>
            <input
                bind:value={newSongProjectTitle}
                type="text"
                class="artist add"
                placeholder="Add a new song"
            />
        </form>
    </div>
</container>

<style lang="scss">
    container {
        display: flex;
        flex-direction: column;
        gap: 5px;

        ul {
            list-style: none;
            margin-inline-start: 0;
            margin-inline-end: 0;
            margin-block-start: 0;
            margin-block-end: 0;
            padding-inline-start: 0;

            > li {
                margin-left: 0;
                position: relative;
                padding: 0 2em;

                &:hover {
                    background-color: #1f1f1f;
                }

                &.selected {
                    background-color: #4b61dd45;
                }
                /* border-top: 1px solid rgba(255, 255, 255, 0.093); */
                p {
                    margin: 0;
                    line-height: 2em;
                    color: var(--text);
                }

                &:before {
                    content: "–"; /* en dash */
                    position: absolute;
                    opacity: 0.5;
                    top: 2px;
                    margin-left: -1em;
                }
            }
        }

        .add {
            display: inline-flex;
            align-items: center;
            padding: 0 1em;
            p {
                margin: 0;
            }
            input {
                width: 100%;
                align-items: center;
                cursor: default;
                padding: 0.2em 0.5em;
                font-size: 14px;
                outline: none;
                background: none;
                border: none;
                color: var(--text);

                &::placeholder {
                    color: var(--text-inactive);
                }
            }
        }
    }
</style>
