<script lang="ts">
    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import { Buffer } from "buffer";
    import tippy from "tippy.js";
    import type { Song } from "../../App";
    import { db } from "../../data/db";
    import {
        current,
        isPlaying,
        isShuffleEnabled,
        popupOpen,
        isSidebarOpen,
        isWaveformOpen,
        playerTime,
        queriedSongs,
        queue,
        rightClickedTrack,
        rightClickedTracks,
        seekTime,
        lastWrittenSongs,
        rightClickedAlbum,
        repeatMode,
    } from "../../data/store";
    import { currentThemeObject } from "../../theming/store";
    import audioPlayer from "../player/AudioPlayer";
    import Seekbar from "../sidebar/Seekbar.svelte";
    import Icon from "../ui/Icon.svelte";
    import VolumeSlider from "../ui/VolumeSlider.svelte";
    import { setQueue } from "../../data/storeHelper";

    let duration;
    let artworkSrc;
    let currentSong;

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

    current.subscribe(async ({ song }) => {
        if (song) {
            if (
                song.path === currentSong?.path &&
                !$lastWrittenSongs.some(({ path }) => path === song.path)
            ) {
                // same song, no need to update
                // (unless the metadata was just written to eg. updated artwork)
                return;
            }

            const songWithArtwork = await invoke<Song>("get_song_metadata", {
                event: {
                    path: song.path,
                    isImport: false,
                    includeFolderArtwork: true,
                    includeRawTags: false,
                },
            });

            currentSong = song.id;
            duration = songWithArtwork.fileInfo.duration;

            if (songWithArtwork.artwork) {
                if (songWithArtwork.artwork.data.length) {
                    let artworkBuffer = Buffer.from(
                        songWithArtwork.artwork.data,
                    );
                    let artworkFormat = songWithArtwork.artwork.format;
                    artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                        "base64",
                    )}`;
                } else if (songWithArtwork.artwork.src) {
                    artworkSrc = convertFileSrc(songWithArtwork.artwork.src);
                }
            } else {
                artworkSrc = null;
            }
        }
    });

    function togglePlayPause() {
        if (!audioPlayer.currentSong) {
            setQueue($queriedSongs, true);
        } else {
            audioPlayer.togglePlay();
        }
    }

    async function favouriteCurrentSong() {
        if (!$current.song) return;
        $current.song.isFavourite = !$current.song.isFavourite;
        await db.songs.put($current.song);
        $current = $current;
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
            disabled={$current.index === 0}
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
            disabled={$queue.length === 0 ||
                $current.index === $queue?.length - 1}
            onClick={() => audioPlayer.playNext()}
            color={$currentThemeObject["transport-controls"]}
        />

        <Icon
            class="transport-side repeat"
            icon={$repeatMode === "track" ? "ph:repeat-once" : "ph:repeat-bold"}
            color={$repeatMode === "none"
                ? $currentThemeObject["icon-secondary"]
                : $currentThemeObject["transport-repeat"]}
            onClick={() => {
                audioPlayer.cycleRepeat();
            }}
        />
        <div class="favourite">
            <Icon
                class="transport-side favourite {$current.song?.isFavourite
                    ? 'active'
                    : 'inactive'}"
                size={18}
                color={$current.song?.isFavourite
                    ? $currentThemeObject["transport-favorite"]
                    : $currentThemeObject["icon-secondary"]}
                icon={$current.song?.isFavourite
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
                {#if $current.song}
                    <small>
                        {$current.song.title}
                        <span> • {$current.song.artist}</span>
                        <span> • {$current.song.album}</span>
                    </small>
                {/if}
            </div>

            {#if $current.song}
                <div class="track-info-icon">
                    <Icon
                        size={16}
                        icon="mdi:information"
                        onClick={() => {
                            $rightClickedTrack = $current.song;
                            $rightClickedTracks = [];
                            $rightClickedAlbum = null;
                            $popupOpen = "track-info";
                        }}
                        color={$currentThemeObject["icon-secondary"]}
                    />
                </div>
            {/if}
        </div>
    </div>
    <div class="right" data-tauri-drag-region>
        <VolumeSlider />
    </div>

    <div></div>
    {#if $current.song}
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
    {/if}
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
            grid-template-columns: 1fr 1fr;
            grid-template-rows: auto auto 40px;
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
                grid-row: 3;
            }
            @media screen and (max-width: 340px) {
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

            @media screen and (max-width: 600px) {
                grid-row: 1;
                grid-column: 1 / 3;
            }
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
            border-top: 0.7px solid var(--panel-primary-border-accent2);
            border-bottom: 0.7px solid var(--panel-primary-border-main);
            border-right: 0.7px solid var(--panel-primary-border-main);
            border-left: 0.7px solid var(--panel-primary-border-main);

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
                height: 30px;

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

            @media screen and (max-width: 600px) {
                grid-row: 3;
            }
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

            @media screen and (max-width: 600px) {
                grid-row: 2;
                grid-column: 1 / 3;
            }
        }
        .seekbar {
            position: absolute;
            width: 100%;
            max-width: 600px;
            margin: 0 5px;
            z-index: 12;
            top: -8px;
            .elapsed-time {
                width: max-content;
                white-space: nowrap;
                font-size: 11px;
                opacity: 0.7;
            }
        }
    }
</style>
