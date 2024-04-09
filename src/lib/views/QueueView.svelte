<script lang="ts">
    import { fly } from "svelte/transition";
    import {
        albumPlaylist,
        currentSong,
        currentSongIdx,
        isShuffleEnabled,
        playlist,
        playlistIsAlbum,
        shuffledPlaylist
    } from "../../data/store";
    import audioPlayer from "../player/AudioPlayer";
    $: upcoming = $isShuffleEnabled
        ? $shuffledPlaylist.filter((v, i) =>
              $playlistIsAlbum ? true : i >= $currentSongIdx
          )
        : $playlist.filter((v, i) =>
              $playlistIsAlbum ? true : i >= $currentSongIdx
          );
</script>

{#if $playlist?.length}
    <div class="container">
        <div class="queue">
            <header>
                <p>
                    Queue
                    {#if $playlistIsAlbum}
                        <span>(Album mode)</span>
                    {:else}
                        <span>(Library mode)</span>
                    {/if}
                </p>
                {#if $playlistIsAlbum}
                <small>{$currentSong.album}</small>
                {:else}
                <small style="opacity: 0.5;">Showing upcoming tracks</small>
                {/if}
            </header>
            <div class="tracks">
                {#each upcoming as track, idx}
                    <div
                        class="track"
                        class:playing={$currentSong.id === track.id}
                        on:click={() => {
                            audioPlayer.playSong(track);
                        }}
                    >
                        <p class="number">{idx + 1}.</p>
                        <p class="title">{track.title}</p>
                        {#if $currentSong.id === track.id}<iconify-icon
                                icon="heroicons-solid:volume-up"
                            />{/if}
                    </div>
                {/each}
            </div>
        </div>
    </div>
{/if}

<style lang="scss">
    .container {
        h1 {
            font-family: "Snake";
            margin: 0 0.5em 0.5em;
            text-align: left;
        }
        display: flex;
        flex-direction: column;
        /* background: rgba(36, 34, 34, 0.943); */
        margin-top: 5px;
        box-sizing: content-box;
        border-radius: 5px;

        border-top: 0.7px solid #ffffff36;
        border-right: 0.7px solid #ffffff16;
        border-left: 0.7px solid #ffffff2a;

        /* border: 0.7px solid #ffffff0b; */
        overflow: hidden;
        color: white;
        bottom: 0;
        z-index: 11;
        /* min-width: 180px;
        width: max-content; */
        height: 100%;
        max-width: 290px;
        width: 290px;
        position: relative;
        /* background-color: rgba(36, 33, 34, 0.648); */
        .queue {
            overflow-y: auto;
        }

        header {
            position: sticky;
            top: 0;
            height: 50px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-items: center;
            margin: 0;
            z-index: 2;
            border-bottom: 0.7px solid #ffffff2a;

            p {
                margin: 3px 0 0 0;
                font-weight: 500;
            }

            small {
                margin: 0;
                line-height: normal;
            }

            span {
                opacity: 0.5;
            }

            div {
            }
            backdrop-filter: blur(8px);

            background-color: #322c3f7e;
        }

        .album-info {
            display: flex;
            flex-direction: column;
            margin: 0 1em 1em;
            p {
                margin: 0;
                text-align: left;
                &:nth-child(2) {
                    opacity: 0.6;
                    position: relative;
                    &:before {
                        content: "by ";
                        opacity: 0.4;
                    }
                }
            }
        }
    }

    .tracks {
        display: flex;
        flex-direction: column;
        justify-content: space-evenly;
        /* background-color: rgb(36, 32, 38); */

        .track {
            padding: 0.1em 1em;
            height: 26px;
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 4px;
            user-select: none;
            border-radius: 3px;
            color: #9591a3;
            white-space: nowrap;
            overflow: hidden;
            cursor: default;
            .number {
                margin: 0;
                text-align: left;
                font-size: 12px;
                vertical-align: bottom;
                line-height: normal;
                opacity: 0.5;
            }
            .title {
                margin: 0;
                vertical-align: bottom;
                text-align: left;
                font-size: 13.2px;
                text-overflow: ellipsis;
                overflow: hidden;
                &:nth-child(1) {
                    opacity: 0.4;
                    width: auto;
                }
            }

            &:hover {
                background-color: rgba(0, 0, 0, 0.087);
            }

            &.playing {
                color: #dad4ed;
                font-weight: bold;
                iconify-icon {
                    color: #7f61dd;
                }
            }
            &:not(:nth-child(1)) {
                border-top: 1px solid rgba(255, 255, 255, 0.03);
            }
        }
    }
</style>
