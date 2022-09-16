<script lang="ts">
    import { liveQuery } from "dexie";
    import type { ArtistProject, Song, SongProject } from "src/App";
    import { isDraggingExternalFiles } from "../../data/store";
    import { db } from "../../data/db";

    import Library from "../Library.svelte";
    import ContentDropzone from "../your-music/ContentDropzone.svelte";
    import Scrapbook from "../your-music/Scrapbook.svelte";
    import SongDetails from "../your-music/SongDetails.svelte";
    import SongProjects from "../your-music/SongProjects.svelte";
    import YourArtists from "../your-music/YourArtists.svelte";
    let selectedArtist: ArtistProject;
    let selectedSong: Song;
    let selectedSongProject: SongProject;
    let songProjectSelection;
    $: songs = liveQuery<Song[]>(async () => {
        let resultsArray = [];
        if (selectedArtist) {
            const results = await db.songs
                .where("artist")
                .equalsIgnoreCase(selectedArtist.name);
            resultsArray = await results.toArray();
        }
        return resultsArray;
    });

    $: songProjects = liveQuery<SongProject[]>(async () => {
        let resultsArray = [];
        if (selectedArtist) {
            const results = await db.songProjects
                .where("artist")
                .equalsIgnoreCase(selectedArtist.name);
            resultsArray = await results.toArray();
        }
        return resultsArray;
    });

    let librarySongsHighlighted = [];

    function onSelectSong(song: Song) {
        selectedSong = song;
        const existingSongProject = $songProjects.find(
            (s) => s.id === song.songProjectId
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
        if (selectedArtist?.name !== selectedSongProject?.artist) {
            selectedSongProject = null;
        }
    }
</script>

<container>
    {#if $isDraggingExternalFiles}
        <ContentDropzone songProject={selectedSongProject} />
    {/if}
    <header>
        <h2>Artists</h2>
        <YourArtists bind:selectedArtist />
    </header>
    <section class="scrapbook">
        <div>
            <h2>Scrapbook</h2>
        </div>
        <div class="content">
            <Scrapbook />
        </div>
    </section>
    <section class="songs">
        <div>
            <h2>Songs in progress</h2>
        </div>
        <SongProjects
            songProjects={$songProjects}
            artist={selectedArtist}
            {onSelectSongProject}
            bind:selectedSongProject={songProjectSelection}
        />
        <hr />
        <div>
            <h2>In Library</h2>
        </div>
        <Library
            theme="outline"
            bind:songsHighlighted={librarySongsHighlighted}
            {songs}
            isSmartQueryEnabled={false}
            fields={[
                { name: "Title", value: "title" },
                { name: "Album", value: "album" }
            ]}
            {onSongsHighlighted}
        />
    </section>
    <section class="song-details">
        <SongDetails
            songProject={selectedSongProject}
            {onDeleteSongProject}
            song={selectedSong}
            {onSelectSong}
        />
    </section>

    <img class="bulb" src="images/bulby_bulb.png" alt="" />
</container>

<style lang="scss">
    .bulb {
        position: fixed;
        bottom: 0;
        right: 4em;
        width: 150px;
        z-index: 0;
        display: flex;
        opacity: 0.3;
    }
    .arrow {
        transform: rotate(90deg);
        width: 20px;
        top: 5px;
        position: relative;
    }

    container {
        text-align: left;
        display: grid;
        grid-template-columns: 350px 5fr 4fr;
        grid-template-rows: auto 1fr;
        height: 100vh;
        overflow: hidden;
        h2 {
            margin: 0;
            font-family: "Snake";
            font-size: 3em;
            color: #bbb9b9;
        }

        p {
            opacity: 0.5;
        }
        section {
            background-color: #242026b3;
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

        header {
            grid-row: 1;
            grid-column: 1 / 3;
            display: flex;
            flex-direction: row;
            gap: 2em;
            align-items: center;
            padding: 0.7em 2em;
            border-bottom: 1px solid rgba(255, 255, 255, 0.093);
            border-left: 1px solid rgba(255, 255, 255, 0.093);
            background-color: rgba(0, 0, 0, 0.159);

            h2 {
                margin: 0.2em 0;
            }
        }

        .scrapbook {
            background-color: transparent;
            grid-row: 1 / 3;
            grid-column: 3;
            height: 100%;
            border-left: 1px solid rgba(255, 255, 255, 0.093);
            overflow: auto;
            div {
                padding: 2em;
            }
            .content {
                padding-top: 0;
            }
        }

        .songs {
            grid-row: 2;
            grid-column: 1;
            border-right: 1px solid rgba(255, 255, 255, 0.093);
            div {
                padding: 2em;
            }
        }

        .song-details {
            grid-row: 2;
            grid-column: 2;
            height: 100%;
            div {
                padding: 2em;
            }
        }
    }
    hr {
        border-top: 1px solid rgba(255, 255, 255, 0.093);
        border-bottom: none;
        border-left: none;
        border-right: none;
    }
</style>
