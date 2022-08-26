<script lang="ts">
  import "iconify-icon";
  import { db } from "../data/db";
  import { get } from "svelte/store";
  import { openTauriImportDialog } from "../data/LibraryImporter";
  import {
    currentSong,
    currentSongIdx,
    os,
    queriedSongs,
    query,
    rightClickedTrack,
    rightClickedTracks,
    songsJustAdded,
  } from "../data/store";
  import AudioPlayer from "./AudioPlayer";
  import TrackMenu from "./TrackMenu.svelte";
  import { liveQuery } from "dexie";
  import ImportPlaceholder from "./ImportPlaceholder.svelte";
  import hotkeys from "hotkeys-js";
  import { version } from "@tauri-apps/api/os";

  /**
   * Song that's currently playing
   */
  let currentIndex = 0;

  //   let songs = liveQuery(() => {
  //     let results = db.songs.orderBy(orderBy);
  //     if (reverse) {
  //       results = results.reverse();
  //     }
  //     let toShow = results.toArray();
  //     return results.toArray();
  //   });

  $: songs = liveQuery(async () => {
    let results;
    if ($query.query.length) {
      results = db.songs
        .where("title")
        .startsWithIgnoreCase($query.query)
        .or("artist")
        .startsWithIgnoreCase($query.query)
        .or("album")
        .startsWithIgnoreCase($query.query);
    } else {
      results = db.songs.orderBy($query.orderBy);
    }
    if ($query.reverse) {
      results = results.reverse();
    }
    let resultsArray = await results.toArray();
    queriedSongs.set(resultsArray);
    return resultsArray;
  });

  $: count = liveQuery(() => {
    return db.songs.count();
  });

  $: artistCount = liveQuery(async () => {
    const artists = await db.songs.orderBy("artist").uniqueKeys();
    return artists.length;
  });

  $: albumCount = liveQuery(async () => {
    const albums = await db.songs.orderBy("album").uniqueKeys();
    return albums.length;
  });

  $: noSongs = !$songs || $songs.length === 0;

  function updateOrderBy(newOrderBy) {
    if ($query.orderBy === newOrderBy) {
      $query.reverse = !$query.reverse;
    }
    $query.orderBy = newOrderBy;
    $query = $query;
  }

  let removeFromJustAdded = [];

  async function clearJustAdded(songId) {
    setTimeout(() => {
      $songsJustAdded = $songsJustAdded.filter((s) => s.id !== songId);
      $songsJustAdded = $songsJustAdded;
    }, 1000);
  }

  function isSongJustAdded(songId) {
    // TODO optimize using songjustadded (boolean)
    const isAdded = $songsJustAdded.map((s) => s.id).includes(songId);
    if (isAdded) {
      clearJustAdded(songId);
    }
    return isAdded;
  }

  let songIdsHighlighted: Song[] = [];

  function isSongHighlighted(song: Song) {
    return songIdsHighlighted.map((s) => s.id).includes(song.id);
  }

  function toggleHighlight(song, idx, isKeyboardArrows = false) {
    highlightedSongIdx = idx;
    if (isSongHighlighted(song)) {
      if (songIdsHighlighted.length) {
        songIdsHighlighted = [];
        highlightSong(song, idx, isKeyboardArrows);
      } else {
        unhighlightSong(song);
      }
    } else {
      highlightSong(song, idx, isKeyboardArrows);
    }
  }

  let isCmdOrCtrlPressed = false;
  let isShiftPressed = false;
  let rangeStartSongIdx = null;
  let rangeEndSongIdx = null;
  let highlightedSongIdx = 0;

  // Shortcuts
  if ($os === "Darwin") {
    hotkeys("cmd", function (event, handler) {
      isCmdOrCtrlPressed = true;
    });
  } else if ($os === "Windows_NT" || $os === "Linux") {
    hotkeys("ctrl", function (event, handler) {
      isCmdOrCtrlPressed = true;
    });
  }
  // hotkeys("up", function (event, handler) {});

  // hotkeys("down", function (event, handler) {});
  addEventListener("keydown", (event) => {
    if (event.keyCode === 16) {
      isShiftPressed = true;
      console.log("shift pressed");
    } else if (event.keyCode === 38) {
      event.preventDefault();
      // up
      if (highlightedSongIdx > 0) {
        toggleHighlight($songs[highlightedSongIdx - 1], highlightedSongIdx - 1, true);
      }
    } else if (event.keyCode === 40) {
      event.preventDefault();
      if (highlightedSongIdx < $songs.length) {
        toggleHighlight($songs[highlightedSongIdx + 1], highlightedSongIdx + 1, true);
      }
    }
  });
  addEventListener("keyup", (event) => {
    if (event.keyCode === 16) {
      isShiftPressed = false;
      console.log("shift lifted");
    }
  });

  function highlightSong(song: Song, idx, isKeyboardArrows: boolean) {
    console.log("highlighted", song, idx);
    if (!isKeyboardArrows && isShiftPressed) {
      if (rangeStartSongIdx === null) {
        rangeStartSongIdx = idx;
      } else {
        rangeEndSongIdx = idx;
        // Highlight all the songs in between

        if (rangeEndSongIdx < rangeStartSongIdx) {
          let startIdx = rangeStartSongIdx;
          rangeStartSongIdx = rangeEndSongIdx;
          rangeEndSongIdx = startIdx;
        }
        songIdsHighlighted = $songs.slice(
          rangeStartSongIdx,
          rangeEndSongIdx + 1
        );
        rangeStartSongIdx = null;
        rangeEndSongIdx = null;
        $rightClickedTrack = null;
        console.log("highlighted2", songIdsHighlighted);
      }
    } else if (
      (isKeyboardArrows && isShiftPressed) || 
      isCmdOrCtrlPressed ||
      hotkeys.isPressed(91)
    ) {
      songIdsHighlighted.push(song);
      rangeStartSongIdx = idx;
    } else {
      songIdsHighlighted = [song];
      $rightClickedTracks = [];
      $rightClickedTracks = $rightClickedTracks;
      rangeStartSongIdx = idx;
    }
    console.log("start", rangeStartSongIdx);

    songIdsHighlighted = songIdsHighlighted;
  }

  function unhighlightSong(song: Song) {
    songIdsHighlighted.splice(songIdsHighlighted.indexOf(song), 1);
    songIdsHighlighted = songIdsHighlighted;
  }

  let showTrackMenu = false;
  let pos;

  function onRightClick(e, song, idx) {
    if (!songIdsHighlighted.includes(song)) {
      highlightSong(song, idx, false);
    }

    console.log("songIdsHighlighted", songIdsHighlighted);
    if (songIdsHighlighted.length > 1) {
      $rightClickedTracks = songIdsHighlighted;
      $rightClickedTrack = null;
    } else {
      $rightClickedTrack = song;
    }
    showTrackMenu = true;
    pos = { x: e.clientX, y: e.clientY };
  }

  function onDoubleClickSong(song, idx) {
    $currentSongIdx = idx;
    AudioPlayer.playSong(song);
  }

  const onAudioEnded = () => {
    console.log("audio ended");
    playNext();
  };

  function playNext() {
    AudioPlayer.playSong($songs[++$currentSongIdx]);
  }

  // Play next automatically
  AudioPlayer.setAudioFinishedCallback(onAudioEnded);
