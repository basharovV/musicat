<script lang="ts">
    import { onMount } from "svelte";
    import type { IACollection, IAFile, IAItem } from "../../App";
    import {
        getIACollection,
        getIACollections,
        getIAItem
    } from "../../data/InternetArchiveAPI";
    import IaItemPlayer from "../internet-archive/IAItemPlayer.svelte";
    import webAudioPlayer, {
        currentItem,
        currentSrc,
        isIAPlaying
    } from "../player/WebAudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    import IaFileBlock from "../internet-archive/IAFileBlock.svelte";
    import {
        currentIAFile,
        iaCollections,
        iaSelectedCollection,
        iaSelectedCollectionItems,
        iaSelectedItem,
        webPlayerVolume
    } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { open } from "@tauri-apps/plugin-shell";

    let audios;
    let minWidth = 300;
    let container;

    let scrollContainerCollections;
    let isLoadingCollections = false;
    let collections: IACollection[];
    let collectionsPage = 1;

    let scrollContainerCollection;
    let isLoadingCollection = false;
    let collectionPage = 1;
    let rightClickedItem: IAItem | null = null;
    let showMenu = false;
    let pos = { x: 0, y: 0 };

    let isLoadingItem = false;

    onMount(async () => {
        // Fetch / check in memory
        isLoadingCollections = true;
        $iaCollections = await getIACollections(collectionsPage);
        isLoadingCollections = false;
    });

    async function onScrollCollections(ev) {
        let area = ev.target;

        if (area) {
            let scrolledToBottom =
                area.scrollTop + area.offsetHeight >= area.scrollHeight;
            if (scrolledToBottom && !isLoadingCollections) {
                collectionsPage += 1;
                isLoadingCollections = true;
                const newPage = await getIACollections(collectionsPage);
                $iaCollections.push(...newPage);
                $iaCollections = $iaCollections;
                isLoadingCollections = false;
            }
            console.log("scrolled to bottom", scrolledToBottom);
        }
    }

    async function selectCollection(collection: IACollection) {
        isLoadingCollection = true;
        $iaSelectedCollection = collection;
        $iaSelectedCollectionItems = await getIACollection(
            collection.id,
            collectionPage
        );
        isLoadingCollection = false;
    }

    async function onScrollCollection(ev) {
        let area = ev.target;

        if (area) {
            let scrolledToBottom =
                area.scrollTop + area.offsetHeight >= area.scrollHeight;
            if (scrolledToBottom && !isLoadingCollection) {
                collectionPage += 1;
                isLoadingCollection = true;
                const newPage = await getIACollection(
                    $iaSelectedCollection.id,
                    ++collectionPage
                );
                $iaSelectedCollectionItems.push(...newPage);
                $iaSelectedCollectionItems = $iaSelectedCollectionItems;
                isLoadingCollection = false;
            }
            console.log("scrolled to bottom", scrolledToBottom);
        }
    }

    async function selectItem(item: IAItem) {
        $iaSelectedItem = item;
        isLoadingItem = true;
        // Prepare files
        $iaSelectedItem = await getIAItem(item.id);
        isLoadingItem = false;
    }

    async function openOnArchiveOrg() {
        await open(`https://archive.org/details/${rightClickedItem.id}`);
        showMenu = false;
    }
</script>

