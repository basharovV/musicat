<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    import type { GetHTMLResponse } from "../../App";
    import wtf, { type Document } from "wtf_wikipedia";
    import type { Section } from "wtf_wikipedia";
    import wtfHtml from "wtf-plugin-html";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import { getWikipediaUrlForArtist } from "../../data/WikipediaAPI";
    import { currentSong, isWikiOpen } from "../../data/store";
    import { fade } from "svelte/transition";
    import Icon from "../ui/Icon.svelte";
    wtf.extend(wtfHtml);

    let wikiResult: GetHTMLResponse;
    let wtfResult: Document;
    let error;
    let previousArtist = null;
    let isLoading = false;

    $: sections = wtfResult?.sections();

    async function getWiki() {
        isLoading = true;
        try {
            if (!$currentSong || previousArtist === $currentSong.artist) return;
            const url = await getWikipediaUrlForArtist($currentSong.artist);
            wikiResult = await invoke<GetHTMLResponse>("get_wikipedia", {
                event: {
                    url
                }
            });
            console.log("result", wikiResult);
            error = null;
        } catch (err) {
            error = err;
        } finally {
            isLoading = false;
            previousArtist = $currentSong?.artist;
        }
    }

    async function getWikiWtf() {
        isLoading = true;
        try {
            if (!$currentSong || previousArtist === $currentSong.artist) return;
            const url = await getWikipediaUrlForArtist($currentSong.artist);
            const result = await wtf.fetch(url);
            // Check if array
            if (result instanceof Array) {
            } else {
                console.log("result", result.sections());
                wtfResult = result;
            }

            error = null;
        } catch (err) {
            error = err;
        } finally {
            isLoading = false;
            previousArtist = $currentSong?.artist;
        }
    }

    $: if ($currentSong) {
        // getWikiWtf();
        getWiki();
    }
</script>

<div class="container">
    {#if isLoading}
        <p transition:fade={{ duration: 200 }}>Loading...</p>
    {:else}
        <div class="scroll-container" transition:fade={{ duration: 200 }}>
            <header>
                <Icon
                    icon="material-symbols:close"
                    onClick={() => {
                        $isWikiOpen = false;
                    }}
                />
                <p>Fetched from Wikipedia</p>
            </header>
            <div class="content">
                <!-- {#if wtfResult}
                {#each sections as section}
                    <h2>{section.title()}</h2>
                    {#each section.paragraphs() as paragrah}
                        {@html paragrah.html()}
                        <br />
                        <br />
                    {/each}
                {/each}
            {/if} -->

                {#if wikiResult}
                    {@html wikiResult.html}
                {/if}
            </div>
        </div>
    {/if}

    <ShadowGradient type="bottom" />
</div>

<style lang="scss">
    .container {
        height: 100%;
        width: 100%;
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        border-radius: 5px;
        border-left: 0.7px solid #ffffff2a;
        border-right: 0.7px solid #ffffff2a;
        margin: 5px 0 0 0;
        background-color: #ffffff1b;

        header {
            position: sticky;
            display: flex;
            justify-content: space-between;
            top: 0;
            padding: 1em;
            z-index: 10;
            /* background-color: var(--overlay-bg); */
            backdrop-filter: blur(10px) brightness(0.95);
            border-bottom: 0.7px solid
                color-mix(in srgb, var(--inverse) 70%, transparent);

            p {
                background-color: #ffffff1b;
                margin: 0;
                width: fit-content;
                padding: 0 1em;
                border-radius: 5px;
                &::before {
                    content: "→ ";
                }
            }
        }

        .scroll-container {
            width: 100%;
            overflow-y: auto;
            pointer-events: all;
            display: flex;
            flex-direction: column;
            user-select: none;
        }

        .content {
            padding: 1em;
            text-align: start;
            color: var(--text);
            :global(.hatnote),
            :global(.infobox) {
                display: none;
            }
            :global(p),
            :global(span) {
                font-size: 16px;
            }
        }
    }
    iframe {
        height: 100%;
    }
</style>
