<script lang="ts">
    import { type } from "@tauri-apps/plugin-os";
    import { open } from "@tauri-apps/plugin-shell";
    import { onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        isShuffleEnabled,
        popupOpen,
        playlist,
        rightClickedTrack,
        rightClickedTracks,
        selectedPlaylistFile,
        shuffledPlaylist
    } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuDivider from "../menu/MenuDivider.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import type { Song } from "../../App";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;
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
            case "Linux":
                explorerName = "File manager";
                break;
        }
    });

    let songId;
    let isConfirmingDelete = false;
    let isConfirmingRemoveFromPlaylist = false;

    $: {
        if (
            $rightClickedTracks?.map((s) => s.id).includes(songId) ||
            songId !== $rightClickedTrack?.id
        ) {
            isConfirmingDelete = false;
            isConfirmingRemoveFromPlaylist = false;
        }
    }

    function closeMenu() {
        showMenu = false;
        isConfirmingDelete = false;
        isConfirmingRemoveFromPlaylist = false;
    }

    function spliceMultiple(arr: Song[], indexes: number[]) {
        // Sort the indexes in descending order to avoid index shifting issues
        indexes.sort((a, b) => b - a);
        // Create a copy of the array to avoid mutating the original array
        let newArray = arr.slice();
        // Iterate over the sorted indexes and remove elements
        indexes.forEach((index) => {
            if (index >= 0 && index < newArray.length) {
                newArray.splice(index, 1);
            }
        });
        return newArray;
    }

    async function removeFromQueue() {
        console.log("delete");
        if (!isConfirmingDelete) {
            songId = $rightClickedTrack?.id;
            isConfirmingDelete = true;
            return;
        }

        if ($rightClickedTracks.length) {
            closeMenu();
            const spliced = spliceMultiple(
                $isShuffleEnabled ? $shuffledPlaylist : $playlist,
                $rightClickedTracks.map((t) => t.viewModel.index)
            );
            console.log("spliced", spliced);
            if ($isShuffleEnabled) {
                $shuffledPlaylist = spliced;
            } else {
                $playlist = spliced;
            }
            $rightClickedTracks = [];
            isConfirmingDelete = false;
        } else if ($rightClickedTrack) {
            closeMenu();

            ($isShuffleEnabled ? $shuffledPlaylist : $playlist).splice(
                $rightClickedTrack.viewModel.index,
                1
            );
            $playlist = $playlist;
            $rightClickedTrack = null;
            isConfirmingDelete = false;
        }
    }

    function searchArtistOnYouTube() {
        closeMenu();
        const query = encodeURIComponent($rightClickedTrack.artist);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchSongOnYouTube() {
        closeMenu();
        const query = encodeURIComponent($rightClickedTrack.title);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchArtistOnWikipedia() {
        closeMenu();
        const query = encodeURIComponent($rightClickedTrack.artist);
        open(`https://en.wikipedia.org/wiki/${query}`);
    }
    function openInFinder() {
        closeMenu();
        const query = $rightClickedTrack.path.replace(
            $rightClickedTrack.file,
            ""
        );
        open(query);
    }
    function lookUpChords() {
        closeMenu();
        const query = encodeURIComponent(
            $rightClickedTrack.artist +
                " " +
                $rightClickedTrack.title +
                " chords"
        );
        open(`https://duckduckgo.com/?q=${query}`);
    }
    function lookUpLyrics() {
        closeMenu();
        const query = encodeURIComponent(
            $rightClickedTrack.artist +
                " " +
                $rightClickedTrack.title +
                " lyrics"
        );
        open(`https://duckduckgo.com/?q=${query}`);
    }
    function openInfo() {
        closeMenu();
        $popupOpen = 'track-info';
    }
    // Enrichers

    let isFetchingOriginCountry = false;

    async function enrichArtistCountry() {
        isFetchingOriginCountry = true;
        const country = await findCountryByArtist($rightClickedTrack.artist);
        console.log("country", country);
        if (country) {
            $rightClickedTrack.originCountry = country;

            // Find all songs with this artist
            const artistSongs = await db.songs
                .where("artist")
                .equals($rightClickedTrack.artist)
                .toArray();
            artistSongs.forEach((s) => {
                s.originCountry = country;
                db.songs.update(s.id, s);
            });
        }
        isFetchingOriginCountry = false;
    }
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu} fixed>
        <MenuOption
            isDisabled={true}
            text={$rightClickedTrack
                ? $rightClickedTrack.title
                : $rightClickedTracks.length + " tracks"}
        />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={() => {
                removeFromQueue();
            }}
            text={$rightClickedTrack
                ? "Remove track from queue"
                : "Remove tracks from queue"}
            confirmText="Click again to confirm"
        />
        {#if $rightClickedTrack}
            <MenuDivider />
            <MenuOption text="Enrich" isDisabled />
            <MenuOption
                onClick={enrichArtistCountry}
                text={!$rightClickedTrack.originCountry
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
