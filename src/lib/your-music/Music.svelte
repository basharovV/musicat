<script lang="ts">
    import type { ArtistProject, SongProject } from "src/App";
    import {
        currentSongProjects,
        songbookSelectedArtist,
    } from "../../data/store";

    import {
        deleteSongProject,
        loadSongProject,
        loadSongProjectsForArtist,
    } from "../../data/ArtistsToolkitData";
    import SongDetails from "../your-music/SongDetails.svelte";
    import SongProjects from "../your-music/SongProjects.svelte";
    let selectedSongProject: SongProject;

    let songProjectSelection: string;

    let isLoading = true;

    async function loadSongbook() {
        isLoading = true;
        $currentSongProjects = await loadSongProjectsForArtist(
            $songbookSelectedArtist.name,
        );
        // Sort the $currentSongProjects alphabetically
        console.log("songbook: $currentSongProjects", $currentSongProjects);
        if (!selectedSongProject) {
            $currentSongProjects?.length &&
                onSelectSong($currentSongProjects[0]);
        }
        isLoading = false;
    }

    $: if ($songbookSelectedArtist) {
        // Set the first song project
        loadSongbook();
    }

    let librarySongsHighlighted = [];

    async function onSelectSong(song: string) {
        selectedSongProject = await loadSongProject(
            $songbookSelectedArtist.name,
            song,
        );
    }

    async function onSelectSongProject(songProject: SongProject | string) {
        if (typeof songProject === "string") {
            await onSelectSong(songProject);
        } else {
            selectedSongProject = songProject;
        }
        loadSongbook();
    }

    async function onDeleteSongProject(songProject: SongProject) {
        // TODO
        await deleteSongProject(songProject.artist, songProject.title);
        loadSongbook();
    }

    function onSongsHighlighted($currentSongProjectsHighlighted) {
        $currentSongProjectsHighlighted.length > 0 &&
            onSelectSong(librarySongsHighlighted[0]);
    }

    $: {
        if ($songbookSelectedArtist?.name !== selectedSongProject?.artist) {
            selectedSongProject = null;
        }
    }
</script>

<div class="container">
    <section class="songs">
        {#if $songbookSelectedArtist}
            <div>
                <h2>my songs</h2>
            </div>
            {#if $songbookSelectedArtist}
                <SongProjects
                    songProjects={$currentSongProjects}
                    artist={$songbookSelectedArtist}
                    {onSelectSongProject}
                    bind:selectedSongProject
                />
            {/if}
        {/if}
    </section>
    <section class="song-details">
        {#if selectedSongProject}
            <SongDetails
                songProject={selectedSongProject}
                artist={$songbookSelectedArtist}
                {onDeleteSongProject}
                {onSelectSong}
            />
        {/if}
    </section>
</div>

<style lang="scss">
    .container {
        height: 100%;
        display: grid;
        grid-template-columns: auto 1fr;
        grid-template-rows: 1fr 1fr;
        gap: 5px;
    }

    .songs {
        grid-row: 1 / 3;
        grid-column: 1;
        background-color: var(--panel-background);
        border-radius: 5px;
        border: 0.7px solid var(--panel-primary-border-accent1);
        div {
            padding: 2em;
        }
    }

    h2 {
        margin: 0;
        font-family: "Snake";
        font-size: 3em;
        color: var(--text);
    }

    .title {
        grid-row: 1;
        grid-column: 1;
        padding: 2em;
    }

    .subtitle {
        display: block;
        margin-top: 1em;
        opacity: 0.5;
    }

    hr {
        border-top: 1px dashed rgba(255, 255, 255, 0.093);
        border-bottom: none;
        border-left: none;
        border-right: none;
    }

    section {
        p {
            opacity: 0.5;
        }
    }
    .song-details {
        grid-row: 1 / 3;
        grid-column: 2;
        height: 100%;
        background-color: var(--panel-background);
        border-radius: 5px;
        border: 0.7px solid var(--panel-primary-border-accent1);
        div {
            padding: 2em;
        }
    }
</style>
