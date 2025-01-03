<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { getLyrics } from "../../data/LyricsGrabber";
    import {
        currentSong,
        currentSongLyrics,
        isLyricsHovered,
        isLyricsOpen,
        popupOpen,
        isWaveformOpen,
        userSettings
    } from "../../data/store";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    export var isFullScreen = false;
    export var right = 0;
    var onLyricsUpdated;

    function onLyricsChanged(e) {
        onLyricsUpdated(e.target.value);
    }

    const fontSizes = ["font-xs", "font-s", "font-m", "font-l", "font-xl"];

    let currentFontSizeIdx = 2;
    $: fontSize = fontSizes[currentFontSizeIdx];

    function increaseFontSize() {
        currentFontSizeIdx =
            currentFontSizeIdx <= fontSizes.length - 2
                ? currentFontSizeIdx + 1
                : fontSizes.length - 1;
    }
    function decreaseFontSize() {
        currentFontSizeIdx =
            currentFontSizeIdx > 0 ? currentFontSizeIdx - 1 : 0;
    }

    let isLoading = false;
    let error = null;

    function replaceHeadingsWithSpan(lyricsText) {
        // Use a regular expression to match any text within square brackets
        lyricsText = lyricsText
            .replace(/\[(.*?)\]/g, function (match, p1) {
                // Create a <span> element with the class as the lowercase text within square brackets
                return (
                    '<span class="section" contenteditable="false">' +
                    p1 +
                    "</span>"
                );
            })
            .replace(/(?:\r\n|\r|\n)/g, "<br>");
        return lyricsText;
    }

    async function grabLyrics() {
        if (
            $currentSongLyrics?.songId !== $currentSong.id &&
            $currentSong?.title &&
            $currentSong?.artist
        ) {
            isLoading = true;
            try {
                let result = await getLyrics(
                    $currentSong.title,
                    $currentSong.artist
                );
                $currentSongLyrics = result.lyrics
                    ? {
                          songId: $currentSong.id,
                          lyrics: result.lyrics,
                          writers: result.writers
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

    function getWritersText(writersList: string[]) {
        if (writersList?.length === 1) {
            return writersList[0];
        } else if (writersList?.length === 2) {
            return `${writersList[0]} & ${writersList[1]}`;
        } else if (writersList?.length > 2) {
            let last = writersList[writersList?.length - 1];
            return (
                writersList.slice(0, writersList.length - 1).join(", ") +
                " and " +
                last
            );
        }
        return null;
    }

    let scrolledToBottom = false;

    function onScroll(ev) {
        let area = ev.target;
        if (area)
            scrolledToBottom =
                area.scrollTop + area.offsetHeight >= area.scrollHeight;
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
    class="container"
    class:full-screen={isFullScreen}
    class:extra-space={$isWaveformOpen}
    style="right: {right + 24}px"
    on:mouseenter={() => {
        $isLyricsHovered = true;
    }}
    on:mouseleave={() => {
        $isLyricsHovered = false;
    }}
>
    {#if $isLyricsHovered && scrolledToBottom && $currentSongLyrics?.writers?.length}
        <div class="credits" in:fly={{ y: 5, duration: 150 }}>
            <small>
                Written by <span
                    >{getWritersText($currentSongLyrics.writers)}</span
                ></small
            >
        </div>
    {/if}
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
            <div class="options">
                <Icon
                    icon="mdi:format-font-size-decrease"
                    onClick={decreaseFontSize}
                    boxed
                    size={20}
                />
                <Icon
                    icon="mdi:format-font-size-increase"
                    onClick={increaseFontSize}
                    boxed
                    size={20}
                />
            </div>
        </header>
        {#if !$userSettings.geniusApiKey?.length}
            <div class="placeholder">
                <br />
                <p>Missing Genius API key</p>
                <small
                    >This feature uses the Genius API to check for lyrics</small
                >
                <p>
                    Create one on <a
                            href="https://genius.com/developers"
                            target="_blank"
                        >genius.com/developers</a
                    >, <br />then add it in settings:
                </p>
                <br />
                <ButtonWithIcon
                    text="Add key in settings"
                    onClick={() => {
                        $popupOpen = 'settings';
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
            <div
                class="textarea {fontSize}"
                contenteditable="plaintext-only"
                placeholder="Start typing..."
                on:input={onLyricsChanged}
                on:scroll={onScroll}
            >
                {@html replaceHeadingsWithSpan($currentSongLyrics.lyrics)}
            </div>
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
        bottom: 3.5em;
        right: 2em;
        max-height: 400px;
        text-align: center;
        min-width: 330px;
        max-width: 360px;
        z-index: 13;
        height: 100%;
        border-radius: 5px;
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(73, 70, 70);
        background: rgba(53, 50, 54, 0.8);
        box-shadow: 0px 5px 40px var(--overlay-shadow);
        backdrop-filter: blur(8px);

        &.extra-space {
            bottom: 8em;
        }

        .credits {
            margin-top: -3.8em;
            margin-left: 1.5em;
            margin-right: 1.5em;
            margin-bottom: 1em;
            position: absolute;
            bottom: 0;
            width: 85%;
            z-index: 10;
            padding: 0.3em 1em;
            border-top: 1px solid rgb(73, 68, 68);
            /* border-radius: 5px; */
            /* background: rgb(40, 39, 44); */

            line-height: normal;
            small {
                opacity: 0.5;
            }
        }
        .content {
            position: relative;
            width: 100%;
            height: 100%;
            display: flex;
            flex-direction: column;
            overflow: hidden;

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
                        max-width: 200px;
                        opacity: 0.7;
                        white-space: nowrap;
                        text-overflow: ellipsis;
                        overflow: hidden;
                    }
                }

                .options {
                    display: flex;
                    flex-direction: row;
                    align-items: center;
                    justify-content: flex-end;
                    padding-right: 1em;
                    user-select: none;
                }
            }

            .textarea {
                appearance: none;
                width: 100%;
                height: 100%;
                padding: 1em 2em 4.5rem 2em;
                text-align: center;
                outline: none;
                overflow-y: auto;
                overflow-x: hidden;
                border: none;
                background: none;
                font-size: inherit;
                line-height: 1.8em;
                resize: none;
                color: rgb(236, 213, 222);
                font-weight: normal;
                font-family: "Lyrics";

                &.font-xs {
                    font-size: 12px;
                }

                &.font-s {
                    font-size: 14px;
                }

                &.font-m {
                    font-size: 16px;
                }

                &.font-l {
                    font-size: 20px;
                }

                &.font-xl {
                    font-size: 26px;
                }
                :global(.section) {
                    margin: 10px 0;
                    color: rgb(179, 146, 179);
                    background: rgb(73, 70, 70, 0.5);
                    border-radius: 5px;
                    padding: 3px 5px;
                    display: inline-block;
                    line-height: normal;
                }
            }
        }
    }
</style>
