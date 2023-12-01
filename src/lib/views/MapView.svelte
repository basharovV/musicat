<script lang="ts">
    import { liveQuery } from "dexie";

    import "../../deps/jsvectormap/scss/jsvectormap.scss";
    import { world } from "../../deps/jsvectormap/maps/world.js";
    import JsVectorMap from "../../deps/jsvectormap/js";
    import DataVisualization from "../../deps/jsvectormap/js/dataVisualization";
    JsVectorMap.addMap("world", world);

    import { onMount } from "svelte";

    import "iconify-icon";
    import type { Song } from "src/App";
    import BuiltInQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import {
        isSmartQueryBuilderOpen,
        playlist,
        playlistIsAlbum,
        playlistIsCountry,
        queriedSongs,
        query,
        selectedPlaylistId,
        selectedSmartQuery,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        uiView
    } from "../../data/store";
    import SmartQuery from "../smart-query/Query";
    import { codes, countries } from "../data/CountryCodes";
    import audioPlayer from "../AudioPlayer";
    import { getFlagEmoji } from "../../utils/EmojiUtils";
    import Particles from "svelte-particles";
    import { loadSlim } from "tsparticles-slim";
    import { shuffleArray } from "../../utils/ArrayUtils";
    let isLoading = true;

    let container: HTMLElement;
    let map;
    let width = 0;
    let height = 0;
    let mapPadding = 0; //px

    $: songs = liveQuery(async () => {
        let results;
        let isSmartQueryResults = false;
        let isIndexed = true;

        if ($selectedPlaylistId !== null) {
            const playlist = await db.playlists.get($selectedPlaylistId);
            console.log("playlist to get", playlist);
            results = await db.songs.bulkGet(playlist.tracks);
            isIndexed = false;
            // Filter within playlist
            if ($query.query.length) {
                results = results.filter(
                    (song) =>
                        song.title
                            .toLowerCase()
                            .includes($query.query.toLowerCase()) ||
                        song.artist
                            .toLowerCase()
                            .includes($query.query.toLowerCase()) ||
                        song.album
                            .toLowerCase()
                            .includes($query.query.toLowerCase())
                );
            }
        } else if ($uiView === "smart-query") {
            /**
             * User-built smart queries don't support indexing
             */
            if ($isSmartQueryBuilderOpen) {
                if ($smartQuery.isEmpty) {
                    results = [];
                } else if ($smartQueryUpdater) {
                    results = await $smartQuery.run();
                }

                isSmartQueryResults = true;
                isIndexed = false;
            } else {
                /**
                 * Built-in smart queries support indexing, so they return a
                 * Collection instead of an array.
                 */
                console.log("selected query: ", $selectedSmartQuery);
                if ($selectedSmartQuery.startsWith("~usq:")) {
                    // Run the query from the user-built blocks
                    const queryName = $selectedSmartQuery.substring(5);
                    const savedQuery = await db.smartQueries.get(queryName);
                    const query = new SmartQuery(savedQuery);
                    results = await query.run();
                    console.log("results query: ", results);
                    isIndexed = false;
                } else {
                    // Run the query from built-in functions
                    results = await BuiltInQueries.find(
                        (q) => q.value === $selectedSmartQuery
                    ).query();

                    isIndexed = true;
                }
                isSmartQueryResults = true;
            }
        } else if ($query.query.length) {
            results = db.songs
                .orderBy(
                    $query.orderBy === "artist"
                        ? "[artist+year+album+trackNumber]"
                        : $query.orderBy === "album"
                          ? "[album+trackNumber]"
                          : $query.orderBy
                )
                .and(
                    (song) =>
                        song.title
                            .toLowerCase()
                            .startsWith($query.query.toLowerCase()) ||
                        song.artist
                            .toLowerCase()
                            .startsWith($query.query.toLowerCase()) ||
                        song.album
                            .toLowerCase()
                            .startsWith($query.query.toLowerCase())
                );
        } else {
            results = db.songs.orderBy(
                $query.orderBy === "artist"
                    ? "[artist+year+album+trackNumber]"
                    : $query.orderBy === "album"
                      ? "[album+trackNumber]"
                      : $query.orderBy
            );
        }
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

        /**
         * Set in store
         */
        if (isSmartQueryResults) {
            smartQueryResults.set(resultsArray);
        } else {
            queriedSongs.set(resultsArray);
        }

        isLoading = false;
        $playlist = resultsArray;
        $playlistIsAlbum = false;
        return resultsArray;
    });

    let data = null;
    let dataCountMap = null;

    let selectedCountry = null;
    let selectedCountryPos = null;
    let dataSetCountryValue = null;

    $: {
        if ($songs !== null) {
            updateData();
        }

        if (data) {
            dataCountMap = data
                ? Object.entries(data).reduce(
                      (dataSetObj, currentCountry: object) => {
                          dataSetObj[codes[currentCountry[0]]] =
                              currentCountry[1].data.length;
                          return dataSetObj;
                      },
                      {}
                  )
                : [];

            if (window && map) {
                map.dataVisualization = new DataVisualization(
                    {
                        scale: ["#9070BB", "#984EFF"],
                        values: dataCountMap ? dataCountMap : []
                    },
                    map
                );
                map.updateSize();
                onResize();

                $playlistIsCountry && setSelectedCountry($playlistIsCountry);
            }
        }
        console.log("countMap", dataCountMap);
    }

    function setSelectedCountry(countryCode) {
        // Get path
        // map._clearSelected('regions');
        map.setSelectedRegions([countryCode]);
        // const allPaths = document.querySelectorAll("[data-code]");
        // allPaths.forEach((path) => {
        //     path.style.animation = null;
        // });
        const path: SVGPathElement = document.querySelector(
            `[data-code=${countryCode}]`
        );
        // path.style.animation =
        //     "playing-outer 2s ease-in-out infinite alternate-reverse";
        path.classList.add("country-playing");
    }

    let nOfCountries = 0;

    const groupBy = function (xs, key) {
        return xs.reduce(function (rv, x) {
            if (rv[x[key]] === undefined) {
                rv[x[key]] = {
                    data: []
                };
            }
            (rv[x[key]].data = rv[x[key]].data || []).push(x);
            return rv;
        }, {});
    };

    async function updateData() {
        if ($songs) {
            data = groupBy($songs, "originCountry");
            delete data.undefined;
            nOfCountries = data.length;
            console.log("data", data);
        }
    }

    function resetMap() {
        map = new JsVectorMap({
            onLoaded(map) {
                onResize();
            },
            draggable: true,
            zoomButtons: false,
            zoomOnScroll: true,
            zoomOnScrollSpeed: 0.3,
            zoomMax: 5,
            zoomMin: 1,
            zoomAnimate: true,
            zoomStep: 1.5,
            selector: "#map",
            showTooltip: true,
            visualizeData: {
                scale: ["#eeeeee", "#999999"],
                values: dataCountMap ? dataCountMap : []
            },
            // Play country
            onRegionClick(event, code) {
                if (!data) return;
                console.log("clicked", code);
                if (data[countries[code]]) {
                    selectedCountry = code;
                    selectedCountryPos = {
                        x: event.pageX - 11 - mapPadding * 2,
                        y: event.pageY
                    };
                    dataSetCountryValue = data[countries[code]] || null;
                    onCountryClicked();
                }
            },
            onRegionTooltipShow(event, tooltip, code) {
                if (!data) return;
                if (data[countries[code]]) {
                    const countryValue = data[countries[code]] || null;

                    tooltip.text(
                        `<h5 style="margin:0">${tooltip.text()}</h5>
                        
                        <p>${countryValue.data.length} artists</p>`,
                        true // Enables HTML
                    );
                }
            },
            map: "world",
            regionsSelectable: false,
            regionsSelectableOne: true,
            regionStyle: {
                initial: {
                    fill: "#645479",
                    stroke: "#4F4464",
                    strokeWidth: 1,
                    fillOpacity: 1
                },
                selected: {
                    fill: "#59CD70",
                    stroke: "#4F4464",
                    strokeWidth: 1,
                    fillOpacity: 1
                }
                // hover: { fill: "#E9E6EE" }
            }
        });
    }

    onMount(async () => {
        onResize();
        resetMap();
    });

    function onResize() {
        height = container.offsetHeight;
        width = container.clientWidth - 10;
        if (map) {
            map.height = height;
            map.width = width;
            map.reset();
        }
    }

    function onCountryClicked() {
        $playlist = dataSetCountryValue.data;
        console.log(dataSetCountryValue.data[0]);
        audioPlayer.playSong(dataSetCountryValue.data[0]);
        $playlistIsCountry = selectedCountry;
    }

    $: numberOfTracks = $playlist.length;
    $: numberOfArtists = [...new Set($playlist.map((item) => item.artist))]
        .length;

    // Particles

    let particlesOptions = {
        preset: "stars",
        fullScreen: false,
        background: {
            color: "transparent"
        }
    };

    let particlesInit = async (engine) => {
        const options = {
            particles: {
                number: {
                    value: 100
                },
                move: {
                    bounce: true,
                    direction: "none",
                    enable: true,
                    outModes: {
                        default: "out"
                    },
                    random: true,
                    speed: 0.4,
                    straight: false
                },
                opacity: {
                    animation: {
                        enable: true,
                        speed: 1,
                        sync: false
                    },
                    value: { min: 0, max: 0.3 }
                },
                size: {
                    value: { min: 1, max: 3 }
                }
            }
        };

        await loadSlim(engine, false);
        await engine.addPreset("stars", options, true);
    };
