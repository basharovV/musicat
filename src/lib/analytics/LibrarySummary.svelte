<script lang="ts">
    import type { Song } from "../../App";
    import LL from "../../i18n/i18n-svelte";

    export let songs: Song[] = [];

    // ── Logic ──────────────────────────────────────────────────────────────────

    $: totalSongs = songs.length;
    $: uniqueAlbums = new Set(songs.map((s) => s.albumId)).size;
    $: uniqueArtists = new Set(songs.map((s) => s.artist)).size;
    $: uniqueGenres = new Set(songs.flatMap((s) => s.genre || [])).size;

    $: longestTrack = songs.length
        ? [...songs].reduce((prev, curr) =>
              parseDurationToSeconds(curr.duration) >
              parseDurationToSeconds(prev.duration)
                  ? curr
                  : prev,
          )
        : null;

    function parseDurationToSeconds(duration: string = "0:00"): number {
        const parts = duration.split(":").map(Number);
        return parts.reduce((acc, time) => 60 * acc + time, 0);
    }
</script>

<h2>
    {$LL.analytics.summary.title
        ? $LL.analytics.summary.title()
        : "Library Summary"}
</h2>

<section class="stats-list">
    <div class="row">
        <span class="label">{$LL.analytics.summary.totalSongs()}</span>
        <span class="value">{totalSongs.toLocaleString()}</span>
    </div>
    <div class="row">
        <span class="label">{$LL.analytics.summary.albums()}</span>
        <span class="value">{uniqueAlbums.toLocaleString()}</span>
    </div>
    <div class="row">
        <span class="label">{$LL.analytics.summary.artists()}</span>
        <span class="value">{uniqueArtists.toLocaleString()}</span>
    </div>
    <div class="row">
        <span class="label">{$LL.analytics.summary.genres()}</span>
        <span class="value">{uniqueGenres.toLocaleString()}</span>
    </div>
    {#if longestTrack}
        <div class="row">
            <span class="label">{$LL.analytics.summary.longestTrack()}</span>
            <div class="value track">
                <span class="truncate">{longestTrack.title}</span>
                <span class="duration">{longestTrack.duration}</span>
            </div>
        </div>
    {/if}
</section>

<style lang="scss">
    /* Styles remain the same, just ensure they are scoped */
    .stats-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        width: 100%;
    }

    .row {
        display: flex;
        align-items: baseline;
        padding: 0.25rem 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .label {
        flex: 0 0 35%;
        color: var(--text-secondary);
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .value {
        flex: 1;
        font-weight: 600;
        font-size: 0.9rem;
        display: flex;
        justify-content: flex-start;
    }

    .track {
        display: flex;
        justify-content: space-between;
        gap: 0.5rem;
        overflow: hidden;
        width: 100%;
    }

    .truncate {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .duration {
        font-family: monospace;
        opacity: 0.7;
        flex-shrink: 0;
    }
</style>
