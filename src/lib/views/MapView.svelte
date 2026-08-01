<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, tick } from "svelte";
    import { get, writable, type Writable } from "svelte/store";
    import { songMatchesQuery } from "../../data/LibraryUtils";
    import { parsePlaylist } from "../../data/M3UUtils";
    import { queryAllTracks } from "../../data/query";
    import BuiltInQueries from "../../data/SmartQueries";
    import { beetsSearch, beetsSongsOnly } from "../../data/beets";
    import {
        expandedSongWithStems,
        isInit,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isTagOrCondition,
        libraryUpdater,
        queriedSongs,
        query,
        queueMirrorsSearch,
        selectedPlaylistFile,
        selectedSmartQuery,
        selectedTags,
        smartQuery,
        smartQueryResults,
        smartQueryUpdater,
        toDeletePlaylist,
        uiView,
        userSettings,
    } from "../../data/store";
    import { setQueue } from "../../data/storeHelper";
    import audioPlayer from "../player/AudioPlayer";
    import SmartQuery from "../smart-query/Query";

    import JsVectorMap from "../../deps/jsvectormap/js";
    import DataVisualization from "../../deps/jsvectormap/js/dataVisualization";
    import { world } from "../../deps/jsvectormap/maps/world.js";
    import "../../deps/jsvectormap/scss/jsvectormap.scss";
    JsVectorMap.addMap("world", world);

    import type {
        MapTooltipData,
        PlaylistFile,
        Song,
        SongOrder,
        SongsByCountry,
    } from "src/App";
    import {
        addOriginCountryStatus,
        queue,
        queueCountry,
    } from "../../data/store";
    import { currentThemeObject } from "../../theming/store";
    import { groupBy } from "../../utils/ArrayUtils";
    import { codes, countries, getFlagEmoji } from "../../data/CountryCodes";
    import { addCountryDataAllSongs } from "../../data/LibraryEnrichers";
    import MapTooltip from "../map/MapTooltip.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import ProgressBar from "../ui/ProgressBar.svelte";
    import Icon from "../ui/Icon.svelte";
    import Globe from "../map/Globe.svelte";
    import { fade } from "svelte/transition";

    export let songOrder: SongOrder;

    let isLoading = true;

    let container: HTMLElement;
    let map;
    let width = 0;
    let height = 0;
    let mapPadding = 0; //px

    let viewOptions = [
        {
            value: "2d",
            icon: "mdi:map",
        },
        {
            value: "3d",
            icon: "stash:globe",
        },
    ];

    let view = viewOptions[0].value;

    let globeViewOptions = [
        {
            value: "mesh",
            icon: "lucide:globe-2",
        },
        {
            value: "satellite",
            icon: "flat-color-icons:globe",
        },
    ];

    let globeView = globeViewOptions[0].value;

    /**
     * Same compound sort as CanvasLibraryView.svelte -- see that file for
     * why (fixes the same "TODO this needs to match the multiple indexes
     * sorting from Dexie" gap on the artist/album cases).
     */
    function sortSongs(results: Song[], order: SongOrder): Song[] {
        const dir = order.reverse ? -1 : 1;
        return results.sort((a, b) => {
            let result = 0;
            switch (order.orderBy) {
                case "artist":
                    result =
                        a.artist.localeCompare(b.artist) ||
                        (a.year ?? 0) - (b.year ?? 0) ||
                        a.album.localeCompare(b.album) ||
                        (a.trackNumber ?? 0) - (b.trackNumber ?? 0);
                    break;
                case "album":
                    result =
                        a.album.localeCompare(b.album) ||
                        (a.trackNumber ?? 0) - (b.trackNumber ?? 0);
                    break;
                case "trackNumber":
                    result = (a.trackNumber ?? 0) - (b.trackNumber ?? 0);
                    break;
                case "title":
                case "year":
                case "duration":
                case "genre":
                    result = String(a[order.orderBy]).localeCompare(
                        String(b[order.orderBy]),
                    );
                    break;
                default:
                    result = 0;
            }
            return result * dir;
        });
    }

    const librarySongs: Writable<Song[]> = writable(null);
    $: songs = $userSettings.beetsDbLocation ? beetsSongsOnly : librarySongs;

    async function fetchLibrarySongs(
        uiView: string,
        toDeletePlaylist: { tracks: string[] } | null,
        selectedPlaylistFile: PlaylistFile | null,
        query: string,
        isSmartQueryBuilderOpen: boolean,
        smartQuery: SmartQuery,
        smartQueryUpdater: number,
        selectedSmartQuery: string,
        isTagCloudOpen: boolean,
        selectedTags: Set<string>,
        isTagOrCondition: boolean,
        songOrder: SongOrder,
        _libraryUpdater: number,
    ) {
        isLoading = true;
        let results: Song[];
        let isSmartQueryResults = false;

        if (uiView === "to-delete") {
            if (toDeletePlaylist === null || toDeletePlaylist.tracks.length === 0) {
                results = [];
            } else {
                results = (
                    await invoke<(Song | null)[]>("get_tracks_by_ids", {
                        ids: toDeletePlaylist.tracks,
                    })
                ).filter((s): s is Song => s !== null);
            }
            results = results.map((s, idx) => ({
                ...s,
                viewModel: { index: idx, viewId: idx.toString() },
            }));
        } else if (selectedPlaylistFile !== null) {
            results = await parsePlaylist(selectedPlaylistFile);
            results = results.map((s, idx) => ({
                ...s,
                viewModel: { index: idx, viewId: idx.toString() },
            }));
            if (query.length) {
                results = results.filter((song) => songMatchesQuery(song, query));
            }
        } else if (uiView === "smart-query" || uiView === "favourites") {
            if (isSmartQueryBuilderOpen) {
                if (smartQuery.isEmpty) {
                    results = [];
                } else if (smartQueryUpdater) {
                    results = await smartQuery.run();
                } else {
                    results = [];
                }
            } else if (selectedSmartQuery.startsWith("~usq:")) {
                const loaded = await SmartQuery.loadWithUQI(
                    selectedSmartQuery as `~usq:${string}`,
                );
                results = await loaded.run();
            } else {
                results = await BuiltInQueries[selectedSmartQuery].run();
            }
            isSmartQueryResults = true;
        } else {
            results = await queryAllTracks({});
            if (query.length) {
                results = results.filter((song) => songMatchesQuery(song, query));
            }
        }

        if (isTagCloudOpen && selectedTags.size) {
            results = results.filter((song) => {
                let matches = [];
                if (selectedTags.size === 0) {
                    return true;
                } else if (song.tags?.length === 0) {
                    return false;
                }
                selectedTags.forEach((t) => {
                    song.tags?.forEach((tag) => {
                        if (tag === t) {
                            matches.push(t);
                        }
                    });
                });

                return isTagOrCondition
                    ? matches.length > 0 && matches.every((m) => selectedTags.has(m))
                    : matches.length === selectedTags?.size &&
                          matches.every((m) => selectedTags.has(m));
            });
        }

        results = sortSongs(results, songOrder);

        if (get(expandedSongWithStems)) {
            const idx = results.findIndex(
                (s) => s.id === get(expandedSongWithStems).id,
            );
            if (idx !== -1) {
                expandedSongWithStems.update((s) => ({
                    ...s,
                    viewModel: { ...s.viewModel, index: idx },
                }));
            } else {
                expandedSongWithStems.set(null);
            }
        }

        if (isSmartQueryResults) {
            smartQueryResults.set(results);
        } else {
            queriedSongs.set(results);
        }

        if (uiView === "library" && get(isInit)) {
            if (window["openedUrls"]?.length) {
                await audioPlayer.handleOpenedUrls(window["openedUrls"]);
            }
            isInit.set(false);
        }

        if (get(queueMirrorsSearch)) {
            setQueue(results, false);
            if (query.length === 0) {
                queueMirrorsSearch.set(false);
            }
        }

        librarySongs.set(results);
        isLoading = false;
    }

    $: if (!$userSettings.beetsDbLocation) {
        fetchLibrarySongs(
            $uiView,
            $toDeletePlaylist,
            $selectedPlaylistFile,
            $query,
            $isSmartQueryBuilderOpen,
            $smartQuery,
            $smartQueryUpdater,
            $selectedSmartQuery,
            $isTagCloudOpen,
            $selectedTags,
            $isTagOrCondition,
            songOrder,
            $libraryUpdater,
        );
    }

    let data: SongsByCountry = null;
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
                          if ($userSettings.beetsDbLocation) {
                              dataSetObj[currentCountry[0]] =
                                  currentCountry[1].data.length;
                          } else {
                              dataSetObj[codes[currentCountry[0]]] =
                                  currentCountry[1].data.length;
                          }
                          return dataSetObj;
                      },
                      {},
                  )
                : [];

            if (window && map) {
                map.dataVisualization = new DataVisualization(
                    {
                        scale: [
                            $currentThemeObject["accent-softest"],
                            $currentThemeObject["accent"],
                        ],
                        values: dataCountMap ? dataCountMap : [],
                    },
                    map,
                );
                if (!initialized) {
                    map.updateSize();
                    onResize();
                    initialized = true;
                }

                $queueCountry && setSelectedCountry($queueCountry);
            }
        }
    }

    function setSelectedCountry(countryCode) {
        if (!map) return;
        // Get path
        // map._clearSelected('regions');
        map.setSelectedRegions([countryCode]);
        // const allPaths = document.querySelectorAll("[data-code]");
        // allPaths.forEach((path) => {
        //     path.style.animation = null;
        // });
        const path: SVGPathElement = document.querySelector(
            `[data-code=${countryCode}]`,
        );
        let width = path.getBBox().width;
        let height = path.getBBox().height;
        let x = path.getBBox().x + width / 2;
        let y = path.getBBox().y + height / 2;

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
                window.addEventListener("resize", () => {
                    map.updateSize();
                });
            },
            draggable: true,
            zoomButtons: false,
            zoomOnScroll: true,
            zoomOnScrollSpeed: 0.1,

            zoomMax: 5,
            zoomMin: 1.2,
            zoomAnimate: false,
            zoomStep: 1.5,
            selector: "#map",
            showTooltip: true,
            visualizeData: {
                scale: [
                    $currentThemeObject["accent-softest"],
                    $currentThemeObject["accent"],
                ],
                values: dataCountMap ? dataCountMap : [],
            },
            // Play country
            onRegionClick(event, code) {
                if (!data) return;
                let countryValue = getCountryData(code);
                if (countryValue) {
                    selectedCountry = code;
                    selectedCountryPos = {
                        x: event.pageX - 11 - mapPadding * 2,
                        y: event.pageY,
                    };
                    dataSetCountryValue = countryValue || null;
                    onCountryClicked();
                }
            },
            onRegionTooltipShow(event, tooltip, code) {
                onCountryHovered(code);
            },
            map: "world",
            regionsSelectable: false,
            regionsSelectableOne: true,
            regionStyle: {
                initial: {
                    fill: $currentThemeObject["muted"],
                    stroke: $currentThemeObject["border"],
                    strokeWidth: 0.5,
                    fillOpacity: 1,
                },
                selected: {
                    fill: $currentThemeObject["accent-play"],
                    stroke: $currentThemeObject["border"],
                    strokeWidth: 1,
                    fillOpacity: 1,
                },
                selectedHover: {
                    fill: $currentThemeObject["accent-play"],
                    stroke: $currentThemeObject["border"],
                    strokeWidth: 1,
                    fillOpacity: 1,
                },
                hover: {
                    fill: $currentThemeObject["secondary"],
                    stroke: $currentThemeObject["accent"],
                    strokeWidth: 1,
                },
            },
        });
    }

    onMount(() => {
        onResize();
        resetMap();

        const resizeObserver = new ResizeObserver(() => {
            onResize();
        });

        resizeObserver.observe(container);

        return () => resizeObserver.unobserve(container);
    });

    async function onResize() {
        if (map) {
            map.reset();
            map.updateSize();
        }
        $queueCountry && setSelectedCountry($queueCountry);
    }
    function onCountryClicked() {
        // Should play immediately after setting playlist (and shuffling if necessary)
        console.log(dataSetCountryValue.data[0]);
        $queueCountry = selectedCountry;
        setQueue(dataSetCountryValue.data, 0);
    }

    // Selected country
    $: numberOfTracks = $queue.length;
    $: artists = [...new Set($queue.map((item) => item.artist))];
    $: numberOfArtists = artists.length;
    $: firstFewArtists = artists.slice(0, Math.min(3, artists.length));

    $: counts = dataCountMap ? (Object.values(dataCountMap) as number[]) : [];
    $: minCount = counts.length ? Math.min(...counts) : 0;
    $: maxCount = counts.length ? Math.max(...counts) : 0;
    $: legendGradient = `linear-gradient(to right, ${$currentThemeObject["accent-softest"]}, ${$currentThemeObject["accent"]})`;

    async function addCountryData() {
        await addCountryDataAllSongs();
    }

    async function onQueryChanged(query: string, songOrder: SongOrder) {
        if ($uiView === "map") {
            queriedSongs.set(
                await beetsSearch.updateSearch({
                    query,
                    sortBy: songOrder.orderBy,
                    descending: songOrder.reverse,
                }),
            );
        }
    }

    $: {
        $userSettings.beetsDbLocation &&
            onQueryChanged($query || "", songOrder);
    }
    let globeRef: Globe;

    function getCountryData(code: string) {
        let countryValue;
        if ($userSettings.beetsDbLocation) {
            countryValue = data[code]; // Beets also uses 2 char code.
        } else {
            countryValue = data[countries[code]];
        }
        return countryValue;
    }

    /**
     * Show the tooltip.
     * Called from both the map view and the globe.
     * @param code 2-letter country code
     */
    function onCountryHovered(code: string) {
        if (!data) return;
        let countryValue = getCountryData(code);
        if (countryValue) {
            // Hovered country
            let hoveredCountryPlaylist: Song[] = countryValue.data;

            let hoveredCountryArtists: string[] = [
                ...new Set(hoveredCountryPlaylist.map((item) => item.artist)),
            ];
            let hoveredCountryNumArtists = hoveredCountryArtists.length;
            let hoveredCountryFirstFewArtists = hoveredCountryArtists.slice(
                0,
                Math.min(3, hoveredCountryArtists.length),
            );

            let hoveredCountryFirstFewAlbums: {
                id: string;
                path: string;
                artist: string;
                album: string;
            }[] = hoveredCountryPlaylist.map((item) => ({
                id: item.albumId,
                path: item.path.replace(`/${item.file}`, ""),
                artist: item.artist,
                album: item.album,
            }));

            // distinct albums
            hoveredCountryFirstFewAlbums = hoveredCountryFirstFewAlbums.filter(
                (e, i) =>
                    hoveredCountryFirstFewAlbums.findIndex(
                        (a) => a["album"] === e["album"],
                    ) === i,
            );

            tooltipData = {
                countryName: countries[code],
                emoji: getFlagEmoji(code),
                numberOfArtists: hoveredCountryNumArtists,
                artists: hoveredCountryFirstFewArtists,
                albums: hoveredCountryFirstFewAlbums,
            };
        } else {
            tooltipData = {
                countryName: countries[code],
                emoji: getFlagEmoji(code),
                numberOfArtists: 0,
                artists: [],
                albums: [],
            };
        }
    }
