<script>
    import { db } from "../../data/db";
    import {
        currentSong,
        isQueueCleared,
        isShuffleEnabled,
        isSidebarOpen,
        playlist,
        queriedSongs,
        queueMode,
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

<div class="container" class:extra-margin={!$isSidebarOpen}>
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
        class="toggle clear-btn"
        class:active={$queueMode === "custom"}
        on:click={() => {
            $queueMode = "custom";
            audioPlayer.shouldPlay = false; // Avoid re-starting playback after playlist change
            if ($isShuffleEnabled) {
                $shuffledPlaylist = [];
            } else {
                $playlist = [];
            }
            $isQueueCleared = true;
        }}>Custom queue</button
    >
    <button
        class="toggle fill-btn"
        class:active={$queueMode === "library"}
        on:click={() => {
            $queueMode = "library";
            $playlist = $queriedSongs;
            $isQueueCleared = false;
        }}>Same as library →</button
    >
</div>

<style lang="scss">
    .container {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 5px;
        overflow: visible;
        position: relative;

        &.extra-margin {
            margin-bottom: 5px;
        }

        .new-playlist {
            grid-column: 1 / 3;
            grid-row: 2;

            .new-playlist-input {
                display: grid;
                grid-template-columns: 1fr auto;
                gap: 5px;
                padding: 0 1px;
                button {
                    border-radius: 5px;
                }
            }
        }
    }
    button {
        width: 100%;
        border-radius: 5px;
        height: 30px;
        background-color: rgba(119, 119, 119, 0.24);
        border: 1px solid rgba(255, 255, 255, 0.118);
        color: var(--text);
        vertical-align: center;
        padding-top: 0;
        padding-bottom: 0;
        * {
            user-select: none;
        }
        &:disabled {
            opacity: 0.5;
        }

        &:hover {
            background-color: rgba(119, 119, 119, 0.45);
        }
        &.toggle {
            margin-top: -8px;
            padding-top: 0px;
            border: 1px solid transparent;
            background-color: rgba(119, 119, 119, 0.02);
            color: var(--text-inactive);
            height: 26px;
            font-size: 12.5px;
            &:hover {
                background-color: rgba(119, 119, 119, 0.15);
            }
            &:active {
                background-color: rgba(119, 119, 119, 0.056);
                border: 1px solid transparent;
            }
            &.active {
                background-color: rgba(119, 119, 119, 0.056);
                border-bottom: 0.7px solid #ffffff2a;
                color: var(--text-active);
            }

            &.clear-btn {
                grid-column: 1;
                grid-row: 1;
                border-top-left-radius: 0px;
                border-top-right-radius: 0px;
                border-bottom-left-radius: 5px;
                border-bottom-right-radius: 5px;
                clip-path: polygon(
                    0 0,
                    18% 7%,
                    100% 7%,
                    100% 80%,
                    100% 100%,
                    40% 100%,
                    0 100%
                );

                &.active {
                    border-right: 0.7px solid #ffffff2a;
                    border-left: 0.7px solid #ffffff2a;
                }
            }

            &.fill-btn {
                grid-column: 2;
                grid-row: 1;
                border-radius: 1px;
                border-top-left-radius: 0px;
                border-top-right-radius: 0px;
                border-bottom-left-radius: 5px;
                border-bottom-right-radius: 5px;
                clip-path: polygon(
                    0 9%,
                    86% 9%,
                    100% 0,
                    100% 80%,
                    100% 100%,
                    40% 100%,
                    0 100%
                );
                &.active {
                    border-right: 0.7px solid #ffffff2a;
                    border-left: 0.7px solid #ffffff2a;
                }
            }
        }
    }
</style>
