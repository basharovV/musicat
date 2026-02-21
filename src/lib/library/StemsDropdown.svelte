<script lang="ts">
    import type { Song } from "../../App";
    import audioPlayer from "../player/AudioPlayer";
    import { closeCurrentMenu } from "../ui/ContextMenu";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";

    export let song: Song; // should have stems

    function close() {
        closeCurrentMenu();
    }

    function playStem(stem) {
        audioPlayer.playSong({
            ...song,
            path: stem.path,
        });
    }
</script>

<Menu onClickOutside={close}>
    <MenuOption isDisabled={true} text={"Stems"} />
    {#each song.stems as stem}
        <MenuOption
            isDisabled={false}
            text={stem.name}
            onClick={() => {
                console.log("play stem", stem);
                playStem(stem);
            }}
        />
    {/each}
</Menu>
