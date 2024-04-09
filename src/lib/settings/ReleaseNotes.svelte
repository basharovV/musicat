<div>
    <ul>
        <h3>Release Notes</h3>

        <section>
            <h3>📦 0.6.0</h3>
            <li>
                <b>Library v2. </b> The library is now rendered on a canvas, only rendering the visible rows. 
                Scrolling through large libraries is much faster and more responsive, as is resizing the window. 
            </li>
            <li>
                <b>Faster and sleeker import UX.</b> Import is now almost 10x faster, enabled by multithreading the metadata parsing in the Rust backend. 
                Also, a nice cassete animation while you wait :)
            </li>
            <li>
                <b>Lyrics view</b>. You can now view lyrics for the current song (requires Genius API key in settings).
            </li>
            <li>
                <b>Spectroscope visualizer</b> The visualizer is now positioned in the bottom bar (when enough space is available)
            </li>
            <li>
                <b>Pick up where you left off</b>. The player remembers the last song and seek position when opening the app. 
            </li>
            <li>
                <b>Add country data to your entire library at once</b>. When you don't have any origin country data, add it in bulk from the map view. 
            </li>
            <li>
                Drag and drop library columns to re-order.  
            </li>
            <li>
                <b>Auto-watch folders for changes.</b> Configure the paths to watch in settings. 
            </li>
            <li>
                ID3v1 is now supported in the tagger (will be upgraded to ID3v2.4 on write)
            </li>
            <li>
                You can now <b>Fetch album art</b> from Wikipedia from the track info overlay. 
            </li>
        </section>
        <section>
            <h3>📦 0.5.0</h3>
            <li>
                <b>A brand new Map view!</b> Click on the "MAP" view to display your
                library as a map 🗺 Click on countries to play
            </li>
            <li>
                <b>Assign Origin Country.</b>
                For the map to be useful, enrich your library with country data (right
                click track → "Origin country"), and it will be added to all songs
                by that artist.
            </li>
            <li>
                <b>Stats view with AI model support. 📊</b>
                The stats view shows you insights into your library. Play count,
                popular genre and countries, as well as a timeline view that shows
                albums by release date.
                <br />
                <br />
                <b>🤖 Get insights via an AI model.</b>
                Musicat is now integrated with two APIs - Open AI's and Ollama (local),
                which analyse your music library and provide further insights. At
                the moment, it asks the model for sentiment analysis, a brief summary
                and some fun facts about your library. To configure AI, open settings.
            </li>
            <li>
                <b>Shuffle.</b>
                You can finally shuffle the current play queue.
            </li>
            <li>
                <b>Sticky artists + albums</b>
                As you scroll, the artist and album acts as a sticky header.
            </li>
            <li>
                <b>Button to scroll to Now Playing track</b>
            </li>
            <li>
                <b>Column picker</b>. Right-click on the column header to open the column picker.
            </li>
        </section>
        <section>
            <h3>📦 0.4.0 <i>"For the love of music!"</i></h3>
            <li>
                <b>Introducing the Artist's Toolkit</b> - helping you write
                songs and organize musical ideas inside your music player. How
                cool is that! You can use the Scrapbook to keep your "messy"
                ideas, tag them and easily find them later.
                <br />
                <br /> You can create new songs, attach audio, video, image
                files (or just drag ideas from the scrapbook), and even write
                lyrics!
                <br />
            </li>
            <li>
                <b>Album View!</b> Browse and play albums in your library, with all
                the artwork on display, and beautifully animated CD covers.
            </li>
            <li>
                The default sort in the Library is now the 'discography' sort.
                As you scroll, you see Artists in alphabetical order, albums in
                release date order, and tracks in the right order
            </li>
            <li>
                The default visualizer is now an oscilloscope, with a full
                screen option. More visualizers will be added in later releases.
            </li>
        </section>

        <section>
            <h3>📦 0.3.0 <i>"Enhance, enhance!"</i></h3>
            <li>
                The new Smart Query section allows you to create powerful query
                chains, made up of condition blocks such as "where genre is x"
                and "released between year1 and year2".
            </li>
        </section>

        <section>
            <h3>📦 0.2.0 <i>"Very meta"</i></h3>
            <li>
                Right-click on track -> "Info & metadata" to view and edit the
                metadata tags from the source file.
            </li>
            <li>
                You can update any existing tag, add or replace an image encoded
                in the file.
            </li>
        </section>

        <section>
            <h3>📦 0.1.1 <i>"Search and you shall find, maybe"</i></h3>
            <li>
                Search for tracks in the sidebar! Or press Cmd/Ctr + F. <br
                />Note: this only works for tracks/artists/albums that "start
                with" the query (words in the middle don't match, yet)
            </li>
        </section>
        <section>
            <h3>📦 0.1 <i>"The first one"</i></h3>
            <li>Music player with support MP3, FLAC, WAV, AAC, OGG</li>
            <li>Library database that links to original file location</li>
            <li>Reading ID3, ID3v2 and Vorbis (FLAC) tags</li>
            <li>Sorting by title, artist, album, genre or year</li>
        </section>
    </ul>
</div>

<style lang="scss">
    div {
        overflow: auto;
        height: 100%;
    }
    ul {
        /* border-radius: 5px; */
        /* max-height: 200px; */
        width: 100%;
        height: fit-content;
        color: rgb(212, 212, 212);
        font-size: 1em;
        padding: 0;
        margin: 0;

        > h3 {
            position: sticky;
            top: 0;
            background-color: #333333;
            padding: 0.2em;
            margin: 0;
            z-index: 5;
        }
        section {
            padding: 1em 2em;
            i {
                opacity: 0.3;
                margin-left: 0.2em;
                font-size: 14px;
            }
            li,
            h3 {
                width: fit-content;
                text-align: left;
            }

            li {
                margin-inline-start: 1em;
                text-align: left;
                margin-top: 0.6em;
            }
        }
    }
</style>
