<script lang="ts">
    import { type } from "@tauri-apps/plugin-os";
    import { open } from "@tauri-apps/plugin-shell";
    import { onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        isShuffleEnabled,
        popupOpen,
        rightClickedTrack,
        rightClickedTracks,
    } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuDivider from "../menu/MenuDivider.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import {
        enrichArtistCountry,
        findCountryByArtist,
    } from "../data/LibraryEnrichers";
    import type { Song } from "../../App";
    import { findQueueIndexes, updateQueues } from "../../data/storeHelper";
    import { writable } from "svelte/store";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;
    export let songs: Song[] = [];

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

    let song: Song | null = null;
    let isConfirmingDelete = false;
    let confirmingLength = -1;
    let confirmingHash = "";

    $: if (
        songs.length !== confirmingLength ||
        songs[0].id !== confirmingHash
    ) {
        isConfirmingDelete = false;
    }

    $: if (songs.length === 1) {
        song = songs[0];
    } else {
        song = null;
    }

    function closeMenu() {
        showMenu = false;
        isConfirmingDelete = false;
    }

    async function removeFromQueue() {
        console.log("delete");
        if (!isConfirmingDelete) {
            confirmingLength = songs.length;
            confirmingHash = songs[0].id;
            isConfirmingDelete = true;
            return;
        }

        closeMenu();

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

        songs = [];
        isConfirmingDelete = false;
    }

    function searchArtistOnYouTube() {
        closeMenu();
        const query = encodeURIComponent(song.artist);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchSongOnYouTube() {
        closeMenu();
        const query = encodeURIComponent(song.title);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchArtistOnWikipedia() {
        closeMenu();
        const query = encodeURIComponent(song.artist);
        open(`https://en.wikipedia.org/wiki/${query}`);
    }
    function openInFinder() {
        closeMenu();
        const query = song.path.replace(song.file, "");
        open(query);
    }
    function lookUpChords() {
        closeMenu();
        const query = encodeURIComponent(
            song.artist + " " + song.title + " chords",
        );
        open(`https://duckduckgo.com/?q=${query}`);
    }
    function lookUpLyrics() {
        closeMenu();
        const query = encodeURIComponent(
            song.artist + " " + song.title + " lyrics",
        );
        open(`https://duckduckgo.com/?q=${query}`);
    }
    function openInfo() {
        if (songs.length === 1) {
            $rightClickedTracks = [];
            $rightClickedTrack = song;
        } else {
            $rightClickedTracks = songs;
            $rightClickedTrack = null;
        }

        closeMenu();
        $popupOpen = "track-info";
    }
    // Enrichers

    let isFetchingOriginCountry = writable(false);

    async function fetchingOriginCountry() {
        await enrichArtistCountry(song, isFetchingOriginCountry);
    }

    function unselect() {
        closeMenu();
        songs.length = 0;
    }
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu} fixed>
        <MenuOption
            isDisabled={true}
            text={song ? song.title : songs.length + " tracks"}
        />
        {#if song}
            {#if song.artist}
                <MenuDivider />
                <MenuOption text="⚡️ Enrich" isDisabled />
                <MenuOption
                    onClick={fetchingOriginCountry}
                    text={!song.originCountry
                        ? $isFetchingOriginCountry
                            ? "Looking online..."
                            : "Origin country"
                        : "Origin country ✅"}
                    description="from Wikipedia"
                />
            {/if}
            <MenuDivider />
            <MenuOption
                onClick={searchSongOnYouTube}
                text="YouTube: <i>{song.title}</i>"
            />
            <MenuOption
                onClick={lookUpChords}
                text="Chords: <i>{song.title}</i>"
            />
            <MenuOption
                onClick={lookUpLyrics}
                text="Lyrics: <i>{song.title}</i>"
            />
            <MenuDivider />
            {#if song.artist}
                <MenuOption
                    onClick={searchArtistOnYouTube}
                    text="YouTube: <i>{song.artist}</i>"
                />
                <MenuOption
                    onClick={searchArtistOnWikipedia}
                    text="Wikipedia: <i>{song.artist}</i>"
                />
                <MenuOption
                    onClick={searchArtistOnWikipedia}
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
            isConfirming={isConfirmingDelete}
            onClick={removeFromQueue}
            text={song ? "Remove track from queue" : "Remove tracks from queue"}
            confirmText="Click again to confirm"
        />

        <MenuDivider />

        {#if song}
            <MenuOption onClick={openInFinder} text="Open in {explorerName}" />
        {/if}

        <MenuOption onClick={openInfo} text="Info & metadata" />
    </Menu>
{/if}
