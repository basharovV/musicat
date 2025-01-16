<script lang="ts">
    import { type } from "@tauri-apps/plugin-os";
    import { onMount } from "svelte";
    import {
        isShuffleEnabled,
        popupOpen,
        rightClickedTrack,
        rightClickedTracks,
    } from "../../data/store";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import { enrichSongCountry } from "../data/LibraryEnrichers";
    import type { Song } from "../../App";
    import { findQueueIndexes, updateQueues } from "../../data/storeHelper";
    import {
        searchArtistOnWikiPanel,
        searchArtistOnWikipedia,
        searchArtistOnYouTube,
        searchChords,
        searchLyrics,
        searchSongOnYouTube,
    } from "../menu/search";
    import { openInFinder } from "../menu/file";

    type ActionType = "country" | "delete";

    export let onUnselect: () => void;

    let confirmingType: ActionType = null;
    let loadingType: ActionType = null;
    let position = { x: 0, y: 0 };
    let showMenu = false;
    let song: Song | null = null;
    let songs: Song[] = [];

    let explorerName: string;

    onMount(async () => {
        const os = await type();
        switch (os) {
            case "macos":
                explorerName = "Finder";
                break;
            case "windows":
                explorerName = "Explorer";
                break;
            case "linux":
                explorerName = "File manager";
                break;
        }
    });

    export function close() {
        showMenu = false;
    }

    export function isOpen() {
        return showMenu;
    }

    export function open(_songs: Song[], _position: { x: number; y: number }) {
        songs = _songs;
        position = _position;

        if (songs.length === 1) {
            song = songs[0];
        } else {
            song = null;
        }

        confirmingType = null;

        showMenu = true;
    }

    function compose(action, ...args) {
        return () => {
            close();

            action(...args);
        };
    }

    $: isConfirming = (type: ActionType): boolean => confirmingType === type;
    $: isDisabled = (type?: ActionType): boolean =>
        !!loadingType && loadingType !== type;
    $: isLoading = (type: ActionType): boolean => loadingType === type;

    async function removeFromQueue() {
        console.log("delete");
        if (confirmingType !== "delete") {
            confirmingType = "delete";
            return;
        }

        close();

        const indexes = songs.map((t) => t.viewModel.index);

        if ($isShuffleEnabled) {
            const qIndexes = findQueueIndexes(songs);

            updateQueues(qIndexes, indexes, (queue, indexes) => {
                for (const index of indexes.sort((a, b) => b - a)) {
                    queue.splice(index, 1);
                }
            });
        } else {
            updateQueues(indexes, null, (queue, indexes) => {
                for (const index of indexes.sort((a, b) => b - a)) {
                    queue.splice(index, 1);
                }
            });
        }
    }

    function openInfo() {
        if (songs.length === 1) {
            $rightClickedTracks = [];
            $rightClickedTrack = song;
        } else {
            $rightClickedTracks = songs;
            $rightClickedTrack = null;
        }

        close();
        $popupOpen = "track-info";
    }
    // Enrichers

    async function fetchingOriginCountry() {
        loadingType = "country";

        await enrichSongCountry(song);

        loadingType = "country";
    }

    function unselect() {
        close();

        onUnselect && onUnselect();
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close} fixed>
        <MenuOption
            isDisabled={true}
            text={song ? song.title : songs.length + " tracks"}
        />
        {#if song}
            {#if song.artist}
                <MenuDivider />
                <MenuOption text="⚡️ Enrich" isDisabled />
                <MenuOption
                    isLoading={isLoading("country")}
                    onClick={fetchingOriginCountry}
                    text={!song.originCountry
                        ? isLoading("country")
                            ? "Looking online..."
                            : "Origin country"
                        : "Origin country ✅"}
                    description="from Wikipedia"
                />
            {/if}
            <MenuDivider />
            <MenuOption
                onClick={compose(searchSongOnYouTube, song)}
                text="YouTube: <i>{song.title}</i>"
            />
            <MenuOption
                onClick={compose(searchChords, song)}
                text="Chords: <i>{song.title}</i>"
            />
            <MenuOption
                onClick={compose(searchLyrics, song)}
                text="Lyrics: <i>{song.title}</i>"
            />
            <MenuDivider />
            {#if song.artist}
                <MenuOption
                    onClick={compose(searchArtistOnYouTube, song)}
                    text="YouTube: <i>{song.artist}</i>"
                />
                <MenuOption
                    onClick={compose(searchArtistOnWikipedia, song)}
                    text="Wikipedia: <i>{song.artist}</i>"
                />
                <MenuOption
                    onClick={compose(searchArtistOnWikiPanel, song)}
                    text="Wiki panel: <i>{song.artist}</i>"
                />
                <MenuDivider />
            {/if}
        {/if}
        {#if songs.length > 1}
            <MenuOption onClick={unselect} text="Unselect all" />
        {/if}
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirming("delete")}
            onClick={removeFromQueue}
            text={song ? "Remove track from queue" : "Remove tracks from queue"}
            confirmText="Click again to confirm"
        />

        <MenuDivider />

        {#if song}
            <MenuOption
                onClick={compose(openInFinder, song)}
                text="Open in {explorerName}"
            />
        {/if}

        <MenuOption onClick={openInfo} text="Info & metadata" />
    </Menu>
{/if}
