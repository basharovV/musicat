<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { Album, GetHTMLResponse, Song } from "../../App";
    import wtf, { type Document } from "wtf_wikipedia";
    import type { Section } from "wtf_wikipedia";
    import wtfHtml from "wtf-plugin-html";
    import ShadowGradient from "../ui/ShadowGradient.svelte";
    import { getWikipediaUrlForArtist } from "../../data/WikipediaAPI";
    import {
        albumPlaylist,
        currentSong,
        isPlaying,
        isWikiOpen,
        playlist,
        playlistType,
        queriedSongs,
        wikiArtist,
    } from "../../data/store";
    import { fade, fly } from "svelte/transition";
    import Icon from "../ui/Icon.svelte";
    import { onDestroy, onMount } from "svelte";
    import { db } from "../../data/db";
    import audioPlayer from "../player/AudioPlayer";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import LL from "../../i18n/i18n-svelte";
    wtf.extend(wtfHtml);

    let wikiResult: GetHTMLResponse;
    let wtfResult: Document;
    let error;
    let previousArtist = null;
    let isLoading = false;
    let wikiSong: Song;
    $: sections = wtfResult?.sections();
    async function getWiki(artist: string) {
        if (!artist) return;
        isLoading = true;
        try {
            const url = await getWikipediaUrlForArtist(artist);
            wikiResult = await invoke<GetHTMLResponse>("get_wikipedia", {
                event: {
                    url,
                },
            });
            console.log("result", wikiResult);
            error = null;
        } catch (err) {
            error = err;
        } finally {
            isLoading = false;
            previousArtist = artist;
        }
    }

    async function getWikiWtf(artist: string) {
        isLoading = true;
        try {
            const url = await getWikipediaUrlForArtist(artist);
            const result = await wtf.fetch(url);
            // Check if array
            if (result instanceof Array) {
            } else {
                // console.log("result", result.sentences());
                wtfResult = result;
            }

            error = null;
        } catch (err) {
            error = err;
        } finally {
            isLoading = false;
            previousArtist = $currentSong?.artist;
        }
    }

    let isMounted = false;
    let container: HTMLDivElement;
    let scrollContainer: HTMLDivElement;

    onMount(() => {
        isMounted = true;
        $currentSong?.artist && getWiki($wikiArtist || $currentSong.artist);
        // $currentSong?.artist && getWikiWtf($currentSong.artist);
    });

    onDestroy(() => {
        $wikiArtist = null;
    });

    $: if (
        isMounted &&
        wikiResult &&
        $isPlaying !== undefined &&
        $currentSong
    ) {
        enrichLinks();
    }

    let albumMentions: Mention<Album>[] = [];
    let songMentions: Mention<Song>[] = [];
    let artistMentions: Mention<string>[] = [];

    function onAlbumClicked(ev: MouseEvent) {
        ev.preventDefault();
        ev.stopPropagation();
        const albumId = ev.currentTarget.getAttribute("data-album");
        playAlbum(albumId);
    }

    function onArtistClicked(ev: MouseEvent) {
        ev.preventDefault();
        ev.stopPropagation();
        const artist = ev.currentTarget.getAttribute("data-artist");
        playArtist(artist);
    }

    function onSongClicked(ev: MouseEvent) {
        ev.preventDefault();
        ev.stopPropagation();
        const songId = ev.currentTarget.getAttribute("data-song");
        playSong(songId);
    }

    let isLoadingMentions = true;

    interface Mention<T> {
        data?: T;
        element: HTMLAnchorElement;
    }

    /**
     * Go through all the <a> elements, get the title,
     * and check if there is an artist/song/album in the library that matches.
     *
     * If it does - add a listener to the <a> to play that song/album/artist.
     * Also update the styling of the <a> to indicate that it has been found.
     */
    async function enrichLinks() {
        isLoadingMentions = true;
        let albums: Mention<Album>[] = [];
        let songs: Mention<Song>[] = [];
        let artists: Mention<string>[] = [];

        console.log("Enrich links processing...");
        const allArtists = await db.songs.orderBy("artist").uniqueKeys();
        const allAlbums = await db.albums.toArray();
        const allAlbumTitles = allAlbums.map((album) => album.displayTitle);
        const allSongs = await db.songs.toArray();
        const allSongTitles = allSongs.map((song) => song.title);

        // We only want to enrich the links in the actual article text
        const links =
            container.querySelectorAll<HTMLAnchorElement>("a:not(.navbox a)");

        for (let i = 0; i < links.length; i++) {
            const link = links[i];
            const href = link.getAttribute("href");
            if (!href) continue;
            const title = link.textContent?.trim();
            if (!title) continue;

            // remove listeners
            link.removeEventListener("click", onArtistClicked);
            link.removeEventListener("click", onAlbumClicked);
            link.removeEventListener("click", onSongClicked);
            {
                const album = allAlbums.find((a) => {
                    // It's likely that a user can have a different edition of the same album
                    // eg. Congratulations (Japan Edition)
                    // In this case we strip the parentheses from the title
                    let parentheses = a.displayTitle?.match(/(.*)\((.*)\)/);
                    if (parentheses?.length === 3) {
                        return parentheses[1].trim() === title;
                    }
                    return a.displayTitle === title;
                });
                if (album) {
                    link.setAttribute("data-album", album.id);
                    link.closest("p")?.classList?.add("has-mention");
                    const currentSongAlbum = await db.albums
                        .where("artist")
                        .equalsIgnoreCase($currentSong.artist)
                        .and(
                            (a) =>
                                a.displayTitle.toLowerCase() ===
                                $currentSong.album.toLowerCase(),
                        )
                        .first();
                    // Add playing class or remove it
                    link.classList.toggle(
                        "playing",
                        $isPlaying && currentSongAlbum?.id === album.id,
                    );
                    link.addEventListener("click", onAlbumClicked);
                    if (!albums.find((a) => a.data.id === album.id))
                        albums.push({ data: album, element: link });
                    continue;
                }
            }

            {
                const song = allSongs.find((s) => s.title === title);
                if (song) {
                    link.setAttribute("data-song", song.id);
                    link.closest("p")?.classList?.add("has-mention");
                    // Add playing class or remove it
                    link.classList.toggle(
                        "playing",
                        $isPlaying && $currentSong?.id === song.id,
                    );
                    link.addEventListener("click", onSongClicked);
                    if (!songs.find((s) => s.data.id === song.id))
                        songs.push({ data: song, element: link });
                    continue;
                }
            }

            if (allArtists.includes(title)) {
                link.setAttribute("data-artist", title);
                link.closest("p")?.classList?.add("has-mention");
                // Add playing class or remove it
                link.classList.toggle(
                    "playing",
                    $isPlaying && $currentSong?.artist === title,
                );
                link.addEventListener("click", onArtistClicked);
                if (!artists.find((a) => a.data === title))
                    artists.push({ data: title, element: link });
            }
        }

        albumMentions = albums;
        songMentions = songs;
        artistMentions = artists;
        isLoadingMentions = false;
    }

    function scrollToMention(mention: Mention<any>) {
        mention.element.scrollIntoView({ block: "center", behavior: "smooth" });
        // Also highlight the paragraph (parent element)

        mention.element.closest("p")?.classList.add("highlighted");
        setTimeout(() => {
            mention.element.closest("p")?.classList.remove("highlighted");
        }, 1500);
        // mention.element(closes)
    }

    async function playAlbum(albumId: string) {
        const album = await db.albums.get(albumId);
        if (
            $currentSong?.album.toLowerCase() ===
            album.displayTitle.toLowerCase()
        ) {
            if ($isPlaying) {
                audioPlayer.pause();
            } else {
                audioPlayer.play(true);
            }
        } else if (album) {
            let tracks = await db.songs
                .where("id")
                .anyOf(album.tracksIds)
                .toArray();
            tracks = tracks.sort((a, b) => {
                return a.trackNumber - b.trackNumber;
            });
            if (tracks) audioPlayer.playSong(tracks[0]);
            $playlist = tracks;
            $albumPlaylist = tracks;
            $playlistType = "album";
        }
    }
    async function playArtist(artist: string) {
        if ($currentSong?.artist.toLowerCase() === artist.toLowerCase()) {
            if ($isPlaying) {
                audioPlayer.pause();
            } else {
                audioPlayer.play(true);
            }
        } else {
            const tracks = await db.songs
                .where("artist")
                .equals(artist)
                .toArray();

            if (tracks.length > 0) audioPlayer.playSong(tracks[0]);
            $playlist = tracks;
            $albumPlaylist = tracks;
            $playlistType = "album";
        }
    }

    async function playSong(songId: string) {
        const song = await db.songs.get(songId);
        if ($currentSong?.title.toLowerCase() === song?.title.toLowerCase()) {
            if ($isPlaying) {
                audioPlayer.pause();
            } else {
                audioPlayer.play(true);
            }
        } else {
            if (song) {
                audioPlayer.playSong(song);
            }
        }
    }

    function isPlayingAlbum(title: string) {
        return $currentSong?.album.toLowerCase() === title.toLowerCase();
    }

    let isScrollToTopVisible = false;
    function onScroll() {
        if (scrollContainer) {
            const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
            if (scrollTop > 200) {
                isScrollToTopVisible = true;
            } else {
                isScrollToTopVisible = false;
            }
        }
    }
