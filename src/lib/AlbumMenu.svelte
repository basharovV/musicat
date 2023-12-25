<script lang="ts">
    import { type } from "@tauri-apps/api/os";
    import { open } from "@tauri-apps/api/shell";
    import { onMount } from "svelte";
    import { db } from "../data/db";
    import { isTrackInfoPopupOpen, rightClickedAlbum } from "../data/store";
    import Menu from "./menu/Menu.svelte";
    import MenuDivider from "./menu/MenuDivider.svelte";
    import MenuOption from "./menu/MenuOption.svelte";
    import { fetchAlbumArt } from "./data/LibraryEnrichers";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;
    export let onClose;

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

    function deleteAlbum() {
        console.log("delete");
        if (!isConfirmingDelete) {
            albumId = $rightClickedAlbum?.id;
            isConfirmingDelete = true;
            return;
        }
        if ($rightClickedAlbum) {
            closeMenu();
            db.songs.delete($rightClickedAlbum.id);
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
        const query = encodeURIComponent($rightClickedAlbum.title);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchArtistOnWikipedia() {
        closeMenu();
        const query = encodeURIComponent($rightClickedAlbum.artist);
        open(`https://en.wikipedia.org/wiki/${query}`);
    }
    function openInFinder() {
        closeMenu();
        open($rightClickedAlbum.path);
    }
    function openInfo() {
        closeMenu();
        $isTrackInfoPopupOpen = true;
    }

    // Enrichers
    let isFetchingArtwork = false;
    let artworkResult: { success?: string; error?: string };
    async function fetchArtwork() {
        isFetchingArtwork = true;
        artworkResult = await fetchAlbumArt($rightClickedAlbum);
        isFetchingArtwork = false;
    }
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu}>
        <MenuOption
            isDisabled={true}
            text="{$rightClickedAlbum.title} by {$rightClickedAlbum.artist}"
        />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={deleteAlbum}
            text="Delete album"
            confirmText="Click again to confirm"
        />
        {#if $rightClickedAlbum}
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
            {#if artworkResult}
                <MenuOption
                    text={artworkResult.error || artworkResult.success}
                    isDisabled
                />
            {/if}
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
