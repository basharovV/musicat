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
        padding: 1em;
        color: rgb(218, 215, 215);
        position: relative;
        letter-spacing: 0.6px;

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
    ul {
        /* border-radius: 5px; */
        /* max-height: 200px; */
        width: 100%;
        height: fit-content;
        color: rgb(225, 222, 222);
        font-size: 1em;
        padding: 0;
        margin: 0;

        > h3 {
            position: sticky;
            top: 0;
            background-color: #333333;
            padding: 0.2em;
            margin: 0;
            z-index: 5;
        }
        section {
            padding: 1em 2em;
            i {
                opacity: 0.3;
                margin-left: 0.2em;
                font-size: 14px;
            }
            li,
            h3 {
                width: fit-content;
                text-align: left;
            }

            li {
                margin-inline-start: 1em;
                text-align: left;
                margin-top: 0.6em;
            }
        }
    }
</style>
