<script lang="ts">
    import type { ArtistProject, Song, SongProject } from "src/App";
    import { db } from "../../data/db";
    import { autoWidth } from "../../utils/AutoWidth";

    export let songProjects: SongProject[];
    export let selectedSongProject;
    export let onSelectSongProject: Function;
    export let artistId: ArtistProject;
    let newSongProjectTitle = "";

    async function onCreateSongProject() {
        const createdProjectId = await db.songProjects.add({
            title: newSongProjectTitle,
            artist: artistId.name,
            album: "",
            musicComposedBy: [artistId.name], // Multiple people
            lyricsWrittenBy: [], // Multiple people,
            lyrics: "",
            recordings: [],
            otherContentItems: []
        });
        const createdProject = await db.songProjects.get(createdProjectId);
        selectedSongProject = createdProject;
        onSelectSongProject(createdProject);

        newSongProjectTitle = "";
    }
</script>

<container>
    {#if songProjects}
        <ul>
            {#each songProjects as song (song.id)}
                <li
                    class="song"
                    class:selected={selectedSongProject?.id === song.id}
                    on:click={() => {
                        selectedSongProject = song;
                        onSelectSongProject(song);
                    }}
                >
                    <p>{song.title}</p>
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
                    color:#bbb9b9;
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
                color: #c8aafb;

                &::placeholder {
                    color: rgb(105, 105, 105);
                }
            }
        }
    }
</style>
