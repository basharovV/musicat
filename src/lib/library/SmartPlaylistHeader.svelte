<script lang="ts">
    import {
        loadDynamicPlaylist,
        writeDynamicPlaylist,
    } from "../../data/PlaylistUtils";
    import {
        forceRefreshLibrary,
        isQueueOpen,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isSmartQueryValid,
        isSidebarOpen,
        queueDuration,
        selectedPlaylistFile,
        smartQuery,
        smartQueryInitiator,
        uiView,
        userSettings,
        selectedSmartQuery,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import SmartQuery from "../smart-query/Query";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import Input from "../ui/Input.svelte";
    import { kebabCase } from "lodash-es";
    import { path } from "@tauri-apps/api";
    import type { DynamicPlaylistFile } from "../../App";

    export let selectedQuery;

    let durationText;

    $: if ($queueDuration) {
        durationText = secondsToFriendlyTime($queueDuration);
    } else {
        durationText = null;
    }

    async function closeSmartPlaylist() {
        if ($smartQueryInitiator === "library-cell") {
            $forceRefreshLibrary = true;
            $isSmartQueryBuilderOpen = false;
            $uiView = "library";
        } else {
            $isSmartQueryBuilderOpen = false;

            if ($smartQueryInitiator.startsWith("smart-query:")) {
                const query = $smartQueryInitiator.substring(12);

                if (query === "favourites" || query === "recentlyAdded") {
                    $selectedSmartQuery = query;
                } else {
                    $selectedPlaylistFile = await loadDynamicPlaylist(query);
                }
            }
        }
    }

    async function editSmartPlaylist() {
        $smartQuery = $selectedPlaylistFile.query;

        $isSmartQueryValid = true;
        $isSmartQueryBuilderOpen = true;
        $isSmartQuerySaveUiOpen = false;
    }

    function newSmartPlaylist() {
        if ($selectedSmartQuery) {
            $smartQueryInitiator = `smart-query:${$selectedSmartQuery}`;
            $selectedSmartQuery = null;
        } else {
            $smartQueryInitiator = `smart-query:${$selectedPlaylistFile.path}`;
            $selectedPlaylistFile = null;
        }

        $smartQuery = new SmartQuery();
        $isSmartQueryValid = false;
        $isSmartQueryBuilderOpen = true;
        $isSmartQuerySaveUiOpen = false;
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

    async function save() {
        if ($selectedPlaylistFile) {
            const playlist = $selectedPlaylistFile as DynamicPlaylistFile;

            $smartQuery.toDynoPL(playlist.schema);

            await writeDynamicPlaylist(playlist);
        } else {
            const name = kebabCase($smartQuery.name);
            const filepath = await path.join(
                $userSettings.playlistsLocation,
                `${name}.dynopl.yaml`,
            );
            const schema = $smartQuery.toDynoPL();
            const playlist: DynamicPlaylistFile = {
                title: $smartQuery.name,
                path: filepath,
                schema,
            };

            await writeDynamicPlaylist(playlist);

            $selectedPlaylistFile = playlist;
        }

        // Close the builder UI
        $isSmartQueryBuilderOpen = false;
    }
</script>

{#if $isSmartQueryBuilderOpen}
    <form
        on:submit|preventDefault={save}
        class:window-padding={!$isSidebarOpen && !$isQueueOpen}
    >
        <Input
            bind:value={$smartQuery.name}
            fullWidth
            alt
            placeholder={$LL.smartPlaylists.builder.placeholder()}
        />
    </form>
{:else}
    <h3 class="title">
        <span>
            <Icon
                icon="ic:round-star-outline"
                size={15}
                color={$uiView === "smart-query" ? "#45fffcf3" : "currentColor"}
            /></span
        >&nbsp;{selectedQuery
            ? selectedQuery.name
            : $selectedPlaylistFile?.title}
    </h3>
{/if}
{#if durationText}
    <p class="duration">{durationText}</p>
{/if}
<div class="line" />
<div class="playlist-info">
    <p class="count"></p>
    <!-- TODO -->
</div>
{#if !$isSmartQueryBuilderOpen}
    {#if $selectedPlaylistFile}
        <ButtonWithIcon
            size="small"
            icon="material-symbols:edit-outline"
            onClick={editSmartPlaylist}
            text={$LL.smartPlaylists.editSmartPlaylist()}
            theme="active"
        />
    {/if}
    <ButtonWithIcon
        size="small"
        icon="material-symbols:add"
        onClick={newSmartPlaylist}
        text={$LL.smartPlaylists.newSmartPlaylist()}
        theme="active"
    />
{:else}
    <ButtonWithIcon
        size="small"
        icon="material-symbols:close"
        onClick={closeSmartPlaylist}
        text={$LL.smartPlaylists.builder.close()}
        theme="transparent"
    />
    <ButtonWithIcon
        size="small"
        icon="material-symbols:save-outline"
        onClick={save}
        text={$LL.smartPlaylists.builder.save()}
        disabled={!$isSmartQueryValid || !$smartQuery.isNameSet}
        theme="transparent"
    />
{/if}

<style lang="scss">
    form {
        &.window-padding {
            padding-left: 70px;
        }
    }
    * {
        user-select: none;
    }
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
        background-color: color-mix(in srgb, var(--background) 76%, black);
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
