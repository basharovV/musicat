<script lang="ts">
    import type {
        ArtistContentItem,
        ArtistFileItem,
        ArtistLinkItem
    } from "src/App";

    import ContentBlock from "./ContentBlock.svelte";
    import FileBlock from "./FileBlock.svelte";
    import LinkBlock from "./LinkBlock.svelte";
    import LyricsBlock from "./LyricsBlock.svelte";

    const items: ArtistContentItem[] = [
        {
            name: "Chaos Theory",
            type: "lyrics",
            tags: ["Lyrics"],
            filepath: "lyrics/someplace.txt",
            filename: "someplace.txt"
        },
        {
            name: "Cool guitar riff in Em.aiff",
            type: "file",
            tags: [],
            filepath: "/Media/riff.wav",
            filename: "riff.wav"
        },
        {
            name: "Deja Vu rehearsal",
            type: "link",
            url: "https://www.youtube.com/watch?v=3xOD2pT4xBQ",
            tags: ["Deja Vu", "Rehearsal"]
        },
        {
            name: "Live in L.A",
            type: "link",
            url: "https://www.youtube.com/watch?v=8a3qAp81vY8",
            tags: ["Julian Lage", "Live"]
        },
        {
            name: "Piano progression jazzy.mp3",
            type: "file",
            tags: [],
            filepath: "/Media/pianoprogression.wav",
            filename: "pianoprogression.wav"
        },

        {
            name: "Classical thing",
            type: "file",
            tags: [],
            filepath: "/Media/classicalthing.mp4",
            filename: "classicalthing.mp4"
        },
        {
            name: "Random lyrics",
            type: "lyrics",
            tags: ["Lyrics"],
            filepath: "lyrics/random.txt",
            filename: "random.txt"
        }
    ];
</script>

<div class="legend">
    <div class="lyrics">
        <iconify-icon icon="bi:file-earmark-text" />
        <p>Lyrics</p>
    </div>
    <div class="audio">
        <iconify-icon icon="bi:file-earmark-play" />
        <p>Audio</p>
    </div>
    <div class="video">
        <iconify-icon icon="dashicons:editor-video" />
        <p>Video</p>
    </div>
    <div class="link">
        <iconify-icon icon="akar-icons:link-chain" />
        <p>Link</p>
    </div>
</div>
<div class="scrapbook">
    {#each items as item (item.name)}
        <div class="item">
            {#if item.type === "file"}
                <FileBlock {item} />
            {:else if item.type === "link"}
                <LinkBlock {item} />
            {:else}
                <LyricsBlock {item} />
            {/if}
        </div>
    {/each}
</div>

<style lang="scss">
    .legend {
        display: inline-flex;
        gap: 10px;
        margin-bottom: 1em;
        > div {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            p {
                font-size: 13px;
                margin: 0;
                color: rgb(171, 171, 171);
            }
            &.lyrics > iconify-icon {
                color: rgb(212, 212, 66);
            }
            &.audio > iconify-icon{
                color: rgb(199, 69, 199);
            }
            &.video > iconify-icon{
                color: rgb(224, 72, 72);
            }
            &.link > iconify-icon{
                color: rgb(70, 227, 227);
            }
        }
    }
    .scrapbook {
        display: flex;
        flex-direction: row;
        gap: 0.6em 0.6em;
        flex-wrap: wrap;
    }
    .item {
        height: auto;
        width: auto;
    }
</style>
