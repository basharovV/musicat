<script lang="ts">
    import { open } from "@tauri-apps/plugin-shell";
    import type { PlaylistFile } from "../../App";
    import { parsePlaylist, writePlaylist } from "../../data/M3UUtils";
    import {
        queriedSongs,
        query,
        queueDuration,
        selectedPlaylistFile,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";

    export let playlist: PlaylistFile;

    let tracks = [];

    $: if (playlist) {
        readPlaylist();
    }

    async function readPlaylist() {
        tracks = await parsePlaylist(playlist);
    }

    let durationText;
    $: if ($queueDuration) {
        durationText = secondsToFriendlyTime($queueDuration);
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

    async function openPlaylistDir() {
        await open(playlist.path.replace(playlist.title + ".m3u", ""));
    }
</script>

<h3 class="title">{playlist.title}</h3>

<Icon
    icon="material-symbols:folder"
    onClick={() => {
        openPlaylistDir();
    }}
/>
<div class="line" data-tauri-drag-region />
{#if $queriedSongs.length}
    <div
        class="file-order-hint-container"
        class:is-temp={$query.orderBy !== "none"}
    >
        <div class="file-order-hint">
            <Icon
                icon="qlementine-icons:sort-desc-16"
                size={14}
                color={$query.orderBy !== "none" ? "#ff8c3a" : undefined}
            />
        </div>

        <p>
            {$query.orderBy === "none"
                ? $LL.library.orderHint()
                : $LL.library.orderHintTemp()}
        </p>
    </div>
{/if}
{#if $query.orderBy !== "none"}
    <ButtonWithIcon
        size="small"
        icon="material-symbols:save-outline"
        text="Save order"
        disabled={$query.orderBy === "none"}
        onClick={async () => {
            await writePlaylist($selectedPlaylistFile, $queriedSongs);
            $query.orderBy = "none";
        }}
    />
{/if}
<div class="playlist-info">
    <p class="count">
        {#if tracks.length === 0}
            No tracks
        {:else}
            {tracks.length} track{tracks.length > 1 ? "s" : ""}
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
    * {
        user-select: none;
    }
    .title {
        font-size: 1.1em;
        line-height: initial;
        position: relative;
        background-color: color-mix(in srgb, var(--background) 76%, black);

        border-radius: 5px;
        height: 100%;
        display: flex;
        padding: 0 8px;
        align-items: center;
    }

    .description {
        opacity: 0.5;
    }

    .line {
        height: 100%;
        background-color: #00000022;
        width: 100%;
        border-radius: 5px;
        flex: 1;
        visibility: hidden;
    }
    .file-order-hint-container {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 6px;
        &.is-temp {
            .file-order-hint {
                background: none;
            }
            p {
                color: #ff8c3a;
            }
        }
        .file-order-hint {
            display: flex;
            flex-direction: row;
            align-items: center;
            padding: 4px;
            border-radius: 5px;
            gap: 8px;
            background: var(--accent-secondary);
            color: color-mix(
                in srgb,
                var(--type-bw-inverse),
                var(--inverse) 20%
            );
            p {
                margin: 0;
            }
        }
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
