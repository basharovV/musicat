<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    import type { Song } from "../../App";
    import {
        currentSong,
        currentSongIdx,
        isPlaying,
        isShuffleEnabled,
        isSidebarOpen,
        isTrackInfoPopupOpen,
        isWaveformOpen,
        playerTime,
        playlist,
        queriedSongs,
        rightClickedTrack,
        rightClickedTracks,
        seekTime,
        volume
    } from "../../data/store";
    import { currentThemeObject } from "../../theming/store";
    import Icon from "../ui/Icon.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import { db } from "../../data/db";
    import Seekbar from "../sidebar/Seekbar.svelte";
    import { lookForArt } from "../../data/LibraryImporter";
    import { Buffer } from "buffer";
    import VolumeSlider from "../ui/VolumeSlider.svelte";
    import tippy from "tippy.js";

    let duration;
    let artworkSrc;

    $: elapsedTime = `${(~~($playerTime / 60))
        .toString()
        .padStart(2, "0")}:${(~~($playerTime % 60))
        .toString()
        .padStart(2, "0")}`;

    $: durationText = `${(~~(duration / 60)).toString().padStart(2, "0")}:${(~~(
        duration % 60
    ))
        .toString()
        .padStart(2, "0")}`;

    currentSong.subscribe(async (song) => {
        if (song) {
            const songWithArtwork = await invoke<Song>("get_song_metadata", {
                event: { path: song.path, isImport: false }
            });
            console.log("test", songWithArtwork);
            duration = songWithArtwork.fileInfo.duration;

            if (songWithArtwork.artwork) {
                let artworkFormat = songWithArtwork.artwork.format;
                let artworkBuffer = Buffer.from(songWithArtwork.artwork.data);
                artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                    "base64"
                )}`;
            } else {
                artworkSrc = null;
                const artwork = await lookForArt(song.path, song.file);
                if (artwork) {
                    artworkSrc = artwork.artworkSrc;
                }
            }
        }
    });

    function togglePlayPause() {
        if (!audioPlayer.currentSong) {
            audioPlayer.shouldPlay = true;
            $playlist = $queriedSongs;
        } else {
            audioPlayer.togglePlay();
        }
    }

    async function favouriteCurrentSong() {
        if (!$currentSong) return;
        $currentSong.isFavourite = !$currentSong.isFavourite;
        await db.songs.put($currentSong);
        $currentSong = $currentSong;
    }
</script>

<top-bar data-tauri-drag-region class:sidebar-collapsed={!$isSidebarOpen}>
    <div class="transport" data-tauri-drag-region>
        <div class="shuffle">
            <Icon
                class="transport-side"
                icon="ph:shuffle-bold"
                size={18}
                color={!$isShuffleEnabled
                    ? $currentThemeObject["icon-secondary"]
                    : $currentThemeObject["transport-shuffle"]}
                onClick={() => {
                    $isShuffleEnabled = !$isShuffleEnabled;
                }}
            />
        </div>
        <Icon
            class="transport-middle"
            icon="fe:backward"
            size={24}
            disabled={$currentSongIdx === 0}
            onClick={() => audioPlayer.playPrevious()}
            color={$currentThemeObject["transport-controls"]}
        />
        <Icon
            class="transport-middle"
            size={24}
            onClick={togglePlayPause}
            icon={$isPlaying ? "fe:pause" : "fe:play"}
            color={$currentThemeObject["transport-controls"]}
        />
        <Icon
            class="transport-middle"
            size={24}
            icon="fe:forward"
            disabled={$playlist.length === 0 ||
                $currentSongIdx === $playlist?.length - 1}
            onClick={() => audioPlayer.playNext()}
            color={$currentThemeObject["transport-controls"]}
        />
        <div class="favourite">
            <Icon
                class="transport-side favourite {$currentSong?.isFavourite
                    ? 'active'
                    : 'inactive'}"
                size={18}
                color={$currentSong?.isFavourite
                    ? $currentThemeObject["transport-favorite"]
                    : $currentThemeObject["icon-secondary"]}
                icon={$currentSong?.isFavourite
                    ? "clarity:heart-solid"
                    : "clarity:heart-line"}
                onClick={() => {
                    favouriteCurrentSong();
                }}
            />
        </div>
    </div>
    <div class="middle-container" data-tauri-drag-region>
        <div class="middle" data-tauri-drag-region>
            {#if artworkSrc}
                <div class="artwork">
                    <img src={artworkSrc} alt="artwork" />
                </div>
            {/if}
            <div class="song-info" data-tauri-drag-region>
                <small
                    >{$currentSong?.title}
                    <span> • {$currentSong?.artist}</span>
                    <span> • {$currentSong?.album}</span>
                </small>
            </div>

            <div class="track-info-icon">
                <Icon
                    size={16}
                    icon="mdi:information"
                    onClick={() => {
                        $rightClickedTrack = $currentSong;
                        $rightClickedTracks = [];
                        $isTrackInfoPopupOpen = true;
                    }}
                    color={$currentThemeObject["icon-secondary"]}
                />
            </div>
        </div>
    </div>
    <div class="right" data-tauri-drag-region>
        <VolumeSlider />

        <div
            class="visualizer-icon"
            use:tippy={{
                content: "waveform, loop region, marker editor",
                placement: "top"
            }}
        >
            <Icon
                icon="ph:wave-sine-duotone"
                onClick={() => ($isWaveformOpen = !$isWaveformOpen)}
                color={$isWaveformOpen
                    ? $currentThemeObject["accent-secondary"]
                    : $currentThemeObject["icon-secondary"]}
            />
        </div>
    </div>
    <div></div>

    <div class="seekbar-outer">
        <div class="seekbar">
            <Seekbar
                {duration}
                onSeek={(time) => seekTime.set(time)}
                playerTime={$playerTime}
                style="thin"
                showProgress
            />
        </div>
    </div>
    <div></div>
</top-bar>

<style lang="scss">
    top-bar {
        color: var(--text);
        display: grid;
        grid-template-columns: 170px 1fr 170px;
        grid-template-rows: auto auto;
        align-items: center;
        justify-content: flex-end;
        position: relative;
        margin: 5px 5px 0 0;

        * {
            user-select: none;
        }

        @media screen and (max-width: 600px) {
            grid-template-columns: 120px 1fr 120px;
        }

        &.sidebar-collapsed {
            /* margin-left: 70px; */
        }

        p {
            margin: 0;
        }

        .transport {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 10px;
            padding: 0 10px;
            justify-content: flex-start;

            @media screen and (max-width: 600px) {
                .favourite,
                .shuffle {
                    display: none;
                }
            }
        }

        .middle-container {
            display: flex;
            flex-direction: column;
            width: 100%;
            align-items: center;
            position: relative;
            justify-content: center;
            overflow: hidden;
        }

        .middle {
            justify-self: center;
            max-width: 600px;
            display: flex;
            flex-direction: row;
            position: relative;
            width: 100%;
            overflow: hidden;
            border-radius: 5px;
            border-top: 0.7px solid
                color-mix(in srgb, var(--inverse) 10%, transparent);
            border-bottom: 0.7px solid
                color-mix(in srgb, var(--inverse) 40%, transparent);
            border-right: 0.7px solid
                color-mix(in srgb, var(--inverse) 40%, transparent);
            border-left: 0.7px solid
                color-mix(in srgb, var(--inverse) 40%, transparent);

            background-color: color-mix(
                in srgb,
                var(--library-playing-bg) 4%,
                transparent
            );
            // Inset box shadow
            box-shadow: inset 0 0 30px 2px
                color-mix(in srgb, var(--type-bw-inverse) 6%, transparent);

            &:hover {
                box-shadow: inset 0 0 30px 2px
                    color-mix(in srgb, var(--type-bw-inverse) 17%, transparent);
            }

            .artwork {
                height: 30px;
                width: 30px;
                img {
                    height: 100%;
                    width: auto;
                    object-fit: cover;
                }
            }

            .song-info {
                pointer-events: none;
                white-space: nowrap;
                position: relative;
                width: 100%;
                overflow: hidden;
                text-overflow: ellipsis;
                padding: 0 20px 5px 20px;
                small {
                    font-size: 12.5px;
                    color: var(--text);
                    span {
                        color: var(--text-secondary);
                    }
                }
            }

            .track-info-icon {
                display: flex;
                align-items: center;
                margin-right: 10px;
                margin-bottom: 2px;
                z-index: 10;
                pointer-events: all;
            }
        }

        .right {
            display: flex;
            justify-self: flex-end;
            margin: 0 1em;
            gap: 10px;
        }

        .filler {
            grid-row: 1;
        }

        .seekbar-outer {
            grid-row: 2;
            grid-column: 2;
            width: 100%;
            display: flex;
            justify-content: center;
            align-items: center;
            position: relative;
        }
        .seekbar {
            position: absolute;
            width: 100%;
            max-width: 600px;
            margin: 0 5px;
            z-index: 12;
            top: -7.5px;
            .elapsed-time {
                width: max-content;
                white-space: nowrap;
                font-size: 11px;
                opacity: 0.7;
            }
        }
    }
</style>
