<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import {
        forceRefreshLibrary,
        isTagCloudOpen,
        isTagOrCondition,
        libraryUpdater,
        selectedTags,
        uiView,
    } from "../../data/store";
    import Toggle from "../ui/Toggle.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import LL from "../../i18n/i18n-svelte";

    let splitTags: string[] = [];
    $: {
        // Re-fetch whenever the library changes -- already distinct (the
        // normalized tags table), no client-side dedupe needed.
        $libraryUpdater;
        invoke<string[]>("get_distinct_track_values", { field: "tags" }).then(
            (tags) => (splitTags = tags),
        );
    }

    function selectTag(tag: string) {
        if ($selectedTags?.has(tag)) {
            $selectedTags.delete(tag);
        } else {
            $selectedTags.add(tag);
        }
        $selectedTags = $selectedTags;
    }
</script>

<div class="tag-cloud">
    <div class="tags">
        {#each splitTags as tag}
            <div
                class="tag"
                class:selected={$selectedTags.has(tag)}
                on:click={() => selectTag(tag)}
            >
                <p>{tag}</p>
            </div>
        {/each}
    </div>
    <div class="options">
        <Toggle bind:checked={$isTagOrCondition} textOff="AND" textOn="OR" />
    </div>
    <div class="close">
        <ButtonWithIcon
            size="small"
            icon="material-symbols:close"
            onClick={() => {
                $selectedTags = new Set();
                $forceRefreshLibrary = true;
                $isTagCloudOpen = false;
            }}
            text={$LL.tagCloud.close()}
            theme="transparent"
        />
    </div>
</div>

<style lang="scss">
    .tag-cloud {
        border-radius: 8px;
        margin-right: 5px;

        background-color: var(--accent-softest);
        border: 1.5px solid var(--accent-soft);
        color: var(--primary);
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: center;
    }

    .tags {
        padding: 0.5em 1em 0.5em 0.6em;
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        gap: 5px;
        justify-content: stretch;
    }
    .close {
        height: 100%;
        padding: 0.5em;
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        gap: 5px;
        align-items: center;
        justify-content: stretch;
        border-left: 0.75px solid
            color-mix(in srgb, var(--type-bw-inverse) 15%, transparent);
    }

    .options {
        height: 100%;
        display: flex;
        align-items: center;
        padding: 0 1em;
        border-left: 0.75px solid
            color-mix(in srgb, var(--type-bw-inverse) 15%, transparent);
    }

    .tag {
        border-radius: 20px;
        padding: 1px 10px;

        box-sizing: border-box;
        border: 1px solid var(--border);

        &:hover {
            border: 1px solid var(--accent-soft);
        }

        &:active {
            opacity: 0.5;
        }

        &.selected {
            border-color: var(--accent-soft);
            background-color: var(--accent-softest);
        }

        p {
            height: 20px;
            margin: 0;
            user-select: none;
            line-height: 20px;
            vertical-align: middle;
            cursor: default;
            font-size: 13px;
            position: relative;
            bottom: 1px;
        }
    }
</style>
