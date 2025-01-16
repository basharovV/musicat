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

    let playlistInput = "";
    let position = { x: 0, y: 0 };
    let showMenu = false;

    export function close() {
        showMenu = false;
    }

    export function open(_position: { x: number; y: number }) {
        position = _position;

        showMenu = true;
    }

    function clearQueue() {
        setQueue([]);

        close();
    }

    function resetToLibrary() {
        if ($uiView === "smart-query") {
            setQueue($smartQueryResults, 0);
        } else {
            setQueue($queriedSongs, 0);
        }

        close();
    }

    function saveAsPlaylist() {
        createNewPlaylistFile(playlistInput, $queue);

        playlistInput = "";

        close();
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close} fixed>
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
            onEscPressed={close}
            small
        />
    </Menu>
{/if}
