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
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import type { Song } from "../../App";
    import { findQueueIndexes, updateQueues } from "../../data/storeHelper";

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

    let isFetchingOriginCountry = false;

    async function enrichArtistCountry() {
        isFetchingOriginCountry = true;
        const country = await findCountryByArtist(song.artist);
        console.log("country", country);
        if (country) {
            song.originCountry = country;

            // Find all songs with this artist
            const artistSongs = await db.songs
                .where("artist")
                .equals(song.artist)
                .toArray();
            artistSongs.forEach((s) => {
                s.originCountry = country;
                db.songs.update(s.id, s);
            });
        }
        isFetchingOriginCountry = false;
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
        {#if song}
            <MenuDivider />
            <MenuOption text="Enrich" isDisabled />
            <MenuOption
                onClick={enrichArtistCountry}
                text={!song.originCountry
                    ? isFetchingOriginCountry
                        ? "Looking online..."
                        : "Origin country"
                    : "Origin country ✅"}
                description="from Wikipedia"
            />
            <MenuDivider />
            <MenuOption
                onClick={searchSongOnYouTube}
                text="Search for song on YouTube"
            />
            <MenuOption
                onClick={searchArtistOnYouTube}
                text="Search for artist on YouTube"
            />
            <MenuOption
                onClick={searchArtistOnWikipedia}
                text="Search for artist on Wikipedia"
            />
            <MenuDivider />
            <MenuOption onClick={lookUpChords} text="Look up chords" />
            <MenuOption onClick={lookUpLyrics} text="Look up lyrics" />
            <MenuDivider />

            <MenuOption onClick={openInFinder} text="Open in {explorerName}" />
        {/if}

        <MenuOption onClick={openInfo} text="Info & metadata" />
    </Menu>
{/if}
