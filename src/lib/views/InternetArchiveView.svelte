<script lang="ts">
    import { onMount } from "svelte";
    import type { IACollection, IAItem } from "../../App";
    import {
        getIACollection,
        getIACollections,
        getIAItem
    } from "../../data/InternetArchiveAPI";
    import IaItemPlayer from "../internet-archive/IAItemPlayer.svelte";
    import {
        currentItem,
        currentSrc,
        isPlaying
    } from "../player/WebAudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";

    let audios;
    let minWidth = 300;
    let container;

    let scrollContainerCollections;
    let isLoadingCollections = false;
    let collections: IACollection[];
    let collectionsPage = 1;

    let scrollContainerCollection;
    let isLoadingCollection = false;
    let selectedCollection: IACollection;
    let collectionPage = 1;

    let selectedCollectionData: IAItem[];
    let selectedItemTitle: string;
    let selectedItem: IAItem;
    let isLoadingItem = false;

    onMount(async () => {
        // Fetch
        isLoadingCollections = true;
        collections = await getIACollections(collectionsPage);
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
                collections.push(...newPage);
                collections = collections;
                isLoadingCollections = false;
            }
            console.log("scrolled to bottom", scrolledToBottom);
        }
    }

    async function selectCollection(collection: IACollection) {
        isLoadingCollection = true;
        selectedCollection = collection;
        selectedCollectionData = await getIACollection(
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
                    selectedCollection.id,
                    ++collectionPage
                );
                selectedCollectionData.push(...newPage);
                selectedCollectionData = selectedCollectionData;
                isLoadingCollection = false;
            }
            console.log("scrolled to bottom", scrolledToBottom);
        }
    }

    async function selectItem(item: IAItem) {
        selectedItem = item;
        isLoadingItem = true;
        selectedItem = await getIAItem(item.id);
        isLoadingItem = false;
    }
</script>

