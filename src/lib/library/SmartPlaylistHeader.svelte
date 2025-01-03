<script lang="ts">
    import type { Playlist } from "../../App";
    import { db } from "../../data/db";
    import { findQuery } from "../../data/SmartQueries";
    import {
        forceRefreshLibrary,
        isQueueOpen,
        isSidebarOpen,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isSmartQueryValid,
        queueDuration,
        selectedPlaylistFile,
        selectedSmartQuery,
        smartQuery,
        smartQueryInitiator,
        uiView
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import Input from "../ui/Input.svelte";

    export let selectedQuery;

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

    async function save() {
        let id = await $smartQuery.save();
        // Close the builder UI and set the current selected query to the one we just saved
        $isSmartQueryBuilderOpen = false;
        $selectedSmartQuery = `~usq:${id}`;
        $selectedPlaylistFile = null
        $smartQuery.reset();
    }
</script>

{#if $isSmartQueryBuilderOpen}
    <form on:submit|preventDefault={save} class:window-padding={!$isSidebarOpen && !$isQueueOpen}>
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
        >&nbsp;{selectedQuery?.name}
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
    <div class="button-container">
        <button
            on:click={() => {
                $isSmartQueryBuilderOpen = true;
                $isSmartQuerySaveUiOpen = false;
            }}>{$LL.smartPlaylists.newSmartPlaylist()}</button
        >
    </div>
{:else}
    <ButtonWithIcon
        size="small"
        icon="material-symbols:close"
        onClick={() => {
            if ($smartQueryInitiator === "library-cell") {
                $forceRefreshLibrary = true;
                $isSmartQueryBuilderOpen = false;
                $uiView = "library";
            } else {
                $isSmartQueryBuilderOpen = false;
                // $uiView = "smart-query";
            }
        }}
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

    .button-container {
        padding: 5px;
    }
    button {
        background-color: var(--smart-playlist-button-bg);
        border-radius: 4px;
        height: auto;
        padding: 0.3em 1em;
        p {
            margin: 0;
        }

        &:disabled {
            background-color: var(--smart-playlist-button-disabled-bg);
            color: var(--smart-playlist-button-disabled);
        }
    }
</style>
