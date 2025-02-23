<script lang="ts">
    import { liveQuery } from "dexie";
    import { db } from "../../data/db";
    import { uiView } from "../../data/store";
    import type { Album, Song, SongOrder } from "../../App";
    import ProgressBar from "../ui/ProgressBar.svelte";
    import { groupBy } from "../../utils/ArrayUtils";
    import AlbumsTimeline from "../ui/AlbumsTimeline.svelte";
    import { fade } from "svelte/transition";
    import { cubicInOut } from "svelte/easing";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    export let songOrder: SongOrder;

    // Analytics for your library
    // Including play metrics, GPT summaries
    let isLoading = false;

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;
        results = db.songs.orderBy(
            songOrder.orderBy === "artist"
                ? "[artist+year+album+trackNumber]"
                : songOrder.orderBy === "album"
                  ? "[album+trackNumber]"
                  : songOrder.orderBy,
        );

        let resultsArray: Song[] = [];

        // Depending whether this is a smart query or not
        if (isIndexed) {
            if (songOrder.reverse) {
                results = results.reverse();
            }
            resultsArray = await results.toArray();
        } else {
            resultsArray = results;
        }

        // Do sorting for non-indexed results
        if (!isIndexed) {
            resultsArray = resultsArray.sort((a, b) => {
                switch (songOrder.orderBy) {
                    case "title":
                    case "album":
                    case "track":
                    case "year":
                    case "duration":
                    case "genre":
                        return a[songOrder.orderBy].localeCompare(
                            b[songOrder.orderBy],
                        );
                    case "artist":
                        // TODO this one needs to match the multiple indexes sorting from Dexie
                        // i.e Artist -> Album -> Track N.
                        return a.artist.localeCompare(b.artist);
                }
            });
        }

        isLoading = false;
        return resultsArray;
    });

    $: albums = liveQuery(async () => {
        let resultsArray: Album[] = [];
        resultsArray = await db.albums.toArray();

        isLoading = false;
        return resultsArray;
    });

    let stats = {
        totalSongs: 0,
        totalPlays: 0,
        distinctPlays: 0,
        percentageListened: 0,
        totalDurationMins: 0,
        genres: {
            count: 0,
            mostPlayed: null,
            playCount: 0,
        },
        country: {
            count: 0,
            mostPlayed: null,
            playCount: null,
        },
    };

    let albumsByYear = {};
    const sortObject = (o) =>
        Object.keys(o)
            .sort()
            .reduce((r, k) => ((r[k] = o[k]), r), {});

    $: {
        if ($albums) {
            albumsByYear = groupBy($albums, "year");
            albumsByYear = sortObject(albumsByYear);
        }
    }

    let statsLoaded = false;

    $: {
        if ($songs) {
            stats = $songs.reduce(
                (stats, song: Song) => {
                    stats.totalPlays = !song.playCount
                        ? stats.totalPlays
                        : stats.totalPlays + song.playCount;
                    stats.distinctPlays = !song.playCount
                        ? stats.distinctPlays
                        : stats.distinctPlays + 1;
                    stats.totalDurationMins += Number(
                        song.duration.split(":")[0],
                    );
                    return stats;
                },
                {
                    totalSongs: $songs.length,
                    totalPlays: 0,
                    distinctPlays: 0,
                    percentageListened: 0,
                    totalDurationMins: 0,
                    genres: {
                        count: 0,
                        mostPlayed: null,
                        playCount: 0,
                    },
                    country: {
                        count: 0,
                        mostPlayed: null,
                        playCount: 0,
                    },
                },
            );

            // Genre
            let groupedByGenre = groupBy($songs, "genre");
            let mostPlayedGenreTitle = null;
            let mostPlayedGenre = null;
            Object.entries(groupedByGenre).forEach((g: [string, any]) => {
                if (
                    !mostPlayedGenre ||
                    g[1].data.length > mostPlayedGenre.length
                ) {
                    mostPlayedGenre = g[1].data;
                    mostPlayedGenreTitle = g[0];
                }
            });
            stats.genres.mostPlayed = mostPlayedGenreTitle;
            stats.genres.playCount = mostPlayedGenre.length;
            stats.genres.count = Object.keys(groupedByGenre).length;

            // Country
            let groupedByCountry = groupBy($songs, "originCountry");
            let mostPlayedCountryTitle = null;
            let mostPlayedCountry = null;
            Object.entries(groupedByCountry)
                .filter((c) => c[0] !== "undefined")
                .forEach((g: [string, any]) => {
                    if (
                        !mostPlayedCountry ||
                        g[1].data.length > mostPlayedCountry?.length
                    ) {
                        mostPlayedCountry = g[1].data;
                        mostPlayedCountryTitle = g[0];
                    }
                });
            stats.country.mostPlayed = mostPlayedCountryTitle;
            stats.country.playCount = mostPlayedCountry?.length;
            stats.country.count = Object.keys(groupedByCountry).length;

            // Derived stats
            stats.percentageListened =
                (stats.distinctPlays / $songs.length) * 100;

            console.log("in here");
            statsLoaded = true;
        }
    }
</script>