</script>

<!-- Add style skin -->
<div class="container" bind:this={container}>
    {#if isScrollToTopVisible}
        <div
            class="scroll-to-top"
            transition:fly={{ duration: 150, y: 20, opacity: 0.3 }}
        >
            <ButtonWithIcon
                text="↑ Scroll to top"
                onClick={() => scrollContainer?.scrollTo(0, 0)}
            />
        </div>
    {/if}
    <div
        class="scroll-container"
        bind:this={scrollContainer}
        transition:fade={{ duration: 200 }}
        on:scroll={onScroll}
    >
        <header>
            <Icon
                icon="material-symbols:close"
                onClick={() => {
                    $isWikiOpen = false;
                }}
            />
            <div class="info-wiki">
                {#if isLoading}
                    <p transition:fade={{ duration: 200 }}>Loading...</p>
                {:else}
                    <small>Viewing wiki for: </small>
                    <p>{previousArtist}</p>
                {/if}
            </div>
            {#if previousArtist && previousArtist !== $currentSong?.artist}
                <div class="info-playing">
                    <small>Current artist: </small>
                    <ButtonWithIcon
                        size="small"
                        text="→ {$currentSong?.artist}"
                        onClick={() => getWiki($currentSong?.artist)}
                    />
                </div>
            {/if}
        </header>

        {#if isLoading}
            <div class="content">
                <p transition:fade={{ duration: 200 }}>Loading...</p>
            </div>
        {:else if wikiResult || wtfResult}
            {#if (!isLoadingMentions && albumMentions.length > 0) || songMentions.length > 0 || artistMentions.length > 0}
                <div
                    class="in-article"
                    transition:fly={{
                        duration: 300,
                        y: -20,
                        opacity: 0.4,
                    }}
                >
                    <p>
                        {$LL.wiki.inArticle()}
                        <span>{$LL.wiki.clickHint()}</span>
                    </p>
                    {#if albumMentions.length > 0}
                        <div>
                            <p>{$LL.wiki.albums()}</p>
                            <ul>
                                {#each albumMentions as album}
                                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                                    <li
                                        on:click={() => {
                                            scrollToMention(album);
                                        }}
                                    >
                                        <p>
                                            {album.data.displayTitle}
                                        </p>
                                    </li>
                                {/each}
                                <ul></ul>
                            </ul>
                        </div>
                    {/if}
                    {#if songMentions.length > 0}
                        <div>
                            <p>{$LL.wiki.songs()}</p>
                            <ul>
                                {#each songMentions as song}
                                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                                    <li
                                        on:click={() => {
                                            scrollToMention(song);
                                        }}
                                    >
                                        <p>{song.data.title}</p>
                                    </li>
                                {/each}

                                <ul></ul>
                            </ul>
                        </div>
                    {/if}

                    {#if artistMentions.length > 0}
                        <div>
                            <p>{$LL.wiki.artists()}</p>
                            <ul>
                                {#each artistMentions as artist}
                                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                                    <li
                                        role="listitem"
                                        on:click={() => {
                                            scrollToMention(artist);
                                        }}
                                    >
                                        <p>{artist.data}</p>
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    {/if}
                </div>
            {/if}
            <div class="content">
                {@html wikiResult.html}
                <!-- {#each wtfResult.sections() as section}
                        <h2>{section.title()}</h2>
                        {#each Array.isArray(section.tables()) ? Object(section.tables()) : [section.tables()] as table}
                            {#each table.keyValue() as cell}
                                <p>{JSON.stringify(cell)}</p>
                            {/each}
                        {/each}
                        {#each Array.isArray(section.lists()) ? Object(section.lists()) : [section.lists()] as list}
                            {#each list.lines() as line}
                                <p>{@html line.html()}</p>
                            {/each}
                        {/each}

                        {#each Array.isArray(section.paragraphs()) ? Object(section.paragraphs()) : [section.paragraphs()] as paragraph}
                            <br />
                            {#each Array.isArray(paragraph.sentences()) ? paragraph.sentences() : [paragraph.sentences()] as sentence}
                                {@html sentence.html()}
                            {/each}
                        {/each}
                    {/each} -->
            </div>
        {/if}
    </div>

    <ShadowGradient type="bottom" />
</div>

<style lang="scss">
    .container {
        height: 100%;
        width: 100%;
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        border-radius: 5px;
        border-left: 1px solid color-mix(in srgb, var(--bg) 70%, var(--inverse));
        border-right: 1px solid
            color-mix(in srgb, var(--bg) 70%, var(--inverse));
        margin: 5px 0 0 0;

        header {
            position: sticky;
            display: flex;
            justify-content: space-between;
            top: 0;
            padding: 1em;
            max-width: 100%;
            z-index: 10;
            backdrop-filter: blur(10px) brightness(0.95);
            border-bottom: 1px solid
                color-mix(in srgb, var(--inverse) 70%, transparent);
            background-color: var(--wiki-header-bg);

            .info-playing,
            .info-wiki {
                display: flex;
                flex-direction: column;
                justify-content: center;
                padding: 0 1em;
                flex-grow: 1;
                width: fit-content;
            }
            .info-playing {
                align-items: flex-end;
            }
            .info-wiki {
                align-items: flex-start;
            }
            small {
                line-height: initial;
                margin-bottom: 5px;
                width: fit-content;
            }
            p {
                background-color: var(--wiki-pill-bg);
                color: var(--wiki-pill-text);
                margin: 0;
                width: fit-content;
                padding: 0 5px;
                border-radius: 5px;
            }
        }

        .scroll-container {
            width: 100%;
            min-height: 100%;
            overflow-y: auto;
            pointer-events: all;
            user-select: none;
            display: grid;
            grid-template-rows: auto 1fr;
        }

        .in-article {
            background-color: var(--wiki-inarticle-bg);
            border-bottom: 1px solid
                color-mix(in srgb, var(--inverse) 40%, transparent);
            padding: 1em;

            > p {
                font-weight: normal;
                font-size: 14px;
                margin: 0 0 5px 0;
                text-align: left;
                opacity: 0.7;

                span {
                    font-size: 14px;
                    opacity: 0.5;
                }
            }
            > div {
                display: grid;
                grid-template-columns: 70px 1fr;
                align-items: flex-start;
                gap: 10px;
                p {
                    margin: 5px 0 0 0;
                    font-size: 14px;
                    color: var(--text-secondary);
                }
                ul {
                    padding: 0;
                    list-style-type: none;
                    display: flex;
                    align-items: flex-start;
                    justify-content: flex-start;
                    flex-wrap: wrap;
                    margin: 0 0 5px 0;

                    li {
                        margin: 5px 5px 0px 0;
                        padding: 2px 5px;
                        background-color: var(--wiki-pill-bg);
                        border: 1px solid
                            color-mix(in srgb, var(--inverse) 40%, transparent);
                        border-radius: 5px;
                        display: flex;
                        flex-direction: column;
                        cursor: pointer;
                        &:hover {
                            background-color: var(--wiki-pill-hover-bg);

                            p {
                                color: var(--wiki-pill-hover-text);
                            }
                        }
                        p {
                            font-size: 14px;
                            margin: 0;
                            width: max-content;
                            line-height: initial;
                            color: var(--wiki-pill-text);
                        }
                        small {
                            font-size: 12px;
                            opacity: 0.7;
                            margin: 0;
                            line-height: initial;
                        }
                    }
                }
            }
        }

        .content {
            padding: 1em;
            text-align: start;
            background-color: var(--wiki-bg);
            color: var(--text);
            max-width: 100%;

            :global(.hatnote),
            :global(.infobox),
            :global(.mw-editsection) {
                display: none;
            }
            :global(p),
            :global(span) {
                font-size: 16px;
            }

            :global([data-album]),
            :global([data-song]),
            :global([data-artist]) {
                font-weight: bold;
                border-radius: 5px;
                font-style: normal;
                background-color: color-mix(
                    in srgb,
                    var(--button-bg) 20%,
                    transparent
                );
                border: 1px solid
                    color-mix(in srgb, var(--type-bw-inverse) 40%, transparent);
                padding: 0 5px;
                color: var(--button-text);

                &:before {
                    content: "▶ ";
                    color: var(--text);
                }
            }

            :global([data-album].playing),
            :global([data-artist].playing),
            :global([data-song].playing) {
                background-color: var(--library-playing-bg);
                color: var(--library-playing-text);
                &:before {
                    content: "⏸ ";
                    color: var(--library-playing-text);
                }
            }

            :global(p.has-mention.highlighted) {
                position: relative;
                overflow: visible;
                z-index: 3;
                border-radius: 5px;
                /* border: 1px solid var(--accent); */
                animation: highlight 1.5s ease-in-out forwards;
            }

            :global(ul) {
                list-style-type: none;
                padding: 0;
                margin: 0;
            }

            @keyframes highlight {
                0% {
                    background-color: transparent;
                }
                20% {
                    background-color: color-mix(
                        in srgb,
                        var(--accent) 30%,
                        transparent
                    );
                }
                80% {
                    background-color: color-mix(
                        in srgb,
                        var(--accent) 30%,
                        transparent
                    );
                }
                100% {
                    background-color: transparent;
                }
            }

            :global(.side-box-right) {
                float: none;
            }
            :global(*) {
                max-width: 100%;
                overflow-x: auto;
            }

            // Wikipedia WTF
            :global(.sentence) {
                display: inline;
            }
        }
        .scroll-to-top {
            position: absolute;
            bottom: 0.5em;
            margin: auto;
            left: 0;
            right: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 0.5em 1em;
            border-radius: 10px;
        }
    }
</style>
