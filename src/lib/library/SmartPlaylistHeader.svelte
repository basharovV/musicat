<script lang="ts">
    import { onMount } from "svelte";
    import {
        forceRefreshLibrary,
        isQueueOpen,
        isQueueShowing,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isSmartQueryValid,
        isSidebarOpen,
        isSidebarShowing,
        os,
        queueDuration,
        selectedPlaylistFile,
        selectedSmartQuery,
        smartQuery,
        smartQueryInitiator,
        uiView,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import AlbumOptions from "../albums/AlbumOptions.svelte";
    import SmartQuery from "../smart-query/Query";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import Input from "../ui/Input.svelte";

    export let selectedQuery;

    let durationText;
    let element: HTMLElement;

    $: innerWidth = 800;
    $: showAllOptions = innerWidth >= 1200;

    $: if ($queueDuration) {
        durationText = secondsToFriendlyTime($queueDuration);
    } else {
        durationText = null;
    }

    onMount(() => {
        const resizeObserver = new ResizeObserver(() => {
            innerWidth = element.getBoundingClientRect().width;
        });

        resizeObserver.observe(element);

        return () => resizeObserver.unobserve(element);
    });

    function closeSmartPlaylist() {
        if ($smartQueryInitiator === "library-cell") {
            $forceRefreshLibrary = true;
            $isSmartQueryBuilderOpen = false;
            $uiView = "library";
        } else {
            $isSmartQueryBuilderOpen = false;

            if ($smartQueryInitiator.startsWith("smart-query:")) {
                $selectedSmartQuery = $smartQueryInitiator.substring(12);
            }
        }
    }

    async function editSmartPlaylist() {
        $smartQuery = new SmartQuery(selectedQuery);

        $isSmartQueryValid = true;
        $isSmartQueryBuilderOpen = true;
        $isSmartQuerySaveUiOpen = false;
    }

    function newSmartPlaylist() {
        $smartQueryInitiator = `smart-query:${$selectedSmartQuery}`;
        $selectedSmartQuery = null;
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
        let id = await $smartQuery.save();
        // Close the builder UI and set the current selected query to the one we just saved
        $isSmartQueryBuilderOpen = false;
        $selectedSmartQuery = `~usq:${id}`;
        $selectedPlaylistFile = null;
        $smartQuery.reset();
    }
</script>

<div
    class="header"
    class:shift-controls={!$isSidebarShowing &&
        !$isQueueShowing &&
        $os === "macos"}
    bind:this={element}
>
    <div class="left">
        {#if !$isSmartQueryBuilderOpen}
            <ButtonWithIcon
                size="small"
                icon="material-symbols:add"
                onClick={newSmartPlaylist}
                theme="active"
            />
            {#if $selectedSmartQuery?.startsWith("~usq:")}
                <ButtonWithIcon
                    size="small"
                    icon="material-symbols:edit-outline"
                    onClick={editSmartPlaylist}
                    theme="active"
                />
            {/if}
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
    </div>
    <div class="center">
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
                        color={$uiView.startsWith("smart-query")
                            ? "#45fffcf3"
                            : "currentColor"}
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
    </div>
    <div class="right">
        {#if $uiView === "smart-query:icon"}
            <AlbumOptions bind:showAllOptions />
        {/if}
    </div>
</div>

<style lang="scss">
    .header {
        width: 100%;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: space-between;

        &.shift-controls {
            .left {
                margin-left: 70px;
            }
        }
    }

    form {
        &.window-padding {
            padding-left: 70px;
        }
    }
    * {
        user-select: none;
    }
    .left,
    .center,
    .right {
        display: flex;
        gap: 8px;
        flex-direction: row;
        align-items: center;
        justify-content: space-between;
    }
    .center {
        @media only screen and (min-width: 1800px) {
            position: fixed;
            left: 50%;
            transform: translate(-50%, 0%);
        }
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
