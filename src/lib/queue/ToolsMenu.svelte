<script lang="ts">
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import { enrichSongCountry } from "../data/LibraryEnrichers";
    import type { Song } from "../../App";
    import {
        searchArtistOnWikiPanel,
        searchArtistOnYouTube,
        searchChords,
        searchLyrics,
        searchSongOnYouTube,
    } from "../menu/search";

    type ActionType = "country";

    let confirmingType: ActionType = null;
    let loadingType: ActionType = null;
    let position = { x: 0, y: 0 };
    let showMenu = false;

    let song: Song;

    export function close() {
        showMenu = false;
    }

    export function isOpen() {
        return showMenu;
    }

    export function open(_song: Song, _position: { x: number; y: number }) {
        song = _song;
        position = _position;

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

    async function fetchingOriginCountry() {
        loadingType = "country";

        await enrichSongCountry(song);

        loadingType = "country";
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close} fixed submenu>
        {#if song.artist}
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
            <MenuDivider />
        {/if}
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
    </Menu>
{/if}
