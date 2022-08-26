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
  import { convertFileSrc } from "@tauri-apps/api/tauri";

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
        convertFileSrc(song.path)
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
        lookForArt();
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

  async function lookForArt() {
    const folder = $currentSong.path.replace($currentSong.file, "");

    // Check are there any images in the folder?
    try {
      const src = "asset://" + folder + "folder.jpg";
      const response = await fetch(src);
      if (response.status === 200) {
        console.log("got art!", response);
        artworkSrc = src;
        artworkFormat = "image/jpeg";
      }
    } catch (err) {
      console.error("Couldn't find artwork " + err);
    }
  }
</script>

<sidebar>
  <h1 class="app-title" on:click={openInfoWindow}>Musicat</h1>
  <!-- <div class="knob">
    <Knob bind:value={volumeKnob} max={100} min={0} pixelRange={200} />
  </div> -->
  <div class="top">
    <div class="search-container">
      <input
        class="search"
        type="text"
        placeholder="Search..."
        bind:value={$query.query}
      />
    </div>

    <menu>
      <items>
        <item> <iconify-icon icon="fluent:library-20-filled" />Music</item>
        <!-- <item> <iconify-icon icon="mdi:playlist-music" />Playlists</item> -->

        <!-- <hr />
    <item>
      <iconify-icon
        icon="iconoir:album-carousel"
      />Samples</item
    > -->
      </items>
    </menu>
  </div>

  <div class="track-info">
    <hr />

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
        <p class="is-placeholder">Take control of your library</p>
      {/if}
    </div>
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
  {#if $currentSong}
    <div class="artwork-container">
      <div class="artwork-frame">
        {#if artworkSrc && artworkFormat}
          <img type={artworkFormat} class="artwork" src={artworkSrc} />
        {:else}
          <div class="artwork-placeholder">
            <iconify-icon icon="mdi:music-clef-treble" on:click={playPrev} />
            <!-- <small>No art</small> -->
          </div>
        {/if}
      </div>
    </div>
  {/if}
  <div class="bottom">
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
    margin-bottom: 0;
    cursor: default;
    &:hover {
      opacity: 0.5;
    }
  }

  hr {
    width: 100%;
    border-top: 1px solid rgba(141, 139, 139, 0.139);
    border-bottom: none;
    border-left: none;
    border-right: none;
  }
  menu {
    width: 100%;
    margin: 0;
    user-select: none;
    padding: 1em;
    margin-block-start: 0;

    background-color: #242026;
    items {
      display: flex;
      flex-direction: column;
      border-radius: 3px;
      gap: 3px;
    }

    item {
      text-transform: uppercase;
      font-weight: bold;
      text-align: left;
      width: fit-content;
      padding: 0.3em 0.5em;
      font-size: 13px;
      letter-spacing: 1px;
      color: rgb(181, 182, 186);

      cursor: default;
      &:hover {
        color: rgb(255, 255, 255);
      }
      &:active {
        color: rgb(130, 130, 130);
      }

      iconify-icon {
        margin-right: 5px;
        font-size: 15px;
        text-align: center;
        vertical-align: middle;
        opacity: 0.4;
      }
    }
  }

  .top {
    width: 100%;
    position: sticky;
    top: 0;
  }

  .bottom {
    width: 100%;
    position: sticky;
    top: 400px;
  }

  .search-container {
    padding: 1em;
    width: 100%;
    background-color: #242026;

    .search {
      margin: 0;
      width: 100%;
      height: 30px;
      border-radius: 3px;
      padding-left: 5px;
      font-size: 13px;
      color: white;
      backdrop-filter: blur(8px);
      z-index: 10;
      &::placeholder {
        color: white;
      }
      &:focus {
        /* outline: 1px solid #5123dd; */
        background-color: #504c4c;
      }
      background-color: #6061703a;

      border: 1px solid rgb(63, 63, 63);
    }
  }

  .track-info {
    height: 100%;
    width: 100%;

    background-color: #242026;
    position: sticky;
    top: 110px;
  }

  .spectrum {
    z-index: 1;
    position: absolute;
    bottom: -5px;
    pointer-events: none;
    opacity: 0.2;
  }

  .cd-gif {
    margin-top: 1em;
    width: 30px;
    height: auto;
    margin-left: 10px;
    align-self: center;
    z-index: 0;
  }

  .artwork-container {
    padding: 0em;
    width: 100%;
    height: 200px;

    .artwork-frame {
      width: 100%;
      height: 100%;
      box-sizing: border-box;
      /* border-radius: 3px; */
      /* border: 1px solid rgb(94, 94, 94); */
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
          /* margin-top: 0.7em; */
        }
      }
    }
  }

  .file {
    position: sticky;
    top: 270px;
    background-color: #242026;

    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 8px 0;
    user-select: none;
    pointer-events: none;
    width: 100%;
    color: rgb(125, 125, 125);

    p {
      background-color: rgba(85, 85, 85, 0.162);
      padding: 0em 0.6em;
      margin: 0;
      border-radius: 2px;
      font-size: 0.7em;
      line-height: 1.5em;
      font-weight: 600;
      border-top: 1px solid rgb(49, 49, 49);
      /* border-bottom: 1px dashed rgb(49, 49, 49); */
      font-family: monospace;
      text-transform: uppercase;
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
      z-index: 1;
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

    .is-placeholder {
      font-family: "Snake", Courier, monospace;
      font-size: 2em;
      line-height: 1.2em;
      margin: 0 1em;
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
    height: 100%;
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
