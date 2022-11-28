<p align="center">
<img height="140" src="src-tauri/icons/Square310x310Logo.png">
</p>
<h1 align="center">Musicat</h1>
<p align="center">
<b>A sleek player for your local music library</b>
<br/>
-
<br/>
<small>🔗 linked music library, using original files on disk</small>
<br/>
<small>🏷 with metadata tagging support (read and write ID3v2, Vorbis)</small>
<br/>
<small>💿 a neat mini-player
</small>
<br/>
<small>🪕 Artist's toolkit for musicians
</small>
<br/>
  -
</p>


![screenshot](docs/screenshot-2.jpeg)

Note: musicat is currently in major version zero (0.x) active development, and features are being added regularly. Things may break or change at any time! Keep an eye out for new releases, report bugs and give feedback! 

## Features
- Music player with support for MP3, FLAC, WAV, AAC, OGG
- Albums view
- Mini-player and oscilloscope visualizer
- Sorting releases chronologically within artist sort (discography sort)
- Toggle library view between lossy/lossless/both
- Edit file metadata including embedded artwork
- Use Smart Query to create smart auto-updating playlists based on rules eg. "tracks that contains 'love' released after 1950, longer than 4 mins"

### For musicians
- Keep your messy ideas in the Scrapbook - audio/video clips, text files, links, etc. Tag them and easily find them later.
- Manage your music in the Artist's toolkit
  - Manage your artists and musical projects
  - Create song entries, add info and lyrics
  - Attach files (or drag from the scrapbook), from cool riffs to final masters.
- Look up chords/lyrics in DuckDuckGo, go to artist on Wikipedia

## Build locally
Pre-requisites: Set up the [Tauri](https://tauri.app/) framework.

You can then create a development build using `npm run tauri dev`. 
### macOS Universal build 
To generate a Universal build for macOS:
``npm run tauri build -- --target universal-apple-darwin`

### Windows, Linux
Windows and Linux builds have not been tested yet, but you can generate this using `npm run tauri build` on your system. 

## Keyboard Shortcuts
- <small><kbd>**`Cmd`**</kbd> + <kbd>**`F`**</kbd></small> : Search
- <small><kbd>**`Up`**</kbd> / <kbd>**`Down`**</kbd></small> : Highlight next, prev track (also in metadata viewer/editor)
- <small><kbd>**`ENTER`**</kbd></small> : Play highlighted track 
- <small><kbd>**`Shift`**</kbd> + <kbd>**`Click`**</kbd></small> : Highlight multiple tracks
- <small><kbd>**`Space`**</kbd></small> : Play / pause
- <small><kbd>**`I`**</kbd></small>: Show Info & Metadata

`This app is built using Svelte + Tauri.`
