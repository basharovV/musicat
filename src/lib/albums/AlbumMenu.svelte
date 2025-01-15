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
    import {
        type EnricherResult,
        enrichArtistCountry,
        fetchAlbumArt,
        rescanAlbumArtwork,
    } from "../data/LibraryEnrichers";
    import type { Album, ToImport } from "../../App";
    import { invoke } from "@tauri-apps/api/core";
    import { writable } from "svelte/store";

    export let position = { x: 0, y: 0 };
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
        const query = encodeURIComponent(
            `${$rightClickedAlbum.artist} - ${$rightClickedAlbum.title}`,
        );
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
    let isFetchingOriginCountry = writable(false);

    async function fetchingOriginCountry() {
        await enrichArtistCountry(
            $rightClickedTracks[0],
            isFetchingOriginCountry,
        );
    }

    let fetchArtworkLoading = false;
    let fetchArtworkResult: EnricherResult;
    let fetchArtworkAlbumId: String;

    async function fetchArtwork() {
        fetchArtworkLoading = true;
        fetchArtworkResult = null;
        fetchArtworkAlbumId = $rightClickedAlbum.id;

        fetchArtworkResult = await fetchAlbumArt($rightClickedAlbum);

        fetchArtworkLoading = false;
    }

    let rescanLocalArtworkLoading = false;
    let rescanLocalArtworkResult: EnricherResult;
    let rescanLocalArtworkAlbumId: String;

    async function rescanLocalArtwork() {
        rescanLocalArtworkLoading = true;
        rescanLocalArtworkResult = null;
        rescanLocalArtworkAlbumId = $rightClickedAlbum.id;

        rescanLocalArtworkResult = await rescanAlbumArtwork($rightClickedAlbum);

        rescanLocalArtworkLoading = false;
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
    <Menu {...position} onClickOutside={closeMenu}>
        <MenuOption
            isDisabled={true}
            text="{$rightClickedAlbum.displayTitle ??
                $rightClickedAlbum.title} by {$rightClickedAlbum.artist}"
        />
        <MenuOption
            onClick={reImportAlbum}
            text="Re-import album"
            isLoading={isReimporting}
        />
        <MenuDivider />
        <MenuOption text="⚡️ Enrich" isDisabled />
        {#if $rightClickedTracks[0].artist}
            <MenuOption
                onClick={fetchingOriginCountry}
                text={!$rightClickedTracks[0].originCountry
                    ? $isFetchingOriginCountry
                        ? "Looking online..."
                        : "Origin country"
                    : "Origin country ✅"}
                description="from Wikipedia"
            />
        {/if}
        <MenuOption
            onClick={fetchArtwork}
            isLoading={fetchArtworkLoading}
            text="Fetch artwork"
            description={fetchArtworkLoading
                ? "Fetching from Wikipedia..."
                : "Save to folder as cover.jpg"}
        />
        {#if fetchArtworkResult && fetchArtworkAlbumId === $rightClickedAlbum.id}
            <MenuOption
                text={fetchArtworkResult.error || fetchArtworkResult.success}
                isDisabled
            />
        {/if}
        <MenuOption
            onClick={rescanLocalArtwork}
            isLoading={rescanLocalArtworkLoading}
            text="Scan existing artwork"
            description={rescanLocalArtworkLoading
                ? "Rescanning..."
                : "Check encoded art in tracks / folder image"}
        />
        {#if rescanLocalArtworkResult && rescanLocalArtworkAlbumId === $rightClickedAlbum.id}
            <MenuOption
                text={rescanLocalArtworkResult.error ||
                    rescanLocalArtworkResult.success}
                isDisabled
            />
        {/if}
        <MenuOption
            onClick={searchArtworkOnBrave}
            text="Search for artwork on Brave"
        />
        {#if $rightClickedTracks[0].artist}
            <MenuDivider />
            <MenuOption
                onClick={searchArtistOnYouTube}
                text="YouTube: <i>{$rightClickedTracks[0].artist}</i>"
            />
            <MenuOption
                onClick={searchArtistOnWikipedia}
                text="Wikipedia: <i>{$rightClickedTracks[0].artist}</i>"
            />
            <MenuOption
                onClick={searchArtistOnWikipedia}
                text="Wiki panel: <i>{$rightClickedTracks[0].artist}</i>"
            />
        {/if}
        <MenuDivider />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={deleteAlbum}
            text="Remove album from library"
            confirmText="Click again to confirm"
        />
        <MenuDivider />
        <MenuOption onClick={openInFinder} text="Open in {explorerName}" />
        <MenuOption onClick={openInfo} text="Info & metadata" />
    </Menu>
{/if}
