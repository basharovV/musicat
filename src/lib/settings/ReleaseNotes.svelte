<script lang="ts">
    import { onMount } from "svelte";
    import Post from "../../../RELEASE-NOTES.md";
    onMount(() => {});
</script>

<div><Post /></div>

<style lang="scss">
    div {
        overflow: auto;
        height: 100%;
        text-align: left;
        padding: 1em;
        color: rgb(206, 206, 206);

        :global(li) {
            font-size: 0.9em;
        }
    }
    ul {
        /* border-radius: 5px; */
        /* max-height: 200px; */
        width: 100%;
        height: fit-content;
        color: rgb(212, 212, 212);
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
