<script lang="ts">
    import { liveQuery } from "dexie";
    import type { ArtistProject, Song, SongProject } from "src/App";
    import { db } from "../../data/db";
    import {
        isDraggingExternalFiles,
        isScrapbookShown,
        selectedArtistId
    } from "../../data/store";

    import { blur } from "svelte/transition";
    import ArtistInfo from "../your-music/ArtistInfo.svelte";
    import ContentDropzone from "../your-music/ContentDropzone.svelte";
    import Music from "../your-music/Music.svelte";
    import Scrapbook from "../your-music/Scrapbook.svelte";
    import YourArtists from "../your-music/YourArtists.svelte";
    let selectedSong: Song;
    let selectedSongProject: SongProject;
    let songProjectSelection;

    let scrapbookSize = 350;
    const SCRAPBOOK_MIN_SIZE = 300;

    $: selectedArtist = liveQuery<ArtistProject>(async () => {
        const selectedArtist = await db.artistProjects.get($selectedArtistId);
        return selectedArtist;
    });
    let selectedTab;

    let container: HTMLElement;
    let isResizing = false;
    let showCloseScrapbookPrompt = false;
    function onResize(e: MouseEvent) {
        e.stopPropagation();
        e.preventDefault();
        const containerWidth = window.innerWidth;
        console.log("container width", containerWidth);
        console.log("clientX", e.pageX);
        if (containerWidth - e.clientX > SCRAPBOOK_MIN_SIZE) {
            showCloseScrapbookPrompt = false;
            scrapbookSize = containerWidth - e.clientX;
        } else if (containerWidth - e.clientX < SCRAPBOOK_MIN_SIZE - 10) {
            showCloseScrapbookPrompt = true;
        }
    }
    function startResizeListener() {
        isResizing = true;
        container.addEventListener("mousemove", onResize);
        document.addEventListener("mouseup", stopResizeListener);
    }
    function stopResizeListener() {
        isResizing = false;
        container.removeEventListener("mousemove", onResize);
        if (showCloseScrapbookPrompt) {
            $isScrapbookShown = false;
            showCloseScrapbookPrompt = false;
        }
    }
</script>

<container
    bind:this={container}
    class:scrapbook-open={$isScrapbookShown}
    style={$isScrapbookShown
        ? `grid-template-columns: 1fr ${scrapbookSize}px;`
        : "grid-template-columns: 1fr;"}
>
    {#if $isDraggingExternalFiles}
        <ContentDropzone songProject={selectedSongProject} />
    {/if}
    <header>
        <YourArtists bind:selectedTab selectedArtist={$selectedArtist} />
    </header>
    {#if $isScrapbookShown}
        <section class="scrapbook">
            <resize-handle
                on:mousedown={startResizeListener}
                class:resizing={isResizing}
            />
            <div>
                <h2>Scrapbook</h2>
            </div>
            <div class="content">
                {#if showCloseScrapbookPrompt}
                    <div
                        class="close-scrapbook-prompt"
                        transition:blur={{ duration: 100 }}
                    >
                        <h2>Close scrapbook</h2>
                    </div>
                {/if}
                <Scrapbook />
            </div>
        </section>
    {/if}
    <section class="content">
        {#if selectedTab === "Music"}
            <Music {selectedArtist} />
        {:else if selectedTab === "Info"}
            <ArtistInfo artist={$selectedArtist} />
        {/if}
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

    h2 {
        margin: 0;
        font-family: "Snake";
        font-size: 3em;
        color: #bbb9b9;
    }

    container {
        text-align: left;
        display: grid;
        grid-template-columns: 1fr;
        grid-template-rows: auto 1fr;
        overflow: hidden;

        border: 0.7px solid #ffffff2a;
        margin: 5px 5px 5px 0;
        border-radius: 5px;
        overflow: hidden;
        background-color: #0d0c0c2a;

        &.scrapbook-open {
            /* grid-template-columns: 1fr 350px; */ // INLINE STYLED
            grid-template-rows: auto 1fr;
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
            grid-column: 1 / 2;
            display: flex;
            flex-direction: row;
            gap: 2em;
            align-items: center;
            padding: 0 0.7em 0 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.093);
            /* border-left: 1px solid rgba(255, 255, 255, 0.093); */
            background-color: rgba(0, 0, 0, 0.259);

            .bg {
                position: absolute;
                left: 0;
                bottom: 0;
                right: 0;
                top: 0;
                width: 100%;
                object-fit: cover;
                object-position: -0px -320px;
                z-index: -1;
            }
            h2 {
                margin: 0.2em 0;
            }
        }

        .artist-info {
            grid-row: 2;
            grid-column: 1 / 3;
            padding: 0.7em 2em;
            border-bottom: 1px solid rgba(255, 255, 255, 0.093);

            p {
                margin: 0;
            }
        }

        .scrapbook {
            background-color: transparent;
            grid-row: 1 / 3;
            grid-column: 2;
            height: 100%;
            border-left: 1px solid rgba(255, 255, 255, 0.093);
            overflow: visible;
            position: relative;

            .close-scrapbook-prompt {
                position: absolute;
                top: 0;
                bottom: 0;
                left: 0;
                right: 0;
                display: flex;
                align-items: center;
                justify-content: center;
                color: white;
                background-color: #33303ce2;
                backdrop-filter: blur(5px);
                z-index: 10;
            }
            resize-handle {
                display: block;
                position: absolute;
                left: -2px;
                top: 0;
                bottom: 0;
                width: 4px;
                &:hover,
                &.resizing {
                    background-color: #5123dd;
                }

                cursor: ew-resize;
            }
            div {
                padding: 2em;
            }
            .content {
                padding-top: 0;
            }
        }
    }

    .content {
        width: 100%;
        grid-column: 1 / 2;
        grid-row: 2 / 4;
    }
    hr {
        border-top: 1px solid rgba(255, 255, 255, 0.093);
        border-bottom: none;
        border-left: none;
        border-right: none;
    }
</style>
