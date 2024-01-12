<script lang="ts">
    import { fade } from "svelte/transition";
    import { getLyrics } from "../../data/LyricsGrabber";
    import {
        currentSong,
        currentSongLyrics,
        isLyricsHovered,
        isLyricsOpen,
        isSettingsOpen,
        userSettings
    } from "../../data/store";
    import Icon from "../ui/Icon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    export var fontSize = "font-20";
    export var isFullScreen = false;
    var onLyricsUpdated;

    function onLyricsChanged(e) {
        onLyricsUpdated(e.target.value);
    }

    let isLoading = false;
    let error = null;

    async function grabLyrics() {
        if (
            $currentSongLyrics?.songId !== $currentSong.id &&
            $currentSong?.title &&
            $currentSong?.artist
        ) {
            isLoading = true;
            try {
                let lyrics = await getLyrics(
                    $currentSong.title,
                    $currentSong.artist
                );
                $currentSongLyrics = lyrics
                    ? {
                          songId: $currentSong.id,
                          lyrics
                      }
                    : null;
                error = null;
            } catch (err) {
                error = err;
            } finally {
                isLoading = false;
            }
        }
    }

    let isEmpty = false;
    $: if ($currentSong && $currentSong?.title && $currentSong?.artist) {
        grabLyrics();
    } else {
        $currentSongLyrics = null;
        isEmpty = true;
    }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
    class="container {fontSize}"
    class:full-screen={isFullScreen}
    on:mouseenter={() => {
        $isLyricsHovered = true;
    }}
    on:mouseleave={() => {
        $isLyricsHovered = false;
    }}
>
    <div class="content">
        <header>
            <div class="close-icon">
                <Icon
                    icon="mingcute:close-circle-fill"
                    onClick={() => {
                        $isLyricsHovered = false;
                        $isLyricsOpen = false;
                    }}
                />
            </div>
            <div class="info">
                <p>Lyrics</p>
                <small>{$currentSong.title}</small>
            </div>
            <div></div>
        </header>
        {#if !$userSettings.geniusApiKey?.length}
            <div class="placeholder">
                <br />
                <p>Missing Genius API key</p>
                <small
                    >This feature uses the Genius API to check for lyrics</small
                >
                <p>
                    Create one on <a href="https://genius.com/developers"
                        >genius.com/developers</a
                    >, <br />then add it in settings:
                </p>
                <br />
                <ButtonWithIcon
                    text="Add key in settings"
                    onClick={() => {
                        $isSettingsOpen = true;
                    }}
                />
            </div>
        {:else if error}
            <div class="placeholder error">
                <br />
                <p>Error fetching lyrics</p>
                <small>{error}</small>
            </div>
        {:else if isLoading}
            <div class="loading" transition:fade={{ duration: 150 }}>
                <h4>Fetching lyrics...</h4>
                <span
                    ><LoadingSpinner />
                    <p>one sec...</p></span
                >
            </div>
        {:else if $currentSongLyrics}
            <textarea
                placeholder="Start typing..."
                on:input={onLyricsChanged}
                bind:value={$currentSongLyrics.lyrics}
            />
        {:else}
            <div class="placeholder">
                <br />
                <p>No lyrics</p>
                {#if isEmpty}
                    <small>Not enough metadata</small>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style lang="scss">
    .container {
        position: absolute;
        bottom: 3em;
        right: 2em;
        max-height: 400px;
        text-align: center;
        min-width: 330px;
        max-width: 360px;
        overflow: hidden;
        z-index: 2;
        height: 100%;
        border-radius: 5px;
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(73, 70, 70);
        background: rgba(53, 50, 54, 0.8);
        box-shadow: 0px 5px 40px rgba(0, 0, 0, 0.259);
        backdrop-filter: blur(8px);

        .content {
            position: relative;
            width: 100%;
            height: 100%;
            display: flex;
            flex-direction: column;

            p {
                opacity: 0.5;
            }

            .loading {
                position: absolute;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                margin: auto;
                display: flex;
                height: fit-content;
                flex-direction: column;
                align-items: center;
                * {
                    margin: 0;
                }
                span {
                    display: inline-flex;
                    gap: 4px;
                    align-items: center;
                }
            }

            .placeholder {
                margin: auto 2em;
                display: flex;
                height: 100%;
                align-items: center;
                justify-content: center;
                flex-direction: column;
                p {
                    margin: 0;
                }
                &.error {
                    small {
                        color: rgb(249, 119, 119);
                    }
                }
            }

            header {
                display: grid;
                grid-template-columns: 1fr 1fr 1fr;
                border-bottom: 1px solid rgb(59, 56, 56);

                .close-icon {
                    display: flex;
                    flex-direction: row;
                    justify-content: flex-start;
                    align-items: center;
                    padding: 0 1em;
                    opacity: 0.5;
                }
                .info {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    padding: 0.5em 0;
                    * {
                        margin: 0;
                        line-height: normal;
                    }
                    p {
                        opacity: 1;
                        white-space: initial;
                    }
                    small {
                        opacity: 0.7;
                    }
                }
            }

            textarea {
                appearance: none;
                width: 100%;
                height: 100%;
                padding: 3rem 2em;
                text-align: center;
                outline: none;
                border: none;
                background: none;
                font-size: inherit;
                line-height: 1.8em;
                resize: none;
                color: rgb(236, 213, 222);
                font-weight: 500;
            }

            &.font-12 {
                font-size: 12px;
            }

            &.font-14 {
                font-size: 14px;
            }

            &.font-20 {
                font-size: 20px;
            }

            &.font-24 {
                font-size: 24px;
            }

            &.font-32 {
                font-size: 32px;
            }

            &.font-48 {
                font-size: 48px;
            }
        }
    }
</style>
