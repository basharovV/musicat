<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { db } from "../../data/db";
    import {
        bottomBarNotification,
        isSmartQueryBuilderOpen,
        playlistDuration,
        query,
        selectedPlaylistFile,
        selectedSmartQuery,
        toDeletePlaylist,
        uiView,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import type { Song } from "../../App";

    let playlist = $toDeletePlaylist;
    let isDeleting = false;
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

    /**
     * Playlists are M3U files, and in special cases (like the to-delete playlist, a separate table)
     * @param tracks
     */
    async function deleteTracksFromPlaylist(tracks: Song[]) {
        // Delete from internal playlist (currently only used for the "to-delete" playlist)
        const toDelete = await db.internalPlaylists.get($toDeletePlaylist.id);
        tracks.forEach((t) => {
            const trackIdx = toDelete.tracks.findIndex((pt) => pt === t.id);
            toDelete.tracks.splice(trackIdx, 1);
        });
        await db.internalPlaylists.put(toDelete);
    }

    async function deleteTracksFromDB(tracks: Song[]) {
        tracks.forEach((t) => {
            db.songs.delete(t.id);
        });
    }

    async function deleteTracksFromFileSystem(tracks: Song[]) {
        // Delete from file system (ie. move to trash)
        await invoke("delete_files", {
            event: {
                files: tracks.map((t) => t.path),
            },
        });
    }

    async function deleteAll() {
        isDeleting = true;
        // Get bulk
        const songs = await db.songs.bulkGet($toDeletePlaylist.tracks);
        const count = songs.length;

        $bottomBarNotification = {
            text: $LL.toDelete.notification.deleting(count),
            timeout: 2000,
        };

        await deleteTracksFromFileSystem(songs);
        await deleteTracksFromPlaylist(songs);
        await deleteTracksFromDB(songs);
        db.internalPlaylists.delete($toDeletePlaylist.id);

        $bottomBarNotification = {
            text: $LL.toDelete.notification.deleted(count),
            timeout: 2000,
        };

        isDeleting = false;

        $uiView = "library";
        $isSmartQueryBuilderOpen = false;
        $selectedPlaylistFile = null;
        $selectedSmartQuery = null;
        $query.orderBy = $query.libraryOrderBy;
    }

    async function keepAll() {
        db.internalPlaylists.delete($toDeletePlaylist.id);

        $uiView = "library";
        $isSmartQueryBuilderOpen = false;
        $selectedPlaylistFile = null;
        $selectedSmartQuery = null;
        $query.orderBy = $query.libraryOrderBy;
    }
</script>

<h3 class="title">{$LL.toDelete.title()}</h3>

<p class="description">{$LL.toDelete.description()}</p>

<div class="line" data-tauri-drag-region />
<ButtonWithIcon
    size="small"
    theme="translucent"
    icon="charm:tick"
    text={$LL.toDelete.keepAllBtn()}
    disabled={$toDeletePlaylist?.tracks.length === 0}
    onClick={async () => {
        keepAll();
    }}
/>
<ButtonWithIcon
    size="small"
    icon="ant-design:delete-outlined"
    text={$LL.toDelete.deleteAllBtn()}
    isLoading={isDeleting}
    disabled={$toDeletePlaylist?.tracks.length === 0}
    onClick={async () => {
        deleteAll();
    }}
/>
<div class="playlist-info">
    <p class="count">
        {#if $toDeletePlaylist?.tracks.length === 0}
            No tracks
        {:else}
            {$toDeletePlaylist?.tracks.length} track{$toDeletePlaylist?.tracks
                .length > 1
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