</script>

{#if noSongs}
  <ImportPlaceholder />
{:else}
  <library>
    <table>
      <thead>
        <td on:click={() => updateOrderBy("title")}
          ><div>
            Title
            {#if $query.orderBy === "title"}
              {#if $query.reverse}
                <iconify-icon icon="heroicons-solid:sort-descending" />
              {:else}
                <iconify-icon icon="heroicons-solid:sort-ascending" />
              {/if}
            {/if}
          </div></td
        >
        <td on:click={() => updateOrderBy("artist")}
          ><div>
            Artist
            {#if $query.orderBy === "artist"}
              {#if $query.reverse}
                <iconify-icon icon="heroicons-solid:sort-descending" />
              {:else}
                <iconify-icon icon="heroicons-solid:sort-ascending" />
              {/if}
            {/if}
          </div></td
        >
        <td on:click={() => updateOrderBy("album")}
          ><div>
            Album
            {#if $query.orderBy === "album"}
              {#if $query.reverse}
                <iconify-icon icon="heroicons-solid:sort-descending" />
              {:else}
                <iconify-icon icon="heroicons-solid:sort-ascending" />
              {/if}
            {/if}
          </div></td
        >
        <td on:click={() => updateOrderBy("year")}
          ><div>
            Year
            {#if $query.orderBy === "year"}
              {#if $query.reverse}
                <iconify-icon icon="heroicons-solid:sort-descending" />
              {:else}
                <iconify-icon icon="heroicons-solid:sort-ascending" />
              {/if}
            {/if}
          </div></td
        >
        <td on:click={() => updateOrderBy("genre")}
          ><div>
            Genre
            {#if $query.orderBy === "genre"}
              {#if $query.reverse}
                <iconify-icon icon="heroicons-solid:sort-descending" />
              {:else}
                <iconify-icon icon="heroicons-solid:sort-ascending" />
              {/if}
            {/if}
          </div></td
        >
      </thead>
      {#if $songs}
        {#each $songs as song, idx (song.id)}
          <tr
            class:playing={get(currentSong) && song.id === $currentSong.id}
            class:just-added={$songsJustAdded && isSongJustAdded(song.id)}
            class:highlight={songIdsHighlighted && isSongHighlighted(song)}
            on:contextmenu|preventDefault={(e) => onRightClick(e, song, idx)}
            on:click={() => toggleHighlight(song, idx)}
            on:dblclick={() => onDoubleClickSong(song, idx)}
          >
            <td>
              <div>
                {song.title === "" ? "-" : song.title}
                {#if get(currentSong) && song.id === $currentSong.id}
                  <iconify-icon icon="heroicons-solid:volume-up" />
                {/if}
              </div>
            </td>
            <td>{song.artist}</td>
            <td>{song.album}</td>
            <td>{song.year === 0 ? "-" : song.year}</td>
            <td>{song.genre}</td>
          </tr>
        {/each}
      {/if}
    </table>
    <button style="margin-top:2em" on:click={openTauriImportDialog}
      >Add music +</button
    >
    <bottom-bar>
      <p>{$count} songs</p>
      <p>{$artistCount} artists</p>
      <p>{$albumCount} albums</p>
    </bottom-bar>
  </library>
{/if}

<TrackMenu bind:showMenu={showTrackMenu} bind:pos />

<style lang="scss">
  $odd_color: rgb(42, 38, 40);
  $even_color: rgb(42, 38, 40);
  $selected_color: #5123dd;
  $playing_text_color: #00ddff;
  $highlight_color: #2a5b8c45;
  $text_color: rgb(211, 211, 211);
  $added_color: rgb(44, 147, 44);

  library {
    height: 100vh;
    overflow: auto;
    font-size: 0.9em;
    background-color: rgb(36, 33, 34);
    position: relative;
    overflow: overlay;
  }

  bottom-bar {
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: rgba(28, 26, 26, 0.645);
    backdrop-filter: blur(8px);
    border-top: 1px solid rgb(51, 51, 51);
    color: white;
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 1em;
    justify-content: flex-end;
    padding: 0 1em;
    width: 100%;
    height: 30px;

    p {
      color: rgb(176, 161, 161);
      margin: 0;
    }
  }

  table {
    -webkit-border-horizontal-spacing: 0px;
    -webkit-border-vertical-spacing: 0px;
    width: 100%;
    thead {
      font-weight: bold;
      background-color: #71658e7e;
      position: sticky;
      top: 0;
      backdrop-filter: blur(8px);
      > td {
        border-right: none;
        div {
          display: flex;
          align-items: center;
          justify-content: space-between;
        }
        &:hover {
          cursor: ns-resize;
          background-color: #604d8d;
        }
      }
    }

    td {
      text-align: left;
      user-select: none;
      cursor: default;
      padding-inline-start: 1em;
      padding-inline-end: 1em;
      border-right: 0.5px solid rgba(242, 242, 242, 0.144);
      max-width: 200px;
      overflow: hidden;
      text-overflow: ellipsis;
      div {
        display: flex;
        align-items: center;
        justify-content: space-between;
      }
    }

    tr {
      white-space: nowrap;
      user-select: none;
      color: $text_color;
      &.highlight {
        background-color: $highlight_color !important;
      }
      &.playing {
        background: $selected_color !important;
        color: $playing_text_color;
      }
      &.just-added {
        background-color: $added_color !important;
        color: white;
      }
      &:nth-child(odd) {
        background-color: $odd_color;

        &:hover {
          background-color: #1f1f1f;
        }
      }
      &:nth-child(even) {
        background-color: $even_color;

        &:hover {
          background-color: #1f1f1f;
        }
      }
    }
  }
</style>