</script>

<svelte:window on:resize={onResize} />

<container bind:this={container}>
    <div class="bg" />

    <Particles id="tsparticles" options={particlesOptions} {particlesInit} />
    <content>
        {#if $playlistIsCountry}
            <div class="header">
                <div class="options">
                    <div class="shuffle">
                        <iconify-icon icon="ph:shuffle-bold" />
                        <div>Shuffle</div>
                    </div>
                </div>
                <div id="info">
                    <p>Listening to music from</p>
                    <h2>
                        {getFlagEmoji($playlistIsCountry)}{countries[
                            $playlistIsCountry
                        ]}
                    </h2>
                    <small
                        >{numberOfTracks} tracks from {numberOfArtists} artists</small
                    >
                </div>
            </div>
        {/if}
    </content>
    {#if songs}
        <div class="map-container">
            <div id="map" style="width: {width}px; height: {height}px" />
            <!-- {#if selectedCountryPos}
                <svg
                    class="now-playing"
                    {width}
                    {height}
                    viewBox="0 0 {width} {height}"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <circle
                        class="now-playing-inner"
                        cx={selectedCountryPos.x}
                        cy={selectedCountryPos.y}
                        style="transform-origin: {selectedCountryPos.x}px {selectedCountryPos.y}px;"
                        r="11"
                        fill="#2CFAA0"
                    />
                    <circle
                        class="now-playing-outer"
                        cx={selectedCountryPos.x}
                        cy={selectedCountryPos.y}
                        style="transform-origin: {selectedCountryPos.x}px {selectedCountryPos.y}px;"
                        r="20"
                        fill="#70B67E"
                    />
                </svg>
            {/if} -->
        </div>
    {/if}
</container>

<style lang="scss">
    container {
        height: 100vh;
        width: 100%;
        position: relative;
    }

    .bg {
        position: absolute;
        top: 0;
        bottom: 0;
        right: 0;
        left: 0;
        z-index: -1;
        height: 100%;
        width: 100%;
    }

    .now-playing {
        z-index: 2;
        position: absolute;
        top: 0;
        bottom: 0;
        right: 0;
        left: 0;
        pointer-events: none;
        .now-playing-inner {
            animation: playing-inner 2s ease-in-out infinite alternate-reverse;
        }
        .now-playing-outer {
            animation: playing-outer 2s ease-in-out infinite alternate-reverse;
        }
    }

    :global(.country-playing) {
        transform-origin: center;
        paint-order: stroke;
        stroke: rgba(255, 255, 255, 0.1);
        /* animation: playing-shake 2s ease-in-out infinite alternate-reverse; */
    }

    @keyframes playing-inner {
        0% {
            transform: scale(1);
            opacity: 0.2;
        }
        100% {
            transform: scale(2);
            opacity: 0.5;
        }
    }

    @keyframes playing-outer {
        0% {
            transform: scale(1);
            opacity: 0.2;
        }
        100% {
            transform: scale(1.4);
            opacity: 0.5;
        }
    }

    @keyframes playing-shake {
        0% {
            stroke-width: 0px;
        }
        100% {
            stroke-width: 10px;
        }
    }
    @keyframes dash {
        to {
            stroke-dashoffset: 0;
        }
    }
    .map-container {
    }

    content {
        position: absolute;
        width: 100%;
        height: auto;
        top: 0;
        left: 0;
        right: 0;
        z-index: 2;
    }

    .header {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        align-items: center;
        justify-content: center;
        margin-top: 2em;
        .options {
            p {
                margin: 0;
            }

            .shuffle {
                display: flex;
                width: fit-content;
                margin: auto;
                padding: 0.5em 2em;
                border-radius: 10px;
                border: 1px solid rgba(255, 255, 255, 0.113);
                background: rgba(128, 128, 128, 0.105);
                gap: 5px;
                flex-direction: row;
                align-items: center;
                cursor: default;
                &:hover {
                    opacity: 0.6;
                }
            }
        }
    }

    #dropdown {
        display: block;
        width: fit-content;
        margin: 2em auto;
        color: rgb(32, 31, 30);
        h2 {
            text-align: center;
        }
        select {
            font-size: 1em;
            color: rgb(50, 48, 48);
        }
    }

    #map {
        overflow: hidden;
        margin: auto;
        display: flex;
        height: fit-content;
    }

    #info {
        margin: auto;
        width: fit-content;
        display: flex;
        align-items: center;
        flex-direction: column;
        small,
        p {
            opacity: 0.5;
        }
        * {
            margin: 0;
        }
    }

    :global(#tsparticles) {
        // This is the id of the div created by the Particles component
        height: 100vh;
        position: absolute;
        left: 0;
        right: 0;
        margin: 0;
        padding: 0;
        z-index: 0;
    }
</style>