<div class="container">
    <div class="grid-container" bind:this={container}>
        <header>
            <h2>Internet Archive</h2>
            <div>
                <p>Explore and download free, public domain music</p>
                <small
                    >from <a href="https://archive.org/details/audio_music"
                        >archive.org</a
                    ></small
                >
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
                    class:show={collections?.length}
                    style="grid-template-columns: repeat(auto-fit, minmax(100%, 0.1fr));width: 100%;"
                >
                    {#if collections}
                        {#each collections as collection}
                            <div
                                role="listitem"
                                tabindex="0"
                                class="item"
                                class:selected={collection?.id ===
                                    selectedCollection?.id}
                                on:click={() => {
                                    selectCollection(collection);
                                }}
                            >
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
                    {#if selectedCollection}
                        <div class="top-row">
                            <img
                                src="https://archive.org/services/img/{selectedCollection.id}"
                            />
                            <h2>{selectedCollection.title}</h2>
                        </div>
                        <p class="description">
                            {selectedCollection.description}
                        </p>
                    {:else}
                        <p>Select a collection on the left</p>
                    {/if}
                </div>
                <div class="selected-collection">
                    <div class="list">
                        {#if selectedCollectionData}
                            <ul>
                                {#each selectedCollectionData as item}
                                    <li
                                        class:selected={item.id ===
                                            selectedItem?.id}
                                        class:playing={item.id ===
                                            $currentItem?.id && $isPlaying}
                                        on:click={() => {
                                            selectItem(item);
                                        }}
                                    >
                                        <p>{item.title}</p>
                                        {#if item.id === $currentItem?.id && $isPlaying}
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

            <!-- COLUMN 2-->
            <div class="column-audio">
                {#if isLoadingItem}
                    <div class="header">
                        <h2>{selectedItem.title}</h2>
                        <p>Loading</p>
                        <div class="loading">
                            <LoadingSpinner />
                        </div>
                    </div>
                {:else if selectedItem}
                    <div class="header">
                        <h2>{selectedItem.title}</h2>
                        <div class="player">
                            <IaItemPlayer item={selectedItem} />
                        </div>
                    </div>
                    <h3>Files</h3>
                    <div class="files">
                        {#if selectedItem.files}
                            {#each selectedItem.files as file}
                                <div class="file">
                                    <div class="left">
                                        <Icon
                                            icon="bi:file-earmark-play"
                                            color="#ded2de"
                                        />
                                        <div class="info">
                                            <p class="title">
                                                {file.title ?? file.name}
                                            </p>
                                            <p class="format">{file.format}</p>
                                        </div>
                                    </div>
                                    <p class="size">
                                        {(
                                            Number(file.size) /
                                            (1024 * 1000)
                                        ).toFixed(2)} MB
                                    </p>
                                </div>
                            {/each}
                        {/if}
                    </div>

                    <div class="credits">
                        {#if selectedItem.date}
                            <p class="date">
                                Publication date: <span
                                    >{selectedItem.date}</span
                                >
                            </p>
                        {/if}
                        {#if selectedItem.performer}
                            <p>Performed by {selectedItem.performer}</p>
                        {/if}
                        {#if selectedItem.writer}
                            <p>Written by {selectedItem.writer}</p>
                        {/if}
                    </div>
                {:else}
                    <p class="hint">Select an item on the left</p>
                {/if}
            </div>
        </div>
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
        border-top: 0.7px solid #ffffff19;
        border-bottom: 0.7px solid #ffffff2a;
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
        border-bottom-left-radius: 5px;
        border-bottom-right-radius: 5px;
        border-left: 0.7px solid #ffffff2a;
        border-bottom: 0.7px solid #ffffff2a;
        background-color: #242026b3;
        header {
            display: grid;
            grid-template-columns: auto 1fr;
            padding: 0.75em 2em;
            align-items: center;
            h2 {
                margin: 0;
                font-family: monospace;
                letter-spacing: 1px;
            }
            div {
                display: flex;
                flex-direction: column;
                align-items: flex-end;
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
            border-top: 0.7px solid #ffffff2a;
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
                border-top: 0.7px solid #ffffff2a;

                &.selected {
                    background-color: #464148b3;
                }

                &:hover {
                    cursor: pointer;
                    background-color: #3d383fb3;
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
                    color: #ece8eeb3;
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
                background-color: #242026b3;
                backdrop-filter: blur(8px);
                flex-wrap: wrap;
                border-bottom: 0.7px solid #ffffff2a;

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

                    &:hover {
                        background-color: #423e44b3;
                        cursor: pointer;
                    }

                    &:active {
                        background-color: #565159b3;
                        cursor: pointer;
                    }

                    &.selected {
                        background-color: #565159b3;
                        &.playing {
                            background-color: #5123dd;
                            color: #00ddff;
                        }
                    }

                    &.playing {
                        background-color: #5123dd;
                        color: #00ddff;
                    }
                    p {
                        margin: 0;
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
                background-color: #242026b3;
                padding: 0.5em;
                border-bottom: 0.7px solid #ffffff2a;
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
                color: grey;
                span {
                    color: white;
                }
            }

            .files {
                display: grid;
                width: 100%;
                grid-template-columns: repeat(auto-fit, minmax(100%, 0.5fr));

                .file {
                    display: flex;
                    border-radius: 5px;
                    background-color: #242026b3;
                    padding: 10px;
                    margin: 0.25em;
                    gap: 10px;
                    align-items: center;
                    justify-content: space-between;
                    border: 0.7px solid #ffffff2a;

                    .left {
                        display: flex;
                        align-items: center;
                        gap: 10px;

                        .info {
                            display: flex;
                            flex-direction: column;
                            align-items: flex-start;
                            text-align: start;
                            .title {
                            }
                            .format {
                                color: grey;
                            }
                        }
                    }

                    .size {
                        color: grey;
                        white-space: nowrap;
                    }
                    &:hover {
                        cursor: pointer;
                        background-color: #3d383fb3;
                    }
                    &:active {
                        cursor: pointer;
                        background-color: #524d54b3;
                    }

                    p {
                        margin: 0;
                        text-align: start;
                    }
                }
            }

            .credits {
                padding: 1em;
                margin-top: 1em;
                color: grey;
            }

            .hint {
                padding-bottom: 1em;
                border-bottom: 0.7px solid #ffffff2a;
            }
        }
    }
</style>