<div class="container">
    {#if !statsLoaded}
        <div class="placeholder" in:fade={{ duration: 90, easing: cubicInOut }}>
            <p>💿 one sec...</p>
        </div>
    {:else}
        <div class="analytics" in:fade={{ duration: 500 }}>
            <section class="plays">
                <h1><span>{stats.totalSongs}</span> songs</h1>
                <h2>
                    <span
                        >{(stats.totalDurationMins / 60).toFixed(2)} hours</span
                    >
                </h2>
                <p>
                    You have listened to {stats.percentageListened.toPrecision(
                        2,
                    )}% of your library ({stats.distinctPlays} / {$songs?.length}
                    songs)
                </p>

                <ProgressBar percent={stats.percentageListened} />
            </section>

            <section class="genres">
                <h1>
                    <span>{stats.genres.count} </span>genres
                </h1>
                <h2>
                    The genre dominating your library is <span
                        >{stats.genres.mostPlayed}</span
                    >
                </h2>
                <p>
                    from {stats.genres.playCount} tracks
                </p>
            </section>

            <section class="country">
                {#if stats.country.mostPlayed && stats.country.count}
                    <h1>
                        <span>{stats.country.count} </span>countries
                    </h1>
                    <h2>
                        You have more music from <span
                            >{stats.country.mostPlayed}</span
                        >
                        than any other country
                    </h2>

                    <p>
                        from {stats.country.playCount} tracks
                    </p>
                {:else}
                    <p>No country data in your library</p>
                    <br />
                    <ButtonWithIcon
                        text="Add country data"
                        onClick={() => ($uiView = "map")}
                    />
                {/if}
            </section>
            <section class="timeline">
                <AlbumsTimeline {albumsByYear} />
            </section>
        </div>
    {/if}
</div>

<style lang="scss">
    .container {
        overflow: auto;
        display: flex;
        border: 0.7px solid var(--panel-primary-border-main);
        margin: 5px 5px 0 0;
        border-radius: 5px;
        overflow: hidden;
        background-color: var(--panel-background);
    }

    .placeholder {
        margin: auto;
        p {
            opacity: 0.6;
        }
    }

    .loading {
        h2 {
            opacity: 0.5;
            animation: fade 1.2s ease-in-out infinite alternate-reverse;
        }
    }

    @keyframes fade {
        from {
            opacity: 0.7;
        }
        to {
            opacity: 0.2;
        }
    }
    .analytics {
        width: 100%;
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr 1fr 1fr;
        grid-template-rows: auto auto 1fr;

        section {
            padding: 4em;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;

            h2 {
                margin: 0;
            }

            h1 {
                margin: 0 0 0.5em 0;
                font-size: 2.5em;
            }
            h2,
            h1 {
                max-width: 290px;
            }

            p {
                margin: 1em 0 0 0;
                opacity: 0.5;
            }

            &.plays {
                grid-row: 1;
                grid-column: 1 / 3;
                border-right: 1px solid var(--analytics-border);
                border-bottom: 1px solid var(--analytics-border);
            }
            &.genres {
                grid-row: 1;
                grid-column: 3 / 5;
                border-right: 1px solid var(--analytics-border);
                border-bottom: 1px solid var(--analytics-border);
                span {
                    color: var(--analytics-text-secondary);
                }
            }
            &.country {
                grid-row: 1;
                grid-column: 5 / 7;
                border-bottom: 1px solid var(--analytics-border);
                span {
                    color: var(--analytics-text-primary);
                }
            }
            &.timeline {
                padding: 0 2em;
                grid-row: 2;
                grid-column: 1 / 7;
                border-bottom: 1px solid var(--analytics-border);
                span {
                    color: var(--analytics-text-primary);
                }
            }
            &.short-summary {
                padding: 2em;
                grid-row: 3;
                grid-column: 1 / 7;
                border-bottom: 1px solid var(--analytics-border);
                align-items: center;
                justify-content: flex-start;
                h3 {
                    color: var(--analytics-text-primary);
                    font-weight: 300;
                    text-align: left;
                    margin: 0;
                }
                h2 {
                    max-width: none;
                    line-height: 1.5em;
                    text-align: left;
                    font-weight: 400;
                    font-size: 1.7em;
                }
            }
            &.summary {
                grid-row: 4;
                grid-column: 1 / 4;
                border-bottom: 1px solid var(--analytics-border);
                border-right: 1px solid var(--analytics-border);
                align-items: flex-start;
                justify-content: flex-start;
                h3 {
                    color: var(--analytics-text-primary);
                    font-weight: 300;
                    text-align: left;
                }
                h2 {
                    max-width: none;
                    line-height: 1.5em;
                    text-align: left;
                    font-weight: 400;
                    font-size: 1.3em;
                    :global(span) {
                        border-radius: 4px;
                        border-bottom: 1px solid rgba(128, 128, 128, 0.514);
                        padding: 2px 4px;
                        font-weight: 500;
                        &:hover {
                            border-bottom: 1px solid white;
                            cursor: pointer;
                        }
                    }
                }
            }
            &.did-you-know {
                grid-row: 4;
                grid-column: 4 / 7;
                border-bottom: 1px solid var(--analytics-border);
                align-items: flex-start;
                justify-content: flex-start;
                h3 {
                    color: var(--analytics-text-primary);

                    font-weight: 300;
                    text-align: left;
                }
                h2 {
                    max-width: none;
                    line-height: 1.5em;
                    text-align: left;
                    font-weight: 400;
                    font-size: 1.3em;

                    :global(span) {
                        border-radius: 4px;
                        border-bottom: 1px solid rgba(128, 128, 128, 0.514);
                        padding: 2px 4px;
                        font-weight: 500;
                        &:hover {
                            border-bottom: 1px solid white;
                            cursor: pointer;
                        }
                    }
                }
            }
        }
    }
</style>