</script>

<container>
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
            {:else if $queueCountry}
                <div id="info">
                    <p>Listening to music from</p>
                    <h2>
                        {getFlagEmoji($queueCountry)}{countries[$queueCountry]}
                    </h2>
                    <small
                        >{numberOfTracks} tracks from {numberOfArtists} artists</small
                    >
                </div>
            {/if}
        </div>
    </content>
    {#if songs}
        <div class="map-container" bind:this={container}>
            {#if view === "2d"}
                <div id="map" transition:fade />
            {:else}
                <div id="globe" transition:fade>
                    <Globe
                        bind:this={globeRef}
                        bind:selectedView={globeView}
                        songData={data}
                        {onCountryHovered}
                    />
                </div>
            {/if}
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
        <div class="map-options">
            <!-- <div class="tagging-hint">
                <ButtonWithIcon
                    theme="transparent"
                    icon="mdi:information"
                    text="How to add country data?"
                    onClick={() => {}}
                />
            </div> -->

            <ButtonWithIcon
                options={viewOptions}
                value={view}
                theme="translucent"
                onClick={async (value) => {
                    if (value === view) return;
                    view = value;
                    await tick();
                    if (view === "2d") {
                        onResize();
                        resetMap();
                        globeRef = null;
                    } else {
                        map = null;
                    }
                }}
            ></ButtonWithIcon>

            {#if view === "3d"}
                <ButtonWithIcon
                    options={globeViewOptions}
                    value={globeView}
                    theme="translucent"
                    onClick={async (value) => {
                        if (value === globeView) return;
                        globeView = value;
                    }}
                ></ButtonWithIcon>
            {/if}

            <ButtonWithIcon
                icon="iconamoon:zoom-in"
                theme="translucent"
                onClick={() => {
                    map?._setScale(
                        map.scale * map.params.zoomStep,
                        map._width / 2,
                        map._height / 2,
                        false,
                        map.params.zoomAnimate,
                    );
                    globeRef?.zoomIn();
                }}
            />
            <ButtonWithIcon
                icon="iconamoon:zoom-out"
                theme="translucent"
                onClick={() => {
                    map?._setScale(
                        map.scale / map.params.zoomStep,
                        map._width / 2,
                        map._height / 2,
                        false,
                        map.params.zoomAnimate,
                    );
                    globeRef?.zoomOut();
                }}
            />
            <div class="spacer"></div>
            {#if nOfCountries > 0}
                <div class="map-legend">
                    <span class="label">{minCount}</span>
                    <div
                        class="gradient-bar"
                        style="background: {legendGradient}"
                    ></div>
                    <span class="label">{maxCount} tracks</span>
                </div>
            {/if}
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
        margin: 0px 0 0 0;
        border-radius: 5px;
        overflow: hidden;
        background-color: var(--panel-background);

        .map-options {
            position: absolute;
            bottom: 15px;
            padding: 0 15px;
            z-index: 1;
            display: flex;
            flex-direction: row;
            gap: 5px;
            width: 100%;

            .tagging-hint {
                z-index: 10;
                flex-grow: 1;
                opacity: 0.5;
            }
            .control {
                background: var(--overlay-bg);
                padding: 8px 12px;
                border-radius: 6px;
                border: 1px solid
                    color-mix(in srgb, var(--inverse) 30%, transparent);
                backdrop-filter: blur(4px);
                box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
                cursor: pointer;

                &:hover {
                    background: var(--overlay-bg-hover);
                }
            }
            .spacer {
                flex-grow: 1;
            }

            .map-legend {
                padding: 8px 12px;
                border-radius: 6px;
                display: flex;
                align-items: center;
                gap: 10px;

                .label {
                    font-size: 0.9rem;
                    font-weight: 600;
                    color: var(--secondary);
                    white-space: nowrap;
                }

                .gradient-bar {
                    width: 100px;
                    height: 8px;
                    border-radius: 4px;
                }
            }
        }
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
        z-index: 100;

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

    .map-container {
        width: 100%;
        height: 100%;
    }

    #map {
        overflow: hidden;
        margin: auto;
        display: flex;
        &:active {
            cursor: grab;
        }
    }

    #globe {
        width: 100%;
        height: 100%;
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
        border: 1px solid color-mix(in srgb, var(--inverse) 20%, transparent);
        background: var(--overlay-bg);
        box-shadow: 0px 5px 40px var(--overlay-shadow);
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
