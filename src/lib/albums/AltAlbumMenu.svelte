<script lang="ts">
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import type { Album, Song } from "../../App";
    import { appendToQueue } from "../../data/storeHelper";

    export let onClose: () => void;

    let album: Album;
    let position = { x: 0, y: 0 };
    let showMenu = false;
    let song: Song;
    let songs: Song[];

    export function open(
        _album: Album,
        _songs: Song[],
        _position: { x: number; y: number },
    ) {
        album = _album;
        song = _songs[0];
        songs = _songs;
        position = _position;

        showMenu = true;
    }

    function close() {
        showMenu = false;

        onClose && onClose();
    }

    function append() {
        appendToQueue(songs);

        close();
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close}>
        <MenuOption
            isDisabled={true}
            text="{album.displayTitle ?? album.title} by {album.artist}"
        />
        <MenuDivider />
        <MenuOption onClick={append} text="Append to queue" />
    </Menu>
{/if}
