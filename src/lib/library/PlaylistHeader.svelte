<script lang="ts">
    import type { Playlist } from "../../App";
    import { db } from "../../data/db";
    import { playlistDuration, selectedPlaylistId } from "../../data/store";

    export let playlist: Playlist;

    let durationText;
    $: if ($playlistDuration) {
        durationText = secondsToFriendlyTime($playlistDuration);
    } else {
        durationText = null;
    }

    // For playlists header only
    function secondsToFriendlyTime(seconds) {
        if (seconds < 0) return "Invalid input"; // handle negative input

        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const remainingSeconds = seconds % 60;

        let result = [];

        if (hours > 0) result.push(hours + "h");
        if (minutes > 0) result.push(minutes + "m");
        if (remainingSeconds > 0 || result.length === 0)
            result.push(remainingSeconds.toFixed(0) + "s");

        return result.join(" ");
    }
</script>

<h3 class="title">{playlist.title}</h3>
<div class="line"/>
<div class="playlist-info">
    <p class="count">
        {#if playlist.tracks.length === 0}
            No tracks
        {:else}
            {playlist.tracks.length} track{playlist.tracks.length > 1
                ? "s"
                : ""}
        {/if}
    </p>
    {#if durationText}
        <p class="duration">{durationText}</p>
    {/if}
</div>

<style lang="scss">
    .label {
        font-size: 1em;
        color: rgb(104, 96, 113);
        line-height: initial;
        margin: 0;
        font-weight: 600;
        margin-left: -58px;
    }
    .title {
        font-size: 1.1em;
        line-height: initial;
        position: relative;
        background-color: #242026;
        border-radius: 5px;
        height: 100%;
        display: flex;
        padding: 0 8px;
        align-items: center;
    }

    .line {
        height: 100%;
        background-color: #00000022;
        width: 100%;
        border-radius: 5px;
        flex: 1;
        visibility: hidden;
    }

    .playlist-info {
        display: flex;
        flex-direction: row;
        align-items: center;
        padding: 0 8px;
        gap: 8px;
    }

    .count {
        font-size: 0.9em;
        margin: 0;
        line-height: initial;
        opacity: 0.5;
        width: max-content;
    }
    .duration {
        font-size: 0.9em;
        width: max-content;
        margin: 0;
        line-height: initial;
        opacity: 0.5;
        &::before {
            content: "•";
            margin-right: 0.4em;
        }
    }
</style>
