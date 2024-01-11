<script lang="ts">
    import { liveQuery } from "dexie";
    import type { ArtistProject, Song, SongProject } from "src/App";
    import { db } from "../../data/db";
    import { selectedArtistId } from "../../data/store";

    import Library from "../library/Library.svelte";
    import SongDetails from "../your-music/SongDetails.svelte";
    import SongProjects from "../your-music/SongProjects.svelte";
    let selectedSong: Song;
    let selectedSongProject: SongProject;
    let songProjectSelection;

    // selectedArtistId.subscribe(async (artistId) => {
    //     selectedArtist = await db.artistProjects.get(artistId);
    // });
    export let selectedArtist;

    $: songs = liveQuery<Song[]>(async () => {
        let resultsArray = [];
        if ($selectedArtistId) {
            const results = await db.songs
                .where("artist")
                .equalsIgnoreCase($selectedArtistId);
            resultsArray = await results.toArray();
        }
        isLoading = false;
        return resultsArray;
    });

    let isLoading = true;

    $: songProjects = liveQuery<SongProject[]>(async () => {
        let resultsArray = [];
        if ($selectedArtistId) {
            const results = await db.songProjects
                .where("artist")
                .equalsIgnoreCase($selectedArtistId);
            resultsArray = await results.toArray();
        }
        return resultsArray;
    });

    let librarySongsHighlighted = [];

    function onSelectSong(song: Song) {
        selectedSong = song;
        const existingSongProject = $songProjects.find(
            (s) => s.id === song?.songProjectId
        );
        if (existingSongProject) {
            selectedSongProject = existingSongProject;
            songProjectSelection = selectedSongProject;
        } else {
            selectedSongProject = {
                title: song?.title ?? "",
                artist: song?.artist ?? "",
                album: song?.album ?? "",
                musicComposedBy: [],
                lyricsWrittenBy: [],
                lyrics: "",
                recordings: [
                    {
                        recordingType: "master",
                        song: song
                    }
                ],
                otherContentItems: []
            };
            songProjectSelection = null;
            console.log("library songs", song);
        }
    }

    function onSelectSongProject(songProject: SongProject) {
        selectedSongProject = songProject;
        selectedSong = null;
        if (selectedSongProject?.recordings) {
            // Also highlighted the corresponding song if any
            const foundSong = $songs?.find(
                (s) => s?.songProjectId === selectedSongProject?.id
            );
            if (foundSong) {
                librarySongsHighlighted = [foundSong];
            } else {
                librarySongsHighlighted = [];
            }
        } else {
            librarySongsHighlighted = [];
        }
    }

    async function onDeleteSongProject(songProject: SongProject) {
        console.log("deieting ", songProject);

        const currentIndex = $songProjects?.findIndex(
            (s) => s.id === songProject.id
        );
        await db.songProjects.delete(songProject.id);
        if (currentIndex > 0 && currentIndex < $songProjects.length - 1) {
            selectedSongProject = $songProjects[currentIndex - 1];
        } else if (currentIndex === 1) {
            selectedSongProject = $songProjects[0];
        } else if (currentIndex === $songProjects.length - 1) {
            selectedSongProject = $songProjects[$songProjects.length - 2];
        }
        songProjectSelection = selectedSongProject;
    }

    function onSongsHighlighted(songsHighlighted) {
        songsHighlighted.length > 0 && onSelectSong(librarySongsHighlighted[0]);
    }

    $: {
        if ($selectedArtistId !== selectedSongProject?.artist) {
            selectedSongProject = null;
        }
    }
</script>

<div class="container">
    <section class="songs">
        {#if $selectedArtist}
            <div>
                <h2>my songs</h2>
            </div>
            <SongProjects
                songProjects={$songProjects}
                artistId={$selectedArtist}
                {onSelectSongProject}
                bind:selectedSongProject={songProjectSelection}
            />
            <hr />
            {#if $songs && $songs?.length}
                <library>
                    <div>
                        <h2>in library</h2>
                    </div>
                    <Library
                        theme="outline"
                        bind:songsHighlighted={librarySongsHighlighted}
                        allSongs={songs}
                        {isLoading}
                        isSmartQueryEnabled={false}
                        fields={[
                            { name: "Title", value: "title" },
                            { name: "Album", value: "album" }
                        ]}
                        {onSongsHighlighted}
                    />
                </library>
            {/if}
        {/if}
    </section>
    <section class="song-details">
        <SongDetails
            songProject={selectedSongProject}
            artist={$selectedArtist}
            {onDeleteSongProject}
            song={selectedSong}
            {onSelectSong}
        />
    </section>
</div>

<style lang="scss">
    .container {
        height: 100%;
        display: grid;
        grid-template-columns: 1fr 1fr;
        grid-template-rows: 1fr 1fr;
    }

    .songs {
        grid-row: 1 / 3;
        grid-column: 1;
        border-right: 1px dashed rgba(255, 255, 255, 0.093);
        div {
            padding: 2em;
        }
    }

    h2 {
        margin: 0;
        font-family: "Snake";
        font-size: 3em;
        color: #bbb9b9;
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
        background-color: #242026b3;
        p {
            opacity: 0.5;
        }
    }
    .song-details {
        grid-row: 1 / 3;
        grid-column: 2;
        height: 100%;
        div {
            padding: 2em;
        }
    }
</style>
