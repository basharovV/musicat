<ul>
    <h3>Release Notes</h3>

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
            Search for tracks in the sidebar! Or press Cmd/Ctr + F. <br/>Note: this only works for
            tracks/artists/albums that "start with" the query (words in the
            middle don't match, yet)
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
            }
        }
    }
</style>
