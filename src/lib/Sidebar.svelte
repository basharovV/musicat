<script lang="ts">
  import * as musicMetadata from "music-metadata-browser";
  import {
    currentSong,
    currentSongIdx,
    isPlaying,
    query,
    queriedSongs,
    volume,
    isInfoPopupOpen,
  } from "../data/store";
  import AudioPlayer from "./AudioPlayer";
  import Seekbar from "./Seekbar.svelte";
  import SpectrumAnalyzer from "./SpectrumAnalyzer.svelte";
  import Marquee from "svelte-fast-marquee";
  import Knob from "./Knob.svelte";
  import hotkeys from "hotkeys-js";
  import { window } from "@tauri-apps/api";
  import { WebviewWindow } from "@tauri-apps/api/window";

  // What to show in the sidebar
  let title;
  let artist;
  let album;
  let artworkFormat;
  let artworkBuffer: Buffer;
  let artworkSrc;
  let codec;

  let duration;

  $: if (codec === "MPEG 1 Layer 3") {
    codec = "MP3";
  }

  let bitrate;
  let sampleRate;

  // Shortcuts
  hotkeys("space", function (event, handler) {
    // Prevent the default refresh event under WINDOWS system
    event.preventDefault();
    AudioPlayer.togglePlay();
  });

  currentSong.subscribe(async (song) => {
    if (song) {
      const metadata = await musicMetadata.fetchFromUrl(
        window.__TAURI__.tauri.convertFileSrc(song.path)
      );
      console.log("metadata", metadata);
      title = metadata.common.title;
      artist = metadata.common.artist;
      album = metadata.common.album;
      codec = metadata.format.codec;
      bitrate = metadata.format.bitsPerSample;
      sampleRate = metadata.format.sampleRate;
      duration = metadata.format.duration;
      if (metadata.common.picture?.length) {
        artworkFormat = metadata.common.picture[0].format;
        artworkBuffer = metadata.common.picture[0].data;
        artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
          "base64"
        )}`;
        console.log("artworkSrc", artworkSrc);
      } else {
        artworkSrc = null;
      }
    }
  });

  function togglePlayPause() {
    if (!AudioPlayer.audioFile.src) {
      AudioPlayer.playSong($queriedSongs[$currentSongIdx]);
    } else {
      AudioPlayer.togglePlay();
    }
  }

  function playNext() {
    AudioPlayer.playSong($queriedSongs[++$currentSongIdx]);
  }

  function playPrev() {
    if ($currentSongIdx === 1) {
      return;
    }
    AudioPlayer.playSong($queriedSongs[--$currentSongIdx]);
  }

  function openInfoWindow() {
    // const webview = new WebviewWindow("theUniqueLabel", {
    //   url: "path/to/page.html",
    // });
    console.log("clicked");
    $isInfoPopupOpen = true;
  }
</script>

<sidebar>
  <h1 class="app-title" on:click={openInfoWindow}>Musicat</h1>
  <!-- <div class="knob">
    <Knob bind:value={volumeKnob} max={100} min={0} pixelRange={200} />
  </div> -->
  <div class="search-container">
    <input
      class="search"
      type="text"
      placeholder="Search"
      bind:value={$query.query}
    />
  </div>
  <img class="cd-gif" src="images/cd6.gif" />

  <div class="info">
    {#if title}
      <p class="title">{title}</p>
    {/if}
    {#if artist}
      <p class="artist">{artist}</p>
    {/if}
    {#if album}
      <small>{album}</small>
    {/if}
    {#if !artist && !title && !album}
      <p>Take control of your library</p>
    {/if}
  </div>

  {#if codec}
    <div class="file">
      <p>{codec}</p>
      {#if bitrate}<p>{bitrate} bit</p>{/if}
      <p>{sampleRate} smpls</p>
    </div>
  {/if}

  <div class="spectrum">
    <SpectrumAnalyzer />
  </div>
  <div class="artwork-container">
    <div class="artwork-frame">
      {#if artworkSrc && artworkFormat}
        <img type={artworkFormat} class="artwork" src={artworkSrc} />
      {:else}
        <div class="artwork-placeholder">
          <iconify-icon icon="mdi:music-clef-treble" on:click={playPrev} />
          <small>Drag art into here to add</small>
        </div>
      {/if}
    </div>
  </div>
  <div class="seekbar">
    <Seekbar {duration} />
  </div>
  <transport>
    <iconify-icon icon="fe:backward" on:click={playPrev} />
    <iconify-icon
      on:click={togglePlayPause}
      icon={$isPlaying ? "fe:pause" : "fe:play"}
    />
    <iconify-icon icon="fe:forward" on:click={playNext} />
  </transport>

  <div class="volume">
    <input
      type="range"
      min="0"
      max="1"
      step="0.01"
      bind:value={$volume}
      class="slider"
      id="myRange"
    />
  </div>
</sidebar>

<style lang="scss">
  $thumb_size: 22px;

  sidebar {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: flex-end;
    height: 100vh;
    max-width: 210px;
    min-width: 210px;
    border-right: 1px solid #ececec1c;
    background-color: #242026;
    overflow: hidden;
  }

  .knob {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3em;
    height: 100%;
  }

  .app-title {
    font-family: "2Peas";
    width: 100%;
    font-size: 2em;
    opacity: 0.2;
    user-select: none;
    cursor: default;
    &:hover {
      opacity: 0.5;
    }
  }

  .search-container {
    width: 100%;
    padding: 1em;
    height: 100%;

    .search {
      margin: 0;
      width: 100%;
      height: 30px;
      border-radius: 3px;
      padding-left: 5px;
      font-size: 13px;
      color: white;
      &::placeholder {
        color: white;
      }
      &:focus {
        outline: 1px solid #5123dd;
        background-color: #504c4c;
      }
      background-color: #343030;

      border: 1px solid rgb(63, 63, 63);
    }
  }

  .spectrum {
    z-index: 1;
    position: absolute;
    bottom: -5px;
    pointer-events: none;
    opacity: 0.2;
  }

  .cd-gif {
    width: 30px;
    margin-left: 10px;
    position: relative;
    align-self: center;
    z-index: 0;
  }

  .artwork-container {
    padding: 0.2em;
    width: 100%;
    height: 200px;

    .artwork-frame {
      width: 100%;
      height: 100%;
      box-sizing: border-box;
      border-radius: 3px;
      border: 1px solid rgb(94, 94, 94);
      display: flex;
      align-items: center;
      justify-content: center;

      .artwork-placeholder {
        opacity: 0.2;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1em;
        iconify-icon {
          margin-top: 0.7em;
        }
      }
    }
  }

  .file {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 8px 0;
    user-select: none;
    pointer-events: none;
    width: 100%;
    font-size: 0.8em;
    color: rgb(125, 125, 125);
    font-weight: 400;

    p {
      text-transform: lowercase;
      /* background-color: rgba(85, 85, 85, 0.373); */
      padding: 0em 0.6em;
      margin: 0;
      border-radius: 2px;
      border: 1px solid rgb(49, 49, 49);
    }
  }
  .info {
    background-color: #242026;
    color: white;
    width: 100%;
    padding: 0.8em 0.3em;

    .artist {
      white-space: nowrap;
      font-weight: bold;
      font-size: 0.9em;
      opacity: 0.8;
    }
    .title {
      white-space: nowrap;
      font-weight: bold;
    }
    small {
      white-space: nowrap;
    }
    * {
      margin: 0;
    }
  }
  @keyframes marquee {
    0% {
      transform: translate(0%, 0);
    }
    100% {
      transform: translate(-100%, 0);
    }
  }

  img {
    width: 100%;
    height: auto;
    border-radius: 2px;
  }

  .seekbar {
    width: 100%;
  }
  transport {
    /* background-color: rgb(255, 255, 255); */
    padding: 0em 1em 0 1em;
    width: 100%;
    color: white;
    z-index: 2;
  }

  .volume {
    padding: 0 2em 1em;
    width: 100%;

    input {
      -webkit-appearance: none;
      width: 100%;
      height: 5px;
      background: #474747;
      outline: none;
      opacity: 1;
      border-radius: 3px;
      -webkit-transition: 0.2s;
      transition: opacity 0.2s;

      &::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: $thumb_size;
        height: $thumb_size;
        background: url("images/volume-up.svg");
      }

      &::-moz-range-thumb {
        width: $thumb_size;
        height: $thumb_size;
        background: #04aa6d;
      }
    }
  }

  iconify-icon {
    font-size: 40px;

    &:hover {
      opacity: 0.5;
    }

    &:active {
      color: rgb(141, 47, 47);
      opacity: 1;
    }
  }
</style>
