<ul>
    <h3>Release Notes</h3>

    <section>
        <h3>📦 0.4.0 <i>"For the love of music!"</i></h3>
        <li>
            <b>Introducing the Artist's Toolkit</b> - helping you write songs
            and organize musical ideas inside your music player. How cool is
            that!
            You can use the Scrapbook to keep your "messy" ideas, tag them and easily
            find them later.
            <br />
            <br /> You can create new songs, attach audio, video, image files
            (or just drag ideas from the scrapbook), and even write lyrics!
            <br />
        </li>
        <li>
            <b>Album View!</b> Browse and play albums in your library, with all the
            artwork on display, and beautifully animated CD covers.
        </li>
        <li>
            The default sort in the Library is now the 'discography' sort. As
            you scroll, you see Artists in alphabetical order, albums in release
            date order, and tracks in the right order
        </li>
        <li>
            The default visualizer is now an oscilloscope, with a full screen
            option. More visualizers will be added in later releases.
        </li>
    </section>

    <section>
        <h3>📦 0.3.0 <i>"Enhance, enhance!"</i></h3>
        <li>
            The new Smart Query section allows you to create powerful query
            chains, made up of condition blocks such as "where genre is x" and
            "released between year1 and year2".
        </li>
    </section>

    <section>
        <h3>📦 0.2.0 <i>"Very meta"</i></h3>
        <li>
            Right-click on track -> "Info & metadata" to view and edit the
            metadata tags from the source file.
        </li>
        <li>
            You can update any existing tag, add or replace an image encoded in
            the file.
        </li>
    </section>

    <section>
        <h3>📦 0.1.1 <i>"Search and you shall find, maybe"</i></h3>
        <li>
            Search for tracks in the sidebar! Or press Cmd/Ctr + F. <br />Note:
            this only works for tracks/artists/albums that "start with" the
            query (words in the middle don't match, yet)
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

<style lang="scss">
    ul {
        background-color: rgba(0, 0, 0, 0.079);
        /* border-radius: 5px; */
        /* max-height: 200px; */
        overflow: auto;
        width: 100%;
        height: 100%;
        color: rgb(212, 212, 212);
        font-size: 1em;
        padding: 0;
        margin: 0;

        > h3 {
            position: sticky;
            top: 0;
            background-color: #333333;
            padding: 0.2em;
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
