<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import {
        current,
        isPlaying,
        isSmartQueryBuilderOpen,
        query,
        queue,
        selectedPlaylistFile,
        selectedSmartQuery,
        toDeletePlaylist,
        uiView,
    } from "../../data/store";
    import hotkeys from "hotkeys-js";
    import audioPlayer from "../player/AudioPlayer";
    import LL from "../../i18n/i18n-svelte";
    import { db } from "../../data/db";

    /**
     * In prune mode, the user is presented songs in the library,
     * and can quickly mark them for deletion via keyboard shortcuts.
     * "K" to keep, "D" to delete, and "Q" to quit the prune mode.
     *
     * Songs marked for deletion will be found in the "To delete" playlist (see sidebar).
     */

    let pressedK = false;
    let pressedD = false;
    let atEnd = false;

    async function getOrCreatePlaylist() {
        let toDelete = $toDeletePlaylist;
        if (!toDelete) {
            await db.internalPlaylists.put({
                id: "todelete",
                title: $LL.toDelete.title(),
                tracks: [],
                path: null,
            });
            toDelete = await db.internalPlaylists.get("todelete");
        }
        return toDelete;
    }

    async function addToDelete() {
        const toDelete = await getOrCreatePlaylist();

        if (!atEnd || !toDelete.tracks.includes($current.song.id)) {
            toDelete.tracks.push($current.song.id);
            db.internalPlaylists.put(toDelete);
        }

        if ($queue.length !== 0 && !atEnd) {
            audioPlayer.playNext();
            pressedD = true;
            setTimeout(() => (pressedD = false), 300);
        }
    }

    async function gotoToDeleteView() {
        const toDelete = $toDeletePlaylist;

        if (!toDelete || toDelete.tracks.length === 0) {
            if ($toDeletePlaylist) {
                db.internalPlaylists.delete($toDeletePlaylist.id);
            }

            $uiView = "library";
            $isSmartQueryBuilderOpen = false;
            $selectedPlaylistFile = null;
            $selectedSmartQuery = null;
            $query.orderBy = $query.libraryOrderBy;
        } else {
            $uiView = "to-delete";
        }
    }

    hotkeys("d", "prune", () => {
        addToDelete();
    });

    hotkeys("k", "prune", () => {
        if ($queue.length !== 0 && !atEnd) {
            audioPlayer.playNext();
            pressedK = true;
            setTimeout(() => (pressedK = false), 300);
        }
    });

    hotkeys("q", "prune", () => {
        gotoToDeleteView();
    });

    onMount(() => {
        hotkeys.setScope("prune");
        // Start playback if not playing
        if (!$isPlaying) audioPlayer?.playCurrent();
    });

    onDestroy(() => {
        hotkeys.deleteScope("prune");
    });

    $: atEnd = $current.index === $queue?.length - 1;
</script>

<div class="container">
    <div class="prune-mode-overlay">
        <h2>Prune Mode</h2>
        <p class="subtitle">
            Listen to tracks and quickly mark them for deletion
        </p>
        <div class="track-info">
            <h3>{$current.song?.title}</h3>
            <h4>{$current.song?.artist}</h4>
            <small>{$current.song?.album}</small>
        </div>
        <div class="shortcuts">
            <div class="delete" class:pressed={pressedD}>
                <p>Press</p>
                <div class="keys">
                    <h1>D</h1>
                </div>
                <p>to delete</p>
            </div>
            <div class="keep" class:pressed={pressedK}>
                <p>Press</p>
                <div class="keys">
                    <h1>K</h1>
                </div>
                <p>to keep</p>
            </div>
        </div>
        <div class="for-delete">
            <p>
                {$toDeletePlaylist?.tracks?.length ?? 0} tracks up for deletion
            </p>
        </div>
        <div class="footer">
            <p>Press Q to finish pruning</p>
            <small
                >Songs to delete will be found in the To Delete playlist</small
            >
        </div>
    </div>
</div>

<style lang="scss">
    .container {
        position: relative;
    }
    .prune-mode-overlay {
        position: absolute;
        left: 0;
        right: 0;
        top: 0;
        bottom: 0px;
        width: auto;
        height: fit-content;
        margin: auto;
        border: 1px solid color-mix(in srgb, var(--inverse) 20%, transparent);
        background-color: var(--overlay-bg);
        backdrop-filter: blur(10px);
        min-width: 500px;
        max-width: 500px;
        min-height: 350px;
        border-radius: 5px;
        color: var(--text);
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        .subtitle {
            opacity: 0.5;
        }
        .track-info,
        .for-delete {
            padding: 1em;
            width: 100%;
            border-bottom: 1px solid
                color-mix(in srgb, var(--inverse), transparent 70%);
            * {
                margin: 0;
            }
        }
        .shortcuts {
            display: grid;
            grid-template-columns: 1fr 1fr;
            justify-content: center;
            align-items: center;
            gap: 10px;
            margin: 1em 0;

            .delete {
                &.pressed {
                    h1 {
                        transition: all 0.2s ease-in-out;
                        box-shadow: 0px 0px 20px
                            var(--prune-button-delete-pressed-shadow);
                    }
                }
                h1 {
                    background-color: var(--prune-button-delete-bg);
                }
            }
            .keep {
                &.pressed {
                    h1 {
                        transition: all 0.2s ease-in-out;
                        box-shadow: 0px 0px 20px
                            var(--prune-button-keep-pressed-shadow);
                    }
                }
                h1 {
                    background-color: var(--prune-button-keep-bg);
                }
            }

            .delete,
            .keep {
                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: center;
                gap: 5px;
                p {
                    margin: 0;
                    opacity: 0.5;
                    &:nth-of-type(2) {
                        opacity: 1;
                    }
                }
                h1 {
                    border: 1px solid var(--prune-button-border);
                    border-radius: 5px;
                    margin: 0;
                    padding: 1em;
                    font-weight: 600;
                }
            }
        }

        .for-delete {
            color: var(--prune-fordelete);
        }

        .footer {
            padding: 1em;
        }
        p {
            margin: 0;
        }
        small {
            opacity: 0.5;
        }
    }
</style>
