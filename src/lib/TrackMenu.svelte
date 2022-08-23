<script>
  import { rightClickedTrack } from "../data/store";
  import { db } from "../data/db";
  import Menu from "./menu/Menu.svelte";
  import MenuOption from "./menu/MenuOption.svelte";
  import { open } from "@tauri-apps/api/shell";
  import { get } from "svelte/store";
  import MenuDivider from "./menu/MenuDivider.svelte";

  export let pos = { x: 0, y: 0 };
  export let showMenu = false;

  async function onRightClick(e) {
    if (showMenu) {
      showMenu = false;
      await new Promise((res) => setTimeout(res, 100));
    }

    showMenu = true;
  }

  let songId;
  let isConfirmingDelete = false;

  $: {
    if (songId !== $rightClickedTrack?.id) {
      isConfirmingDelete = false;
    }
  }

  function closeMenu() {
    showMenu = false;
    isConfirmingDelete = false;
  }

  function deleteTrack() {
    if (!isConfirmingDelete) {
      songId = $rightClickedTrack?.id;
      isConfirmingDelete = true;
      return;
    }
    if ($rightClickedTrack) {
      closeMenu();
      db.songs.delete($rightClickedTrack.id);
      $rightClickedTrack = null;
      isConfirmingDelete = false;
    }
  }

  function searchArtistOnYouTube() {
    closeMenu();
    const query = encodeURIComponent($rightClickedTrack.artist);
    open(`https://www.youtube.com/results?search_query=${query}`);
  }
  function searchSongOnYouTube() {
    closeMenu();
    const query = encodeURIComponent($rightClickedTrack.title);
    open(`https://www.youtube.com/results?search_query=${query}`);
  }
  function searchArtistOnWikipedia() {
    closeMenu();
    const query = encodeURIComponent($rightClickedTrack.artist);
    open(`https://en.wikipedia.org/wiki/${query}`);
  }
  function openInFinder() {
    closeMenu();
    const query = $rightClickedTrack.path.replace($rightClickedTrack.file, "");
    open(query);
  }
  function lookUpChords() {
    closeMenu();
    const query = encodeURIComponent($rightClickedTrack.artist + ' ' + $rightClickedTrack.title + ' chords');
    open(`https://duckduckgo.com/?q=${query}`);
  }
  function lookUpLyrics() {
    closeMenu();
    const query = encodeURIComponent($rightClickedTrack.artist + ' ' + $rightClickedTrack.title + ' lyrics');
    open(`https://duckduckgo.com/?q=${query}`);
  }
</script>

{#if showMenu}
  <Menu {...pos} on:clickoutside={closeMenu}>
    <MenuOption isDisabled={true} text={$rightClickedTrack.title} />
    <MenuOption
      isDestructive={true}
      isConfirming={isConfirmingDelete}
      on:click={deleteTrack}
      text="Delete track"
      confirmText="Click again to confirm"
    />
    <MenuDivider />
    <MenuOption
      on:click={searchSongOnYouTube}
      text="Search for song on YouTube"
    />
    <MenuOption
      on:click={searchArtistOnYouTube}
      text="Search for artist on YouTube"
    />
    <MenuOption
      on:click={searchArtistOnWikipedia}
      text="Search for artist on Wikipedia"
    />
    <MenuDivider />
    <MenuOption on:click={lookUpChords} text="Look up chords" />
    <MenuOption on:click={lookUpLyrics} text="Look up lyrics" />
    <MenuDivider />

    <MenuOption on:click={openInFinder} text="Open in Finder" />
  </Menu>
{/if}
