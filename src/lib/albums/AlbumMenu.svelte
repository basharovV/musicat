<script lang="ts">
    import { type } from "@tauri-apps/plugin-os";
    import { open } from "@tauri-apps/plugin-shell";
    import { onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        popupOpen,
        rightClickedAlbum,
        rightClickedTracks,
    } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuDivider from "../menu/MenuDivider.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { fetchAlbumArt } from "../data/LibraryEnrichers";
    import { rescanAlbumArtwork } from "../../data/LibraryImporter";
    import type { Album, ToImport } from "../../App";
    import { invoke } from "@tauri-apps/api/core";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;
    export let onClose;

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

    let albumId;
    let isConfirmingDelete = false;

    $: {
        if (albumId !== $rightClickedAlbum?.id) {
            isConfirmingDelete = false;
        }
    }

    function closeMenu() {
        showMenu = false;
        isConfirmingDelete = false;
        artworkResult = null;
        onClose && onClose();
    }

    async function deleteAlbum() {
        console.log("delete");
        if (!isConfirmingDelete) {
            albumId = $rightClickedAlbum?.id;
            isConfirmingDelete = true;
            return;
        }
        if ($rightClickedAlbum) {
            closeMenu();
            await db.songs.bulkDelete($rightClickedAlbum.tracksIds);
            await db.albums.delete($rightClickedAlbum.id);
            $rightClickedAlbum = null;
            isConfirmingDelete = false;
        }
    }

    function searchArtistOnYouTube() {
        closeMenu();
        const query = encodeURIComponent($rightClickedAlbum.artist);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchSongOnYouTube() {
        closeMenu();
        const query = encodeURIComponent(
            $rightClickedAlbum.displayTitle ?? $rightClickedAlbum.title,
        );
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchArtistOnWikipedia() {
        closeMenu();
        const query = encodeURIComponent($rightClickedAlbum.artist);
        open(`https://en.wikipedia.org/wiki/${query}`);
    }
    function searchArtworkOnBrave() {
        closeMenu();
        const query = encodeURIComponent(`${$rightClickedAlbum.artist} - ${$rightClickedAlbum.title}`);
        open(`https://search.brave.com/images?q=${query}`);
    }
    function openInFinder() {
        closeMenu();
        open($rightClickedAlbum.path);
    }
    function openInfo() {
        closeMenu();
        $popupOpen = "track-info";
    }

    // Enrichers
    let isFetchingArtwork = false;
    let artworkResult: { success?: string; error?: string };
    let artworkResultForAlbum: String;
    async function fetchArtwork() {
        artworkResult = null;
        isFetchingArtwork = true;
        artworkResultForAlbum = $rightClickedAlbum.id;
        artworkResult = await fetchAlbumArt($rightClickedAlbum);
        isFetchingArtwork = false;
    }

    async function rescanLocalArtwork() {
        rescanAlbumArtwork($rightClickedAlbum);
    }

    let isReimporting = false;

    async function reImportAlbum() {
        isReimporting = true;
        const response = await invoke<ToImport>("scan_paths", {
            event: {
                paths: [$rightClickedAlbum.path],
                recursive: false,
                process_albums: true,
                is_async: false,
            },
        });
        console.log("response", response);
        await db.transaction("rw", db.songs, db.albums, async () => {
            await db.songs.bulkPut(response.songs);
            await db.albums.bulkPut(response.albums);
        });
        isReimporting = false;
    }
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu}>
        <MenuOption
            isDisabled={true}
            text="{$rightClickedAlbum.displayTitle ??
                $rightClickedAlbum.title} by {$rightClickedAlbum.artist}"
        />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={deleteAlbum}
            text="Remove album from library"
            confirmText="Click again to confirm"
        />
        {#if $rightClickedAlbum}
            <MenuOption
                onClick={reImportAlbum}
                text="Re-import album"
                isLoading={isReimporting}
            />
            <MenuDivider />
            <MenuOption text="⚡️ Enrich" isDisabled />
            <MenuOption
                onClick={fetchArtwork}
                isLoading={isFetchingArtwork}
                text="Fetch artwork"
                description={isFetchingArtwork
                    ? "Fetching from Wikipedia..."
                    : "Save to folder as cover.jpg"}
            />
            {#if artworkResult && artworkResultForAlbum === $rightClickedAlbum.id}
                <MenuOption
                    text={artworkResult.error || artworkResult.success}
                    isDisabled
                />
            {/if}
            <MenuOption
                onClick={rescanLocalArtwork}
                text="Scan existing artwork"
                description="Check encoded art in tracks / folder image"
            />
            <MenuOption
                onClick={searchArtworkOnBrave}
                text="Search for artwork on Brave"
            />
            <MenuDivider />
            <MenuOption
                onClick={searchArtistOnYouTube}
                text="Search for artist on YouTube"
            />
            <MenuOption
                onClick={searchArtistOnWikipedia}
                text="Search for artist on Wikipedia"
            />
            <MenuDivider />

            <MenuOption onClick={openInFinder} text="Open in {explorerName}" />
            <MenuOption onClick={openInfo} text="Info & metadata" />
        {/if}
    </Menu>
{/if}
