<script lang="ts">
    import { liveQuery } from "dexie";

    import "../../deps/jsvectormap/scss/jsvectormap.scss";
    import { world } from "../../deps/jsvectormap/maps/world.js";
    import JsVectorMap from "../../deps/jsvectormap/js";
    import DataVisualization from "../../deps/jsvectormap/js/dataVisualization";
    JsVectorMap.addMap("world", world);

    import { onMount } from "svelte";

    import type { MapTooltipData, Song } from "src/App";
    import BuiltInQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import {
        addOriginCountryStatus,
        currentSong,
        isSmartQueryBuilderOpen,
        playlist,
        playlistIsAlbum,
        playlistCountry,
        queriedSongs,
        query,
        selectedPlaylistFile,
        selectedSmartQuery,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        uiView,
        playlistType,
        currentSongIdx,
        isShuffleEnabled,
        isQueueOpen
    } from "../../data/store";
    import SmartQuery from "../smart-query/Query";
    import { codes, countries } from "../data/CountryCodes";
    import audioPlayer from "../player/AudioPlayer";
    import { getFlagEmoji } from "../../utils/EmojiUtils";
    import Particles from "svelte-particles";
    import { loadSlim } from "tsparticles-slim";
    import { groupBy, shuffleArray } from "../../utils/ArrayUtils";
    import MapTooltip from "../map/MapTooltip.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import {
        addCountryDataAllSongs,
        findCountryByArtist
    } from "../data/LibraryEnrichers";
    import ProgressBar from "../ui/ProgressBar.svelte";
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

        if ($selectedPlaylistFile !== null) {
            const playlist = await db.playlists.get($selectedPlaylistFile);
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
                    const queryId = Number($selectedSmartQuery.substring(5));
                    const savedQuery = await db.smartQueries.get(queryId);
                    const query = new SmartQuery(savedQuery);
                    results = await query.run();
                    console.log("results query: ", results);
                    isIndexed = false;
                } else {
                    // Run the query from built-in functions
                    results = await BuiltInQueries[$selectedSmartQuery].query();

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
        return resultsArray;
    });

    let data = null;
    let dataCountMap = null;

    let selectedCountry = null;
    let selectedCountryPos = null;
    let dataSetCountryValue = null;
    let initialized = false;

    // Tooltip

    let tooltipData: MapTooltipData = null;

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
                if (!initialized) {
                    map.updateSize();
                    onResize();
                    initialized = true;
                }

                $playlistCountry && setSelectedCountry($playlistCountry);
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
        let width = path.getBBox().width;
        let height = path.getBBox().height;
        let x = path.getBBox().x + width / 2;
        let y = path.getBBox().y + height / 2;

        console.log("xy", x, y);

        if (particleContainer) {
            console.log(particleContainer);
            // particleContainer._engine.actualOptions.particles.move.center.radius = 100;
            // particleContainer._engine.actualOptions.particles.move.center.x =
            //     x;
            // particleContainer._engine.actualOptions.particles.move.center.y =
            //     y;
            particleContainer._engine.plugins.presets.get(
                "stars"
            ).particles.move.center.x = x;
            particleContainer._engine.plugins.presets.get(
                "stars"
            ).particles.move.center.y = y;
            particleContainer._engine.load("stars");
        }

        console.log("x", x, "y", y);
        // path.style.animation =
        //     "playing-outer 2s ease-in-out infinite alternate-reverse";
        path.classList.add("country-playing");
    }

    let nOfCountries = 0;

    async function updateData() {
        if ($songs) {
            data = groupBy($songs, "originCountry");
            delete data.undefined;
            delete data.null;
            delete data[""];
            nOfCountries = Object.keys(data)?.length ?? 0;
        }
    }

    function resetMap() {
        map = new JsVectorMap({
            onLoaded(map) {
                map.updateSize();
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
                    // countryValue.data.length  = number artists
                    console.log("tooltip", tooltip);

                    // Hovered country

                    let hoveredCountryPlaylist: Song[] = countryValue.data;
                    let hoveredCountryArtists: string[] = [
                        ...new Set(
                            hoveredCountryPlaylist.map((item) => item.artist)
                        )
                    ];
                    let hoveredCountryNumArtists = hoveredCountryArtists.length;
                    let hoveredCountryFirstFewArtists =
                        hoveredCountryArtists.slice(
                            0,
                            Math.min(3, artists.length)
                        );

                    let hoveredCountryFirstFewAlbums: {
                        path: string;
                        artist: string;
                        album: string;
                    }[] = hoveredCountryPlaylist.map((item) => ({
                        path: item.path.replace(`/${item.file}`, ""),
                        artist: item.artist,
                        album: item.album
                    }));

                    // distinct albums
                    hoveredCountryFirstFewAlbums =
                        hoveredCountryFirstFewAlbums.filter(
                            (e, i) =>
                                hoveredCountryFirstFewAlbums.findIndex(
                                    (a) => a["album"] === e["album"]
                                ) === i
                        );

                    tooltipData = {
                        countryName: countries[code],
                        emoji: getFlagEmoji(code),
                        numberOfArtists: hoveredCountryNumArtists,
                        artists: hoveredCountryFirstFewArtists,
                        albums: hoveredCountryFirstFewAlbums
                    };
                } else {
                    tooltipData = {
                        countryName: countries[code],
                        emoji: getFlagEmoji(code),
                        numberOfArtists: 0,
                        artists: [],
                        albums: []
                    };
                }
            },
            map: "world",
            regionsSelectable: false,
            regionsSelectableOne: true,
            regionStyle: {
                initial: {
                    fill: "#645479",
                    stroke: "#4F4464",
                    strokeWidth: 0.5,
                    fillOpacity: 1
                },
                selected: {
                    fill: "#59CD70",
                    stroke: "#4F4464",
                    strokeWidth: 1,
                    fillOpacity: 1
                },
                selectedHover: {
                    fill: "#59CD70",
                    stroke: "#4F4464",
                    strokeWidth: 1,
                    fillOpacity: 1
                },
                hover: {
                    stroke: "#C1B1F3",
                    strokeWidth: 1
                }
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
            map.updateSize();
        }
    }

    function onCountryClicked() {
        // Should play immediately after setting playlist (and shuffling if necessary)
        audioPlayer.shouldPlay = true;
        $playlistCountry = selectedCountry;
        $playlistType = "country";
        $playlist = dataSetCountryValue.data;
        console.log(dataSetCountryValue.data[0]);
    }

    // Selected country
    $: numberOfTracks = $playlist.length;
    $: artists = [...new Set($playlist.map((item) => item.artist))];
    $: numberOfArtists = artists.length;
    $: firstFewArtists = artists.slice(0, Math.min(3, artists.length));

    // Particles
    let particleContainer;
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
                    center: {
                        radius: 50,
                        x: 800,
                        y: 500,
                        mode: "precise"
                    },
                    bounce: true,
                    direction: "outside",
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

    function onParticlesLoaded(evt) {
        particleContainer = evt.detail.particles;
    }

    async function addCountryData() {
        await addCountryDataAllSongs();
    }
</script>

<svelte:window on:resize={onResize} />

<container bind:this={container}>
    <div class="bg" />
    <!-- 
    <Particles
        id="tsparticles"
        options={particlesOptions}
        {particlesInit}
        on:particlesLoaded={onParticlesLoaded}
    /> -->
    {#if !nOfCountries && $addOriginCountryStatus === null}
        <div class="welcome">
            <h2>Welcome to map view!</h2>
            <p>
                To use the map, you need to add some country data to your music:
            </p>

            <small>
                1) right-click a track → "Add Origin country", or
                <br />
                2) add manually in the Track Info overlay,
                <br />
                or click below to scan your whole library
            </small>
            <ButtonWithIcon text="Add country data" onClick={addCountryData} />
            <small class="hint"
                >This feature uses Wikipedia to read artist info. Some artists
                may not have an article, so you need to add the country
                manually.
            </small>
        </div>
    {/if}
    <content>
        <div class="header">
            {#if $addOriginCountryStatus}
                <div id="info">
                    <h2>Putting your music on the map...</h2>
                    <small>i.e checking where your artists are from</small>
                    <div class="progress">
                        <ProgressBar
                            percent={$addOriginCountryStatus.percent}
                        />
                    </div>
                </div>
            {:else if $playlistCountry}
                <div id="info">
                    <p>Listening to music from</p>
                    <h2>
                        {getFlagEmoji($playlistCountry)}{countries[
                            $playlistCountry
                        ]}
                    </h2>
                    <small
                        >{numberOfTracks} tracks from {numberOfArtists} artists</small
                    >
                </div>
            {/if}
        </div>
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
        <div id="map-tooltip">
            <MapTooltip data={tooltipData} />
        </div>
    {/if}
</container>

<style lang="scss">
    * {
        user-select: none;
    }

    container {
        position: relative;
        cursor: grab;
        border: 0.7px solid color-mix(in srgb, var(--inverse) 30%, transparent);
        margin: 5px 0 0 0;
        border-radius: 5px;
        overflow: hidden;
        background-color: var(--panel-background);
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
        opacity: 0.8;
        /* background-size: 35px 35px;
        background-image: repeating-linear-gradient(
            0deg,
            #d6d7e9,
            #d6d7e9 1px,
            transparent 1px,
            transparent
        ); */
        /* box-shadow: 0 0 180px 180px #242026c2 inset; */
        /* animation: bg-move 2s linear infinite forwards; */
        opacity: 0.4;
    }

    @keyframes bg-move {
        from {
            transform: translateY(0px);
        }
        to {
            transform: translateY(-35px);
        }
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
    #map-tooltip {
        position: fixed;
        pointer-events: none;
        margin-top: 1em;
        display: none;

        &:global(.active) {
            display: flex;
        }
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
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        margin-top: 2em;
    }

    #map {
        overflow: hidden;
        margin: auto;
        display: flex;
        height: fit-content;

        &:active {
            cursor: grab;
        }
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

    .welcome {
        position: absolute;
        top: 0;
        right: 0;
        left: 0;
        bottom: 0;
        margin: auto;
        padding: 0 4em;
        width: fit-content;
        display: flex;
        max-width: 500px;
        height: fit-content;
        align-items: center;
        flex-direction: column;
        gap: 1em;
        z-index: 10;
        padding: 2em;
        border-radius: 5px;
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(53, 51, 51);
        background: var(--overlay-bg);
        box-shadow: 0px 5px 40px rgba(0, 0, 0, 0.259);
        backdrop-filter: blur(8px);
        small,
        p {
            opacity: 0.8;
        }
        * {
            margin: 0;
        }

        .hint {
            max-width: 280px;
            opacity: 0.5;
        }
    }

    .progress {
        min-width: 300px;
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
