<script lang="ts">
    import { liveQuery } from "dexie";
    import { db } from "../../data/db";
    import { isSettingsOpen, query, userSettings } from "../../data/store";
    import type { Album, Song } from "../../App";
    import ProgressBar from "../ui/ProgressBar.svelte";
    import { groupBy } from "../../utils/ArrayUtils";
    import OpenAI from "openai";
    import Icon from "../ui/Icon.svelte";
    import { Body, fetch } from "@tauri-apps/api/http";
    import AlbumsTimeline from "../ui/AlbumsTimeline.svelte";
    import { fade } from "svelte/transition";
    import { cubicInOut } from "svelte/easing";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    // Analytics for your library
    // Including play metrics, GPT summaries
    let isLoading = false;

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;
        results = db.songs.orderBy(
            $query.orderBy === "artist"
                ? "[artist+year+album+trackNumber]"
                : $query.orderBy === "album"
                  ? "[album+trackNumber]"
                  : $query.orderBy
        );

        let resultsArray: Song[] = [];

        // Depending whether this is a smart query or not
        if (isIndexed) {
            if ($query.reverse) {
                results = results.reverse();
            }
            resultsArray = await results.toArray();
        } else {
            resultsArray = results;
        }

        // Do sorting for non-indexed results
        if (!isIndexed) {
            resultsArray = resultsArray.sort((a, b) => {
                switch ($query.orderBy) {
                    case "title":
                    case "album":
                    case "track":
                    case "year":
                    case "duration":
                    case "genre":
                        return a[$query.orderBy].localeCompare(
                            b[$query.orderBy]
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
            playCount: 0
        },
        country: {
            count: 0,
            mostPlayed: null,
            playCount: null
        }
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
                        song.duration.split(":")[0]
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
                        playCount: 0
                    },
                    country: {
                        count: 0,
                        mostPlayed: null,
                        playCount: 0
                    }
                }
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
                        g[1].data.length > mostPlayedCountry.length
                    ) {
                        mostPlayedCountry = g[1].data;
                        mostPlayedCountryTitle = g[0];
                    }
                });
            stats.country.mostPlayed = mostPlayedCountryTitle;
            stats.country.playCount = mostPlayedCountry.length;
            stats.country.count = Object.keys(groupedByCountry).length;

            // Derived stats
            stats.percentageListened =
                (stats.distinctPlays / $songs.length) * 100;

            console.log("in here");
            statsLoaded = true;

            if ($userSettings.aiFeaturesEnabled) {
                if (!gptSummary && !gptSummaryLoading) {
                    gptSummaryLoading = true;
                    getGptSummary();
                }
                if (!gptDidYouKnow && !gptDidYouKnowLoading) {
                    gptDidYouKnowLoading = true;
                    getGptDidYouKnow();
                }
                if (!singleWordSummary && !singleWordSummaryLoading) {
                    singleWordSummaryLoading = true;
                    getSingleWordSummary();
                }
            }
        }
    }

    let openai: OpenAI;

    let gptSummaryLoading = false;
    let gptSummary = null;
    let singleWordSummary = null;
    let singleWordSummaryLoading = false;
    let gptDidYouKnowLoading = false;
    let gptDidYouKnow = null;

    async function getOllamaResponse(prompt: string): Promise<string> {
        try {
            const response = await fetch(
                "http://localhost:11434/api/generate",
                {
                    method: "POST",
                    body: Body.json({
                        "model": "mistral",
                        "prompt": prompt,
                        "stream": false
                    })
                }
            );
            const json = await response;
            console.log(json);
            return json.data.response;
        } catch (err) {
            return "No response from ollama. Is the model running?";
        }
    }

    async function getChatGPTResponse(prompt: string) {
        if (!openai) {
            if (!$userSettings.openAIApiKey) {
                // Show error
                return "API Key needed";
            }
            openai = new OpenAI({
                apiKey: $userSettings.openAIApiKey,
                dangerouslyAllowBrowser: true, // This is the default and can be omitted
            });
        }
        const params: OpenAI.Chat.ChatCompletionCreateParams = {
            messages: [
                {
                    role: "user",
                    content:
                        "You are an assistant in a desktop music player app, helping the user discover and learn more about their music library.\n\n" +
                        prompt
                }
            ],
            model: $userSettings.llm,
            temperature: 1.08,
            max_tokens: 256,
            top_p: 1,
            frequency_penalty: 0.04,
            presence_penalty: 0.04
        };
        const chatCompletion: OpenAI.Chat.ChatCompletion =
            await openai.chat.completions.create(params);
        return chatCompletion.choices[0].message.content;
    }

    async function getModelResponse(prompt) {
        console.log($userSettings.llm)
        switch ($userSettings.llm) {
            case "gpt-3.5-turbo":
            case "gpt-4":
                return await getChatGPTResponse(prompt);
            case "ollama":
                return await getOllamaResponse(prompt);
            default:
                return await getOllamaResponse(prompt);
        }
    }

    async function getSingleWordSummary() {
        // Artists
        let groupedByArtists = groupBy($songs, "artist");
        let listOfArtists = Object.keys(groupedByArtists);

        let prompt1 = `You are a helpful assistant that gives short, concise analysis of music libraries. List 3 words that describe the mood of music based on the following list of artists: \n${listOfArtists.join(
            ", "
        )}. Provide your answer as a single Javascript array, like this: ["a", "b", "c"]. Don't mention or include artist names in your response. I don't need any further analysis or explanations/comments.`;
        const response = await getModelResponse(prompt1);
        if (Array.isArray(response)) {
            singleWordSummary = Array.isArray(JSON.parse(response))
                ? JSON.parse(response).join(", ")
                : "";
        } else {
            singleWordSummary = response;
        }
    }
    async function getGptSummary() {
        // Artists
        let groupedByArtists = groupBy($songs, "artist");
        let listOfArtists = Object.keys(groupedByArtists);

        let prompt1 = `Can you provide a brief (50 word) analysis overview of my music library based on the following list of artists: \n${listOfArtists.join(
            ", "
        )}. Don't mention the artists names in your response. Address me in 2nd person. Whenever you mention an artist name in your response, wrap it in a <span> tag.`;
        gptSummary = await getModelResponse(prompt1);
    }

    async function getGptDidYouKnow() {
        // Artists
        let groupedByArtists = groupBy($songs, "artist");
        let listOfArtists = Object.keys(groupedByArtists);

        let prompt2 = `Can you provide a 'Did you know?' fact that links artists from this list: \n\n${listOfArtists.join(
            ", "
        )}. \n\nUse only artists in the list provided. This is very important. I'm specifically interested in how their musical careers are related or intertwined. Keep it short (50 words). Don't include a vague conclusion, just the fact itself and some details. Whenever you mention an artist, wrap it in a <span> tag.`;
        gptDidYouKnow = await getModelResponse(prompt2);
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
                <h2><span>{stats.totalDurationMins / 60} hours</span></h2>
                <p>
                    You have listened to {stats.percentageListened.toPrecision(
                        2
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
            </section>
            <section class="timeline">
                <AlbumsTimeline {albumsByYear} />
            </section>
            {#if $userSettings.aiFeaturesEnabled}
                <section
                    class="short-summary"
                    class:loading={!singleWordSummary &&
                        singleWordSummaryLoading}
                >
                    <h3>
                        <iconify-icon icon="mdi:thunder"></iconify-icon>The
                        sentiment of your library is
                    </h3>
                    <h2 id="gpt-summary">
                        {@html singleWordSummary
                            ? singleWordSummary
                            : "asking AI ..."}
                    </h2>
                </section>
                <section
                    class="summary"
                    class:loading={!gptSummary && gptSummaryLoading}
                >
                    <h3>
                        <iconify-icon icon="mdi:thunder"></iconify-icon>GPT
                        summary of your library
                    </h3>
                    <h2 id="gpt-summary">
                        {@html gptSummary ? gptSummary : "one moment ..."}
                    </h2>
                </section>
                <section
                    class="did-you-know"
                    class:loading={!gptDidYouKnow && gptDidYouKnowLoading}
                >
                    <h3>
                        <iconify-icon icon="mdi:thunder"></iconify-icon>Did you
                        know?
                    </h3>
                    <h2 id="gpt-summary">
                        {@html gptDidYouKnow ? gptDidYouKnow : "thinking ..."}
                    </h2>
                </section>
            {:else}
                <section style="grid-column: 1 / 7">
                    <ButtonWithIcon
                        icon="bi:robot"
                        text="Enable AI analysis"
                        onClick={() => { $isSettingsOpen = true}}
                    />
                    <p>
                        Get sentiment analysis, a summary of your music taste,
                        fun facts about artists.
                    </p>
                    <small
                        >Works with OpenAI (requires setting API Key in
                        settings), or local Ollama model.</small
                    >
                </section>
            {/if}
        </div>
    {/if}
</div>

<style lang="scss">
    .container {
        overflow: auto;
        height: 100%;
        width: 100%;
        display: flex;
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
        /* margin: 4em; */
        width: 100%;
        /* height: fit-content;
        border: 1px solid rgba(255, 255, 255, 0.04);
        border-radius: 15px; */
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr 1fr 1fr;
        grid-template-rows: auto auto 1fr;

        section {
            padding: 4em;
            /* background-color: rgb(203, 191, 240); */
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
                border-right: 1px solid rgba(255, 255, 255, 0.11);
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
            }
            &.genres {
                grid-row: 1;
                grid-column: 3 / 5;
                border-right: 1px solid rgba(255, 255, 255, 0.11);
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
                span {
                    color: #855dff;
                }
            }
            &.country {
                grid-row: 1;
                grid-column: 5 / 7;
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
                span {
                    color: #23dd7f;
                }
            }
            &.timeline {
                padding: 0 2em;
                grid-row: 2;
                grid-column: 1 / 7;
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
                span {
                    color: #23dd7f;
                }
            }
            &.short-summary {
                padding: 2em;
                grid-row: 3;
                grid-column: 1 / 7;
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
                align-items: center;
                justify-content: flex-start;
                h3 {
                    color: #23dd7f;
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
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
                border-right: 1px solid rgba(255, 255, 255, 0.11);
                align-items: flex-start;
                justify-content: flex-start;
                h3 {
                    color: #23dd7f;
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
                border-bottom: 1px solid rgba(255, 255, 255, 0.11);
                align-items: flex-start;
                justify-content: flex-start;
                h3 {
                    color: #23dd7f;

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
