<script lang="ts">
    import { type } from "@tauri-apps/api/os";
    import { open } from "@tauri-apps/api/shell";
    import { onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        isTrackInfoPopupOpen,
        rightClickedTrack,
        rightClickedTracks,
        selectedPlaylistId
    } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuDivider from "../menu/MenuDivider.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { findCountryByArtist } from "../data/LibraryEnrichers";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;
    let explorerName: string;

    onMount(async () => {
        const os = await type();
        switch (os) {
            case "Darwin":
                explorerName = "Finder";
                break;
            case "Windows_NT":
                explorerName = "Explorer";
                break;
            case "Linux":
                explorerName = "File manager";
                break;
        }
    });

    let songId;
    let isConfirmingDelete = false;

    $: {
        if (
            $rightClickedTracks?.map((s) => s.id).includes(songId) ||
            songId !== $rightClickedTrack?.id
        ) {
            isConfirmingDelete = false;
        }
    }

    function closeMenu() {
        showMenu = false;
        isConfirmingDelete = false;
    }

    function deleteTrack() {
        console.log("delete");
        if (!isConfirmingDelete) {
            songId = $rightClickedTrack?.id;
            isConfirmingDelete = true;
            return;
        }
        if ($rightClickedTracks.length) {
            closeMenu();
            $rightClickedTracks.forEach((t) => {
                db.songs.delete(t.id);
            });
            $rightClickedTracks = [];
            isConfirmingDelete = false;
        } else if ($rightClickedTrack) {
            closeMenu();
            db.songs.delete($rightClickedTrack.id);
            $rightClickedTrack = null;
            isConfirmingDelete = false;
        }
    }

    async function removeFromPlaylist() {
        const playlist = await db.playlists.get($selectedPlaylistId);
        if ($rightClickedTracks.length) {
            closeMenu();
            $rightClickedTracks.forEach((t) => {
                const trackIdx = playlist.tracks.findIndex((pt) => pt === t.id);
                playlist.tracks.splice(trackIdx, 1);
            });
            $rightClickedTracks = [];
            isConfirmingDelete = false;
        } else if ($rightClickedTrack) {
            closeMenu();
            const trackIdx = playlist.tracks.findIndex(
                (pt) => pt === $rightClickedTrack.id
            );
            playlist.tracks.splice(trackIdx, 1);
            $rightClickedTrack = null;
            isConfirmingDelete = false;
        }
        await db.playlists.put(playlist);
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
        $isTrackInfoPopupOpen = true;
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
    <Menu {...pos} onClickOutside={closeMenu}>
        <MenuOption
            isDisabled={true}
            text={$rightClickedTrack
                ? $rightClickedTrack.title
                : $rightClickedTracks.length + " tracks"}
        />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={deleteTrack}
            text={$rightClickedTrack ? "Delete track" : "Delete tracks"}
            confirmText="Click again to confirm"
        />
        {#if $selectedPlaylistId}
            <MenuOption
                isDestructive={true}
                isConfirming={isConfirmingDelete}
                onClick={removeFromPlaylist}
                text={$rightClickedTrack
                    ? "Remove from playlist"
                    : `Remove  ${$rightClickedTracks.length} tracks from playlist`}
                confirmText="Click again to confirm"
            />
        {/if}
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
