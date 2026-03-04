<script lang="ts">
    import { onMount } from "svelte";
    import Post from "../../../RELEASE-NOTES.md";
    let container: HTMLDivElement;
    onMount(() => {
        // Add target blank to all links
        const links = container.querySelectorAll("a");
        links.forEach((link) => {
            link.setAttribute("target", "_blank");
        });
    });
</script>

<div bind:this={container}>
    <Post />
</div>

<style lang="scss">
    div {
        height: auto;
        text-align: left;
        padding: 1em 2em;
        color: rgb(218, 215, 215);
        position: relative;
        letter-spacing: 0.6px;
        :global(ul) {
            list-style-type: disc;
            padding-inline-start: 1.5em; // This creates the indent for every level

            :global(ul) {
                list-style-type: circle; // Different bullet for sub-items
                margin-top: 0.5em;
            }
        }
        :global(li) {
            font-size: 1em;
        }
        :global(h3) {
            position: sticky;
            top: 40px;
            background: rgb(74, 72, 77);
            z-index: 3;
            padding-bottom: 10px;
            border-bottom: 1px solid rgba(147, 147, 147, 0.336);
        }
        :global(strong) {
            color: rgb(187, 176, 240);
            letter-spacing: 0.7px;
        }
    }
</style>
