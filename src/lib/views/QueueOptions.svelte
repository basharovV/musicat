<script>
    import { db } from "../../data/db";
    import {
        currentSong,
        isQueueCleared,
        isShuffleEnabled,
        playlist,
        queriedSongs,
        shuffledPlaylist
    } from "../../data/store";
    import audioPlayer from "../player/AudioPlayer";
    import Input from "../ui/Input.svelte";

    let playlistName = "";
    let isPlaylistInputShown = true;

    function onPlaylistNameChange(ev) {}

    async function createPlaylist() {
        const tracks = ($isShuffleEnabled ? $shuffledPlaylist : $playlist).map(
            (t) => t.id
        );
        await db.playlists.add({
            title: playlistName,
            tracks
        });
        playlistName = "";
    }
</script>

<div class="container">
    <div class="new-playlist">
        {#if !isPlaylistInputShown}
            <button
                disabled={($isShuffleEnabled ? $shuffledPlaylist : $playlist)
                    .length === 1}
                on:click={() => {
                    isPlaylistInputShown = true;
                }}>Create playlist</button
            >
        {:else}
            <div class="new-playlist-input">
                <Input
                    fullWidth
                    alt
                    bind:value={playlistName}
                    placeholder="New playlist name"
                    onChange={onPlaylistNameChange}
                    onEnterPressed={createPlaylist}
                />
                <button
                    disabled={playlistName?.length === 0}
                    on:click={createPlaylist}>Create</button
                >
            </div>
        {/if}
    </div>
    <button
        class="clear-btn"
        disabled={($isShuffleEnabled ? $shuffledPlaylist : $playlist).length ===
            0}
        on:click={() => {
            audioPlayer.shouldPlay = false; // Avoid re-starting playback after playlist change
            if ($isShuffleEnabled) {
                $shuffledPlaylist = [];
            } else {
                $playlist = [];
            }
            $isQueueCleared = true;
        }}>Clear queue</button
    >
    <button
        class="fill-btn"
        disabled={($isShuffleEnabled ? $shuffledPlaylist : $playlist).length >=
            1}
        on:click={() => {
            $playlist = $queriedSongs;
            $isQueueCleared = false;
        }}>Mirror library</button
    >
</div>

<style lang="scss">
    .container {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 5px;
        overflow: visible;
        position: relative;

        .new-playlist {
            grid-column: 1 / 3;
            grid-row: 1;

            .new-playlist-input {
                display: grid;
                grid-template-columns: 1fr auto;
                gap: 5px;
                padding: 0 1px;
            }
        }

        .clear-btn {
            grid-column: 1;
            grid-row: 2;
        }

        .fill-btn {
            grid-column: 2;
            grid-row: 2;
        }
    }
    button {
        width: 100%;
        background-color: rgba(119, 119, 119, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.138);
        border-radius: 5px;
        height: 30px;
        color: rgb(210, 210, 210);
        &:hover {
            background-color: rgba(119, 119, 119, 0.15);
        }
        vertical-align: center;
        padding-top: 0;
        padding-bottom: 0;
        * {
            user-select: none;
        }

        &:disabled {
            opacity: 0.5;
        }
    }
</style>
