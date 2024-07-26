<script lang="ts">
    import { draggedAlbum, draggedSongs } from "../../data/store";
    export let x = 0;
    export let y = 0;
    export let show = false;
</script>

{#if show}
    <div style="transform: translateY({y}px) translateX({x}px);">
        {#if $draggedSongs}
            {#if $draggedAlbum}
                <p>{$draggedAlbum?.title} ({$draggedSongs.length} songs)</p>
            {:else if $draggedSongs.length > 1}
                <p>{$draggedSongs.length} songs</p>
            {:else}
                <p>{$draggedSongs[0]?.title}</p>
            {/if}
        {/if}
    </div>
{/if}

<style lang="scss">
    div {
        position: fixed;
        background-color: #20202296;
        backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.23);
        color: white;
        padding: 5px 10px;
        border-radius: 5px;
        z-index: 11;
        pointer-events: none;

        p {
            margin: 0;
        }
    }
</style>
