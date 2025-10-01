<p align="center">
  <img height="140" src="src-tauri/icons/Square310x310Logo.png">
</p>
<h1 align="center">Musicat</h1>

<p align="center">
  <a href="https://github.com/basharovV/musicat/actions"><img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/basharovV/musicat/build.yml?branch=main"></a>
  <a href="https://github.com/basharovV/musicat/releases"><img alt="Latest Release" src="https://img.shields.io/github/v/release/basharovV/musicat"></a>
  <a href="https://github.com/basharovV/musicat/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/github/license/basharovV/musicat"></a>
</p>

<p align="center">
<b>A sleek player for your local music library</b>
<br/>
-
<br/>
<small>🎵 supports MP3, FLAC, WAV, AAC, OGG</small>
<br/>
<small>🔗 linked library, using original files on disk</small>
<br/>
<small>🔊 gapless playback (same sample rate only)</small>
<br/>
<small>🏷 metadata tagger (read and write ID3v2, Vorbis)</small>
<br/>
<small>👀 auto-watch and re-scan folders</small>
<br/>
<small>🧠 smart playlists</small>
<br/>
<small>🖼 download album art, origin country from Wikipedia
</small>
<br/>
<small>🎤 fetch lyrics for current song
</small>
<br/>
<small>🗺 World Map view - see your library on a map
</small>
<br/>
<small>💿 a neat mini-player
</small>
<br/>
<small>🎸 U2's latest album automatically added to your library
</small>
<br/>
<small>📊 Stats, album timeline
</small>
<br/>
<small>🌊 Waveform view
</small>
<br/>
  -
</p>

![screenshot](docs/musicat-aug-2024-promo.webp)

> [!WARNING]
> musicat is currently in major version zero (0.x) active development, and features are being added regularly. Things may break or change at any time! Keep an eye out for new releases, report bugs and give feedback!

## Introduction

This is a player that I built for personal use, with the goal of re-connecting with my local music library in the age of streaming. It's pleasant to use, with a modern UX that is fresh yet feels familiar to classic software like Winamp, foobar2000 and iTunes. It's fast, audiophile-friendly and can handle large libraries of all formats. I wanted to create a tool that puts me as the listener back in the driver's seat, taking an active role in organizing and enriching my library of music that I curate with passion.

## For melomaniacs

Musicat also strives to be a meta-layer on top of your library. Features like Map View and Stats are all derived from the question - _"how can I connect more with my music collection?"_. They might seem like gimmicks, but there is a lot of room to explore this idea, and this app is the perfect playground to do that. Whether it's learning about the artists via a Wikipedia panel, or getting a glimpse into your listening habits, you can expect more functionality of this sort to be built into the app.

## For artists & musicians

As a musician, I also wanted to organize my own music that I've made over the years, or tag bands that I've played in, and use the app as a way to build your own discography, as well as a knowledge base of new song ideas and lyrics. I created a prototype feature called "Artist's Toolkit" (enable in Settings) that captures this idea. It's half-baked at the moment, but it has already proven useful as a concept.

## Screenshots

<table>
  <tr>
    <td align="center"><p>Albums</p><img src="docs/albums.webp" width="320" alt="Albums"></td>
    <td align="center"><p>Queue</p><img src="docs/queue.jpg" width="320" alt="Queue"></td>
    <td align="center"><p>Track Info / Tagger</p><img src="docs/track-info.webp" width="320" alt="Track Info"></td>
  </tr>
  <tr>
    <td align="center"><p>Smart Playlists</p><img src="docs/smart-query.webp" width="320" alt="Smart Query"></td>
    <td align="center"><p>Map</p><img src="docs/map.jpg" width="320" alt="Map"></td>
    <td align="center"><p>Stats</p><img src="docs/stats.jpg" width="320" alt="Stats"></td>
  </tr>
</table>

## Keyboard Shortcuts

- <small><kbd>**`Cmd/Ctrl`**</kbd> + <kbd>**`F`**</kbd></small> : Search
- <small><kbd>**`Up`**</kbd> / <kbd>**`Down`**</kbd></small> : Highlight next, prev track (also in metadata viewer/editor). <small><kbd>**`Shift`**</kbd> to multi-select
- <small><kbd>**`ENTER`**</kbd></small> : Play highlighted track
- <small><kbd>**`Shift`**</kbd> + <kbd>**`Click`**</kbd></small> : Highlight multiple tracks
- <small><kbd>**`Space`**</kbd></small> : Play / pause
- <small><kbd>**`I`**</kbd></small>: Show Info & Metadata
- <small><kbd>**`Alt`**</kbd> + <kbd>**`Q`**</kbd></small>: Open the Queue
- <small><kbd>**`Alt`**</kbd> + <kbd>**`A`**</kbd></small>: Go to Albums
- <small><kbd>**`Alt`**</kbd> + <kbd>**`L`**</kbd></small>: Go to Library
- <small><kbd>**`Cmd/Ctrl`**</kbd> + <kbd>**`L`**</kbd></small>: Open Lyrics

`This app is built using Svelte + Tauri.`

## Running on Linux 🐧

Musicat uses ALSA on Linux, install it if you don't already have it on your system:
