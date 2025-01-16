<script lang="ts">
    import { type } from "@tauri-apps/plugin-os";
    import { onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        popupOpen,
        rightClickedTrack,
        rightClickedTracks,
    } from "../../data/store";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import {
        type EnricherResult,
        enrichSongCountry,
        fetchAlbumArt,
        rescanAlbumArtwork,
    } from "../data/LibraryEnrichers";
    import type { Album, Song, ToImport } from "../../App";
    import { invoke } from "@tauri-apps/api/core";
    import { openInFinder } from "../menu/file";
    import {
        searchArtistOnWikiPanel,
        searchArtistOnYouTube,
        searchArtworkOnBrave,
    } from "../menu/search";
    import { removeQueuedSongs } from "../../data/storeHelper";

    type ActionType =
        | "artwork-local"
        | "artwork-online"
        | "country"
        | "delete"
        | "re-import";

    export let onClose: () => void;

    let album: Album;
    let confirmingType: ActionType = null;
    let explorerName: string;
    let loadingType: ActionType = null;
    let position = { x: 0, y: 0 };
    let result: EnricherResult;
    let resultType: ActionType;
    let showMenu = false;
    let song: Song;
    let songs: Song[];

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

    export function open(
        _album: Album,
        _songs: Song[],
        _position: { x: number; y: number },
    ) {
        album = _album;
        song = _songs[0];
        songs = _songs;
        position = _position;

        confirmingType = null;
        resultType = null;
        result = null;

        showMenu = true;
    }

    function close() {
        showMenu = false;

        onClose && onClose();
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
    $: hasResult = (type: ActionType) => result && resultType === type;

    async function deleteAlbum() {
        console.log("delete");
        if (confirmingType !== "delete") {
            confirmingType = "delete";
            return;
        }

        if (album) {
            loadingType = "delete";

            await db.songs.bulkDelete(album.tracksIds);
            await db.albums.delete(album.id);
            await removeQueuedSongs(album.tracksIds);

            loadingType = null;

            close();
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

        await enrichSongCountry(songs[0]);

        loadingType = null;
    }

    async function fetchArtwork() {
        loadingType = "artwork-online";
        resultType = "artwork-online";
        result = null;

        result = await fetchAlbumArt(album);

        loadingType = null;
    }

    async function rescanLocalArtwork() {
        loadingType = "artwork-local";
        resultType = "artwork-local";
        result = null;

        result = await rescanAlbumArtwork(album);

        loadingType = null;
    }

    async function reImportAlbum() {
        loadingType = "re-import";

        const response = await invoke<ToImport>("scan_paths", {
            event: {
                paths: [album.path],
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

        loadingType = null;
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close}>
        <MenuOption
            isDisabled={true}
            text="{album.displayTitle ?? album.title} by {album.artist}"
        />
        <MenuOption
            isLoading={isLoading("re-import")}
            onClick={reImportAlbum}
            text="Re-import album"
        />
        <MenuDivider />
        <MenuOption text="⚡️ Enrich" isDisabled />
        {#if song.artist}
            <MenuOption
                isLoading={isLoading("country")}
                onClick={fetchingOriginCountry}
                text={!songs[0].originCountry
                    ? isLoading("country")
                        ? "Looking online..."
                        : "Origin country"
                    : "Origin country ✅"}
                description="from Wikipedia"
            />
        {/if}
        <MenuOption
            isLoading={isLoading("artwork-online")}
            onClick={fetchArtwork}
            text="Fetch artwork"
            description={isLoading("artwork-online")
                ? "Fetching from Wikipedia..."
                : "Save to folder as cover.jpg"}
        />
        {#if hasResult("artwork-online")}
            <MenuOption text={result.error || result.success} isDisabled />
        {/if}
        <MenuOption
            isLoading={isLoading("artwork-local")}
            onClick={rescanLocalArtwork}
            text="Scan existing artwork"
            description={isLoading("artwork-local")
                ? "Rescanning..."
                : "Check encoded art in tracks / folder image"}
        />
        {#if hasResult("artwork-local")}
            <MenuOption text={result.error || result.success} isDisabled />
        {/if}
        <MenuOption
            onClick={compose(searchArtworkOnBrave, song)}
            text="Search for artwork on Brave"
        />
        {#if song.artist}
            <MenuDivider />
            <MenuOption
                onClick={compose(searchArtistOnYouTube, song)}
                text="YouTube: <i>{song.artist}</i>"
            />
            <MenuOption
                onClick={compose(searchArtistOnWikiPanel, song)}
                text="Wiki panel: <i>{song.artist}</i>"
            />
        {/if}
        <MenuDivider />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirming("delete")}
            onClick={deleteAlbum}
            text="Remove album from library"
            confirmText="Click again to confirm"
        />
        <MenuDivider />
        <MenuOption
            onClick={compose(openInFinder, song)}
            text="Open in {explorerName}"
        />
        <MenuOption onClick={openInfo} text="Info & metadata" />
    </Menu>
{/if}
