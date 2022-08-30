<script lang="ts">
  import {
    isInfoPopupOpen,
    isTrackInfoPopupOpen,
    rightClickedTrack,
    rightClickedTracks,
  } from "../data/store";
  import { db } from "../data/db";
  import Menu from "./menu/Menu.svelte";
  import MenuOption from "./menu/MenuOption.svelte";
  import { open } from "@tauri-apps/api/shell";
  import { get } from "svelte/store";
  import MenuDivider from "./menu/MenuDivider.svelte";
  import { type, type OsType } from "@tauri-apps/api/os";
  import { onMount } from "svelte";

  export let pos = { x: 0, y: 0 };
  export let showMenu = false;
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

  let songId;
  let isConfirmingDelete = false;

  $: {
    if (
      $rightClickedTracks?.map((s) => s.id).includes(songId) ||
      songId !== $rightClickedTrack?.id
    ) {
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
    if ($rightClickedTracks.length) {
      closeMenu();
      $rightClickedTracks.forEach((t) => {
        db.songs.delete(t.id);
      });
      $rightClickedTracks = [];
      isConfirmingDelete = false;
    } else if ($rightClickedTrack) {
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
    const query = encodeURIComponent(
      $rightClickedTrack.artist + " " + $rightClickedTrack.title + " chords"
    );
    open(`https://duckduckgo.com/?q=${query}`);
  }
  function lookUpLyrics() {
    closeMenu();
    const query = encodeURIComponent(
      $rightClickedTrack.artist + " " + $rightClickedTrack.title + " lyrics"
    );
    open(`https://duckduckgo.com/?q=${query}`);
  }
  function openInfo() {
    $isTrackInfoPopupOpen = true;
  }
</script>

{#if showMenu}
  <Menu {...pos} on:clickoutside={closeMenu}>
    <MenuOption
      isDisabled={true}
      text={$rightClickedTrack
        ? $rightClickedTrack.title
        : $rightClickedTracks.length + " tracks"}
    />
    <MenuOption
      isDestructive={true}
      isConfirming={isConfirmingDelete}
      on:click={deleteTrack}
      text={$rightClickedTrack ? "Delete track" : "Delete tracks"}
      confirmText="Click again to confirm"
    />
    {#if $rightClickedTrack}
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

      <MenuOption on:click={openInFinder} text="Open in {explorerName}" />
      <MenuOption on:click={openInfo} text="Info & metadata" />
    {/if}
  </Menu>
{/if}
