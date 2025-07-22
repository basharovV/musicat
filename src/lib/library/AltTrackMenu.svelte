<script lang="ts">
    import type { Song } from "../../App";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import { appendToQueue } from "../../data/storeHelper";

    let position = { x: 0, y: 0 };
    let showMenu = false;
    let song: Song;
    let songs: Song[];

    export function close() {
        showMenu = false;
    }

    export function isOpen() {
        return showMenu;
    }

    export function open(
        _songs: Song | Song[],
        _position: { x: number; y: number },
    ) {
        position = _position;

        if (Array.isArray(_songs)) {
            song = null;
            songs = _songs;
        } else {
            song = _songs;
            songs = [];
        }

        showMenu = true;
    }

    function append() {
        appendToQueue(songs.length ? songs : [song]);

        close();
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close} fixed>
        <MenuOption
            isDisabled={true}
            text={song ? song.title : songs.length + " tracks"}
        />
        <MenuDivider />
        <MenuOption onClick={append} text="Append to queue" />
    </Menu>
{/if}
