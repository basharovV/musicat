<script lang="ts">
    import {
        queriedSongs,
        queue,
        smartQueryResults,
        uiView,
    } from "../../data/store";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import { setQueue } from "../../data/storeHelper";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuInput from "../ui/menu/MenuInput.svelte";
    import { createNewPlaylistFile } from "../../data/M3UUtils";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;

    let playlistInput = "";

    function clearQueue() {
        setQueue([]);

        closeMenu();
    }

    function closeMenu() {
        showMenu = false;
    }

    function resetToLibrary() {
        if ($uiView === "smart-query") {
            setQueue($smartQueryResults, 0);
        } else {
            setQueue($queriedSongs, 0);
        }

        closeMenu();
    }

    function saveAsPlaylist() {
        createNewPlaylistFile(playlistInput, $queue);

        playlistInput = "";

        closeMenu();
    }
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu} fixed>
        <MenuOption onClick={clearQueue} text="Clear the queue" />
        <MenuOption
            onClick={resetToLibrary}
            text="Reset queue to library order"
        />
        <MenuDivider />
        <MenuOption isDisabled={true} text="Save as playlist" />
        <MenuInput
            bind:value={playlistInput}
            onEnterPressed={saveAsPlaylist}
            placeholder="New playlist"
            onEscPressed={closeMenu}
            small
        />
    </Menu>
{/if}