<div class="container">
    <div class="grid-container" bind:this={container}>
        <header>
            <div class="attribution">
                <h2>Internet Archive</h2>
                <small
                    >public domain music from <a
                        href="https://archive.org/details/audio_music"
                        >archive.org</a
                    ></small
                >
            </div>

            <div class="player">
                <IaItemPlayer />
            </div>

            <div class="volume">
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={$webPlayerVolume}
                    class="slider"
                    id="myRange"
                />
            </div>
        </header>
        <div class="archive-browser">
            <!-- COLUMN 1-->
            <div
                on:scroll={onScrollCollections}
                bind:this={scrollContainerCollections}
            >
                <div
                    class="column-collections"
                    class:show={$iaCollections?.length}
                    style="grid-template-columns: repeat(auto-fit, minmax(100%, 0.1fr));width: 100%;"
                >
                    {#if $iaCollections}
                        {#each $iaCollections as collection}
                            <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                            <!-- svelte-ignore a11y-no-noninteractive-tabindex -->
                            <div
                                role="listitem"
                                tabindex="0"
                                class="item"
                                class:selected={collection?.id ===
                                    $iaSelectedCollection?.id}
                                on:click={() => {
                                    selectCollection(collection);
                                }}
                            >
                                <!-- svelte-ignore a11y-missing-attribute -->
                                <img
                                    src="https://archive.org/services/img/{collection.id}"
                                />
                                <div class="left">
                                    <p class="title">{collection.title}</p>
                                    <div class="info">
                                        <p class="count">
                                            {collection.filesCount} files
                                        </p>
                                        <p class="size">
                                            {(
                                                collection.size /
                                                (1024 * 1000000000)
                                            ).toFixed(2)}
                                            TB
                                        </p>
                                    </div>
                                </div>
                                <p class="description">
                                    {@html collection.description ?? ""}
                                </p>
                            </div>
                        {/each}
                    {/if}
                </div>
                {#if isLoadingCollections}
                    <div>
                        <p>Loading collections...</p>
                        <LoadingSpinner />
                    </div>
                {/if}
            </div>
            <!-- COLUMN 2-->
            <div
                class="column-collection"
                on:scroll={onScrollCollection}
                bind:this={scrollContainerCollection}
            >
                <div class="header">
                    {#if $iaSelectedCollection}
                        <div class="top-row">
                            <img
                                src="https://archive.org/services/img/{$iaSelectedCollection.id}"
                            />
                            <h2>{$iaSelectedCollection.title}</h2>
                        </div>
                        {#if $iaSelectedCollection.description}
                            <p class="description">
                                {$iaSelectedCollection.description}
                            </p>
                        {/if}
                    {:else}
                        <p>Select a collection on the left</p>
                    {/if}
                </div>
                <div class="selected-collection">
                    <div class="list">
                        {#if $iaSelectedCollectionItems}
                            <ul>
                                {#each $iaSelectedCollectionItems as item}
                                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                                    <li
                                        class:selected={item.id ===
                                            $iaSelectedItem?.id}
                                        class:playing={item.id ===
                                            $currentIAFile?.itemId &&
                                            $isIAPlaying}
                                        on:contextmenu|preventDefault={(e) => {
                                            rightClickedItem = item;
                                            pos = {
                                                x: e.clientX,
                                                y: e.clientY
                                            };
                                            showMenu = true;
                                        }}
                                        on:click={() => {
                                            selectItem(item);
                                        }}
                                        on:dblclick={async () => {
                                            await selectItem(item);
                                            await webAudioPlayer.playFromUrl(
                                                $iaSelectedItem.original
                                            );
                                        }}
                                    >
                                        <p>{item.title}</p>
                                        {#if item.id === $currentIAFile?.itemId && $isIAPlaying}
                                            <Icon
                                                icon="f7:speaker-2-fill"
                                                size={14}
                                                color="#00ddff"
                                            />
                                        {/if}
                                    </li>
                                {/each}
                            </ul>
                        {/if}
                    </div>
                    {#if isLoadingCollection}
                        <div>
                            <p>Loading collection...</p>
                            <LoadingSpinner />
                        </div>
                    {/if}
                </div>
            </div>

            <!-- COLUMN 3-->
            <div class="column-audio">
                {#if isLoadingItem}
                    <div class="header">
                        <h2>{$iaSelectedItem?.title}</h2>
                        <p>Loading</p>
                        <div class="loading">
                            <LoadingSpinner />
                        </div>
                    </div>
                {:else if $iaSelectedItem}
                    <div class="header">
                        <h2>{$iaSelectedItem?.title}</h2>
                    </div>
                    <h3>Original</h3>
                    {#if $iaSelectedItem.original}
                        <IaFileBlock file={$iaSelectedItem.original} />
                    {/if}
                    {#if $iaSelectedItem.files}
                        <h3>Files</h3>

                        <div class="files">
                            {#each $iaSelectedItem.files as file}
                                <IaFileBlock {file} />
                            {/each}
                        </div>
                    {/if}

                    <div class="credits">
                        {#if $iaSelectedItem.date}
                            <p class="date">
                                Publication date: <span
                                    >{$iaSelectedItem.date}</span
                                >
                            </p>
                        {/if}
                        {#if $iaSelectedItem.performer}
                            <p>Performed by {$iaSelectedItem.performer}</p>
                        {/if}
                        {#if $iaSelectedItem.writer}
                            <p>Written by {$iaSelectedItem.writer}</p>
                        {/if}
                    </div>
                {:else}
                    <p class="hint">Select an item on the left</p>
                {/if}
            </div>
        </div>

        {#if showMenu}
            <Menu
                {...pos}
                fixed
                onClickOutside={() => {
                    showMenu = false;
                    rightClickedItem = null;
                }}
            >
                <MenuOption
                    text="Open on archive.org"
                    onClick={() => openOnArchiveOrg()}
                />
            </Menu>
        {/if}
    </div>
</div>

<style lang="scss">
    .container {
        position: relative;
        display: grid;
        height: auto;
        grid-template-columns: 1fr;
        grid-template-rows: 1fr;
        margin: 5px 5px 0 0;
        row-gap: 5px;
        border-radius: 5px;
        box-sizing: border-box;
        overflow: hidden;
        /* border: 0.7px solid #ffffff0b; */
    }

    .grid-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        position: absolute;
        top: 0;
        bottom: 0;
        left: 0;
        right: 0;
        header {
            display: grid;
            grid-template-columns: auto 1fr auto;
            align-items: center;
            height: 60px;
            background-color: var(--panel-background);
            border-top: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
            border-left: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
            border-bottom: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
            border-radius: 5px;
            margin-bottom: 5px;

            .volume {
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-self: flex-end;
                padding: 0.5em 1em;
                border-left: 0.7px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);

                input {
                    -webkit-appearance: none;
                    width: 100%;
                    height: 5px;
                    background: #474747d4;
                    outline: none;
                    opacity: 1;
                    margin: auto;
                    border-radius: 3px;
                    -webkit-transition: 0.2s;
                    transition: opacity 0.2s;

                    &::-webkit-slider-thumb {
                        -webkit-appearance: none;
                        appearance: none;
                        width: 20px;
                        height: 20px;
                        background: url("/images/volume-up.svg");
                    }

                    &::-moz-range-thumb {
                        width: 20px;
                        height: 20px;
                        background: #04aa6d;
                    }
                }
            }

            .attribution {
                align-items: center;
                justify-content: center;
                text-align: end;
                padding: 0.5em 1em;

                height: 100%;
                border-right: 0.7px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);
                h3 {
                    white-space: nowrap;
                }
            }

            .player {
                height: 100%;
            }
            h2 {
                margin: 0;
                font-family: monospace;
                letter-spacing: 1px;
                justify-self: flex-start;
            }
            > div {
                display: flex;
                flex-direction: column;
                small,
                p {
                    margin: 0;
                    opacity: 0.6;
                    line-height: initial;
                }
            }
        }
    }

    .archive-browser {
        height: 100%;
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: 1fr;
        gap: 5px;
        overflow: hidden;
        > div {
            height: 100%;
            border-top: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
            border-bottom-left-radius: 5px;
            border-bottom-right-radius: 5px;
            border-left: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
            border-bottom: 0.7px solid
                color-mix(in srgb, var(--inverse) 30%, transparent);
            background-color: var(--panel-background);
            &:not(:nth-child(1)) {
                border-left: 0.7px solid #ffffff2a;
            }
            &:not(:last-child) {
                border-right: 0.7px solid #ffffff2a;
            }
            border-radius: 4px;
            overflow: auto;
        }
        .column-collections {
            width: 100%;
            display: none;
            min-width: 0; // hack to make the grid respect wrap
            opacity: 0;
            transition: opacity 0.15s cubic-bezier(0.455, 0.03, 0.515, 0.955);
            /* background-color: rgb(34, 33, 33); */
            /* background-image: url("images/textures/soft-wallpaper.png"); */
            /* background-repeat: repeat; */

            > .item {
                position: relative;
                width: 100%;
                display: flex;
                height: auto;
                gap: 10px;
                padding: 1em 2em;
                border-top: 0.7px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);
                opacity: 0.8;
                &.selected {
                    background-color: color-mix(
                        in srgb,
                        var(--inverse) 20%,
                        transparent
                    );
                    opacity: 1;
                }

                &:hover {
                    background-color: color-mix(
                        in srgb,
                        var(--inverse) 10%,
                        transparent
                    );
                    opacity: 1;
                }

                img {
                    width: 80px;
                    display: none;
                    border-radius: 4px;
                }

                .left {
                    width: 200px;
                    .info {
                        display: flex;
                        flex-direction: column;
                        p {
                            text-align: left;
                            margin: 0;
                        }
                    }
                }
                .title {
                    font-weight: bold;
                    flex: 20%;
                    text-align: left;
                    margin: 0;
                }
                .description {
                    margin: 0;
                    text-align: left;
                    flex: 50%;
                    color: var(--text-secondary);
                    display: -webkit-box;
                    -webkit-line-clamp: 2;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                    text-overflow: ellipsis;
                }
                .count {
                    opacity: 0.7;
                }
                .size {
                    opacity: 0.5;
                }
            }

            &.show {
                display: grid;
                opacity: 1;
            }
        }

        .column-collection {
            display: flex;
            flex-direction: column;

            .header {
                display: flex;
                flex-direction: column;
                position: sticky;
                top: 0;
                background-color: var(--panel-background);
                backdrop-filter: blur(8px);
                flex-wrap: wrap;
                border-bottom: 0.7px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);
                z-index: 2;
                .top-row {
                    display: flex;
                    width: 100%;

                    img {
                        object-fit: cover;
                        width: 80px;
                        height: auto;
                    }
                    h2 {
                    }
                }
                > .description {
                    width: 100%;
                    display: -webkit-box;
                    -webkit-line-clamp: 2;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                    text-overflow: ellipsis;
                    padding: 0 1em;
                    text-align: left;
                    opacity: 0.7;
                }
            }

            h2 {
                padding: 1em;
                margin: 0;
                text-align: left;
            }
            .list {
                ul {
                    list-style: none;
                    padding: 0;
                    margin: 0;
                }
                li {
                    text-align: left;
                    padding: 0 1em;
                    /* border-bottom: 0.7px solid #ffffff2a; */
                    height: 26px;
                    overflow: hidden;
                    text-overflow: ellipsis;
                    word-wrap: nowrap;
                    display: flex;
                    gap: 4px;
                    opacity: 0.8;
                    &.selected {
                        background-color: color-mix(
                            in srgb,
                            var(--inverse) 20%,
                            transparent
                        );
                        opacity: 1;
                        &.playing {
                            background-color: var(--library-playing-bg);
                            color: var(--library-playing-text);
                        }
                    }

                    &:hover {
                        background-color: color-mix(
                            in srgb,
                            var(--inverse) 10%,
                            transparent
                        );
                        opacity: 1;
                    }
                    &:active {
                        background-color: color-mix(
                            in srgb,
                            var(--inverse) 30%,
                            transparent
                        );
                        opacity: 1;
                    }

                    &.playing {
                        background-color: var(--library-playing-bg);
                        color: var(--library-playing-text);
                        opacity: 1;
                    }
                    p {
                        margin: 0;
                        pointer-events: none;
                        user-select: none;
                    }
                }
            }

            .detail {
                padding: 1em;
            }
        }

        .column-audio {
            .header {
                display: flex;
                flex-direction: column;
                background-color: var(--panel-background);
                padding: 0.5em;
                border-bottom: 0.7px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);
                h2 {
                    padding: 1em;
                    margin: 0;
                    text-align: start;
                }

                .player {
                    margin: 0 5px;
                }

                .loading {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
            }

            .date {
                color: var(--text-secondary);
                span {
                    color: var(--text);
                }
            }

            .files {
                display: grid;
                width: 100%;
                grid-template-columns: repeat(auto-fit, minmax(100%, 0.5fr));
            }

            .credits {
                padding: 1em;
                margin-top: 1em;
                color: grey;
            }

            .hint {
                padding-bottom: 1em;
                border-bottom: 0.7px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);
            }
        }
    }
</style>
