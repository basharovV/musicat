<script lang="ts">
    import { window as tauriWindow } from "@tauri-apps/api";
    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import { LogicalSize, PhysicalPosition } from "@tauri-apps/api/dpi";
    import { currentMonitor } from "@tauri-apps/api/window";
    import { Buffer } from "buffer";
    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import { throttle } from "lodash-es";
    import { onMount } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { flip } from "svelte/animate";
    import { cubicInOut } from "svelte/easing";
    import { fade, fly } from "svelte/transition";
    import type { PlaylistFile, Song } from "../../App";
    import {
        addSongsToPlaylists,
        createNewPlaylistFile,
        deletePlaylistFile,
        renamePlaylist,
    } from "../../data/M3UUtils";
    import SmartQueries from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import {
        current,
        currentIAFile,
        currentSongArtworkSrc,
        draggedAlbum,
        draggedSongs,
        isDraggingFromQueue,
        isFindFocused,
        isMiniPlayer,
        isPlaying,
        isShuffleEnabled,
        isSidebarOpen,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isWaveformOpen,
        isWikiOpen,
        lastWrittenSongs,
        os,
        playerTime,
        popupOpen,
        query,
        queue,
        rightClickedTrack,
        seekTime,
        selectedPlaylistFile,
        selectedSmartQuery,
        shouldFocusFind,
        sidebarManuallyOpened,
        sidebarTogglePos,
        singleKeyShortcutsEnabled,
        smartQueryInitiator,
        toDeletePlaylist,
        uiView,
        userPlaylists,
        userSettings,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { currentThemeObject } from "../../theming/store";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import { isIAPlaying } from "../player/WebAudioPlayer";
    import type { SavedSmartQuery } from "../smart-query/QueryPart";
    import "../tippy.css";
    import Icon from "../ui/Icon.svelte";
    import Input from "../ui/Input.svelte";
    import PlaybackSpeed from "../ui/PlaybackSpeed.svelte";
    import { optionalTippy } from "../ui/TippyAction";
    import VolumeSlider from "../ui/VolumeSlider.svelte";
    import Seekbar from "./Seekbar.svelte";

    const appWindow = tauriWindow.getCurrentWindow();

    // What to show in the sidebar
    let song: Song;
    let title;
    let fileName;
    $: displayTitle = title ?? fileName;
    let artist;
    let album;
    let artworkFormat;
    let artworkBuffer: Buffer;
    let artworkSrc;
    let previousArtworkSrc;
    let previousSongIdx;
    let codec;

    let duration;

    $: if (codec === "MPEG 1 Layer 3") {
        codec = "MP3";
    }

    let bitrate;
    let sampleRate;
    let stereo;

    $: elapsedTime = `${(~~($playerTime / 60))
        .toString()
        .padStart(2, "0")}:${(~~($playerTime % 60))
        .toString()
        .padStart(2, "0")}`;

    $: durationText = `${(~~(duration / 60)).toString().padStart(2, "0")}:${(~~(
        duration % 60
    ))
        .toString()
        .padStart(2, "0")}`;

    // Shortcuts
    hotkeys("space", function (event, handler) {
        // Prevent the default refresh event under WINDOWS system
        event.preventDefault();
        audioPlayer.togglePlay();
    });

    let artworkCanvas: HTMLCanvasElement;

    let placeholderArtwork: HTMLImageElement;

    current.subscribe(async (current) => {
        if (current.song) {
            if (
                current.song.path === song?.path &&
                !$lastWrittenSongs.some(
                    ({ path }) => path === current.song.path,
                )
            ) {
                // same song, no need to update
                // (unless the metadata was just written to eg. updated artwork)
                return;
            }

            song = current.song;

            const songWithArtwork = await invoke<Song>("get_song_metadata", {
                event: {
                    path: song.path,
                    isImport: false,
                    includeFolderArtwork: true,
                },
            });

            if (!songWithArtwork) {
                title = "❗️File read error❗️";
                artist = "Check permissions";
                album = "in use by another program?";
                toast.error(
                    `Error reading file ${song.path}. Check permissions, or if the file is used by another program.`,
                    { className: "app-toast" },
                );
                return;
            }
            console.log("sidebar::currentSong listener", songWithArtwork);
            title = songWithArtwork.title;
            fileName = song.file;
            artist = songWithArtwork.artist;
            album = songWithArtwork.album;
            codec = songWithArtwork.fileInfo.codec;
            stereo = songWithArtwork.fileInfo.channels === 2;
            bitrate = songWithArtwork.fileInfo.bitDepth;
            sampleRate = songWithArtwork.fileInfo.sampleRate;
            duration = songWithArtwork.fileInfo.duration;
            previousArtworkSrc = artworkSrc;
            if (songWithArtwork.artwork) {
                artworkFormat = songWithArtwork.artwork.format;
                if (songWithArtwork.artwork.data?.length) {
                    artworkBuffer = Buffer.from(songWithArtwork.artwork.data);
                    artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                        "base64",
                    )}`;
                } else if (songWithArtwork.artwork.src) {
                    artworkSrc = convertFileSrc(songWithArtwork.artwork.src);
                }

                console.log("artworkSrc", artworkSrc);
                $currentSongArtworkSrc = {
                    src: artworkSrc,
                    format: artworkFormat,
                    size: {
                        width: 200,
                        height: 200,
                    },
                };
            } else {
                artworkSrc = null;
            }

            drawArtwork(previousSongIdx > current.index);
            previousSongIdx = current.index;
        }
    });

    function openTrackInfo() {
        if (song) {
            $rightClickedTrack = song;
            $popupOpen = "track-info";
        }
    }

    function openInfoWindow() {
        // const webview = new WebviewWindow("theUniqueLabel", {
        //   url: "path/to/page.html",
        // });
        console.log("clicked");
        $popupOpen = "info";
    }
    $: {
        console.log("info popup:", $popupOpen === "info");
    }
    let height = 0;
    let width = 0;
    let hasDecorations = false;

    let sidebarToggleX = 0;
    let sidebarToggleY = 0;
    async function onResize() {
        // Check if is miniplayer mode
        height = window.innerHeight;
        width = window.innerWidth;
        hasDecorations = await appWindow.isDecorated();
        if (!$isMiniPlayer && height <= 220 && width <= 210) {
            $isMiniPlayer = true;
            console.log("setting to false");
            // await appWindow.setDecorations(false);
        } else if ($isMiniPlayer && (height > 220 || width > 210)) {
            $isMiniPlayer = false;
            console.log("setting to true");
            // await appWindow.setDecorations(true);
        }

        // Get bottom coordinates of top container
        const topContainer = sidebar?.querySelector(".top");

        if (topContainer) {
            $sidebarTogglePos = {
                x: topContainer.getBoundingClientRect().right,
                y: topContainer.getBoundingClientRect().bottom,
            };
        }
    }

    let searchInput: HTMLInputElement;

    function onSearchInputKeyDown(event: KeyboardEvent) {
        if (event.keyCode === 38) {
            event.preventDefault();
            // up
        } else if (event.keyCode === 40) {
            // down
            event.preventDefault();
        } else if (event.keyCode === 13) {
            event.preventDefault();
        } else if (event.keyCode === 27) {
            event.preventDefault();
            $shouldFocusFind = { target: "search", action: "unfocus" };
        }
    }

    let miniToggleBtn: HTMLElement;
    let widthToRestore = 0;
    let heightToRestore = 0;
    let paddingPx = 40;
    let isMiniToggleHovered = false;
    let isMiniPlayerHovered = false;

    /**
     * We handle the hover event manually because otherwise
     * the hover state gets stuck when resizing and moving the window
     */
    function onMiniToggleMouseOver() {
        isMiniToggleHovered = true;
    }
    function onMiniToggleMouseOut() {
        isMiniToggleHovered = false;
    }
    function onMiniPlayerMouseOver() {
        isMiniPlayerHovered = true;
    }
    function onMiniPlayerMouseOut() {
        isMiniPlayerHovered = false;
    }

    async function toggleMiniPlayer() {
        if (!$isMiniPlayer) {
            widthToRestore = window.innerWidth;
            heightToRestore = window.innerHeight;

            await appWindow.hide();
            await appWindow.setSize(new LogicalSize(210, 210));
            const monitor = await currentMonitor();
            const windowSize = await appWindow.innerSize();
            switch ($userSettings.miniPlayerLocation) {
                case "bottom-left":
                    await appWindow.setPosition(
                        new PhysicalPosition(
                            monitor.position.x + paddingPx,
                            monitor.position.y +
                                monitor.size.height -
                                windowSize.height -
                                paddingPx,
                        ),
                    );
                    break;
                case "bottom-right":
                    await appWindow.setPosition(
                        new PhysicalPosition(
                            monitor.position.x +
                                monitor.size.width -
                                windowSize.width -
                                paddingPx,
                            monitor.position.y +
                                monitor.size.height -
                                windowSize.height -
                                paddingPx,
                        ),
                    );
                    break;
                case "top-left":
                    await appWindow.setPosition(
                        new PhysicalPosition(
                            monitor.position.x + paddingPx,
                            monitor.position.y +
                                ($os === "macos" ? paddingPx + 40 : paddingPx),
                        ),
                    );
                    break;
                case "top-right":
                    await appWindow.setPosition(
                        new PhysicalPosition(
                            monitor.position.x +
                                monitor.size.width -
                                windowSize.width -
                                paddingPx,
                            monitor.position.y +
                                ($os === "macos" ? paddingPx + 40 : paddingPx),
                        ),
                    );
                    break;
            }

            await appWindow.show();
            isMiniPlayerHovered = false; // By default we want to show the pretty artwork
        } else {
            await appWindow.hide();
            if (widthToRestore && heightToRestore) {
                await appWindow.setSize(
                    new LogicalSize(widthToRestore, heightToRestore),
                );
            } else {
                await appWindow.setSize(new LogicalSize(1100, 750));
            }

            await appWindow.center();
            await appWindow.show();
        }

        isMiniToggleHovered = false;
    }

    appWindow.listen("tauri://focus", (evt) => {
        isMiniPlayerHovered = true;
    });
    appWindow.listen("tauri://blur", (evt) => {
        isMiniPlayerHovered = false;
    });

    // Menu
    let menuScrollPos = 0;
    let menuOuterContainer: HTMLDivElement;
    let menuInnerScrollArea: HTMLDivElement;
    let menu: HTMLElement;
    let scrollbar: HTMLDivElement;
    let isScrollbarVisible = false;
    let showMenuTopScrollShadow = false;
    let showMenuBottomScrollShadow = false;

    function onMenuResize() {
        console.log(
            "scrollTop",
            menuInnerScrollArea.scrollTop,
            menuInnerScrollArea.clientHeight,
            menuInnerScrollArea.scrollHeight,
        );
        // Check scroll area size, add shadows if necessary
        if (menuInnerScrollArea) {
            showMenuTopScrollShadow =
                menuInnerScrollArea.scrollTop > 0 &&
                menuInnerScrollArea.scrollHeight >
                    menuInnerScrollArea.clientHeight;
            showMenuBottomScrollShadow =
                menuInnerScrollArea.scrollTop <
                    menuInnerScrollArea.scrollHeight -
                        menuInnerScrollArea.clientHeight &&
                menuInnerScrollArea.scrollHeight >
                    menuInnerScrollArea.clientHeight;
        }
    }

    function onMenuScroll(e) {
        if (!isScrollbarVisible) {
            isScrollbarVisible = true;
            setTimeout(() => {
                isScrollbarVisible = false;
            }, 2000);
        }
        const menuHeight = menuOuterContainer.clientHeight;
        const scrollPercentage =
            menuInnerScrollArea.scrollTop /
            (menuInnerScrollArea.scrollHeight -
                menuInnerScrollArea.clientHeight);
        // console.log("percent", scrollPercentage);
        menuScrollPos =
            (menuHeight - scrollbar.clientHeight - 13) * scrollPercentage;

        // Show top shadow
        showMenuTopScrollShadow =
            menuInnerScrollArea.scrollTop > 0 &&
            menuInnerScrollArea.scrollHeight > menuInnerScrollArea.clientHeight;

        // Show bottom shadow
        showMenuBottomScrollShadow =
            menuInnerScrollArea.scrollTop <
                menuInnerScrollArea.scrollHeight -
                    menuInnerScrollArea.clientHeight &&
            menuInnerScrollArea.scrollHeight > menuInnerScrollArea.clientHeight;
    }

    let newPlaylistTitle = "";
    let updatedPlaylistName = ""; // also for renaming existing playlists
    let isPlaylistsExpanded = false;
    let showPlaylistMenu = false;
    let menuX = 0;
    let menuY = 0;
    let isConfirmingPlaylistDelete = false;
    let playlistToEdit: PlaylistFile = null;
    let draggingOverPlaylist: PlaylistFile = null;
    let hoveringOverPlaylistId: string = null;
    let isRenamingPlaylist = false;

    let isSmartPlaylistsExpanded = false;
    let showSmartPlaylistMenu = false;
    let isConfirmingSmartPlaylistDelete = false;
    let smartPlaylistToEdit: SavedSmartQuery = null;
    let updatedSmartPlaylistName = "";
    let isRenamingSmartPlaylist = false;
    let hoveringOverSmartPlaylistId: number | string = null;

    async function onCreatePlaylist() {
        createNewPlaylistFile(newPlaylistTitle);
        newPlaylistTitle = "";
    }

    async function deletePlaylist() {
        if (!isConfirmingPlaylistDelete) {
            isConfirmingPlaylistDelete = true;
            return;
        }
        await deletePlaylistFile(playlistToEdit);
        $selectedPlaylistFile = $userPlaylists[0];
        showPlaylistMenu = false;
        isConfirmingPlaylistDelete = false;
    }

    async function onRenamePlaylist(playlist: PlaylistFile) {
        await renamePlaylist(playlist, updatedPlaylistName);
        updatedPlaylistName = "";

        isRenamingPlaylist = false;
    }

    function onMouseOverPlaylist(playlist: PlaylistFile) {
        if (
            $draggedSongs.length &&
            draggingOverPlaylist?.title !== playlist?.title
        ) {
            draggingOverPlaylist = playlist;
            hoveringOverPlaylistId = null;
        } else {
            hoveringOverPlaylistId = playlist?.title;
        }
    }

    function onMouseLeavePlaylist() {
        draggingOverPlaylist = null;
        hoveringOverPlaylistId = null;
    }

    // $: {
    //     if ($draggedSongs.length === 0) {
    //         draggingOverPlaylist = null;
    //     }
    // }

    $: savedSmartQueries = liveQuery(async () => {
        return db.smartQueries.toArray();
    });

    async function onRenameSmartPlaylist(smartQuery: SavedSmartQuery) {
        smartQuery.name = updatedSmartPlaylistName;
        await db.smartQueries.put(smartQuery);
        updatedSmartPlaylistName = "";
        isRenamingSmartPlaylist = false;
    }

    async function deleteSmartPlaylist() {
        if (!isConfirmingSmartPlaylistDelete) {
            isConfirmingSmartPlaylistDelete = true;
            return;
        }
        await db.smartQueries.delete(smartPlaylistToEdit.id);
        showSmartPlaylistMenu = false;
        isConfirmingSmartPlaylistDelete = false;

        // Re-select a smart playlist automatically
        const userQueries = await db.smartQueries.toArray();
        if (userQueries.length === 0) {
            $selectedSmartQuery = Object.values(SmartQueries)[0].value;
        } else {
            $selectedSmartQuery = `~usq:${userQueries[0].id}`;
        }
        console.log("queries", userQueries);
    }

    function onMouseOverSmartPlaylist(queryId: number | string) {
        hoveringOverSmartPlaylistId = queryId;
    }

    function onMouseLeaveSmartPlaylist() {
        hoveringOverSmartPlaylistId = null;
    }

    async function onDropSongsToPlaylist(playlist: PlaylistFile) {
        if ($draggedSongs.length) {
            console.log("[Sidebar] Adding to playlist: ", playlist);
            await addSongsToPlaylists(playlist, $draggedSongs);
            $selectedPlaylistFile = $selectedPlaylistFile; // trigger re-render
            toast.success(
                `${
                    $draggedSongs.length > 1
                        ? $draggedSongs.length + " songs"
                        : $draggedSongs[0].title
                } added to ${playlist.path}`,
                {
                    position: "bottom-center",
                },
            );
            $draggedSongs = [];
            $draggedAlbum = null;
            $isDraggingFromQueue = false;
        }
    }

    async function favouriteCurrentSong() {
        if (!$current.song) return;
        $current.song.isFavourite = !$current.song.isFavourite;
        await db.songs.put($current.song);
        $current = $current;
    }
    let sidebar;
    let sidebarWidth = 210;
    let titleElement: HTMLParagraphElement;
    let isTitleOverflowing = false; // to show marquee

    onMount(async () => {
        shouldFocusFind.subscribe((event) => {
            console.log("event", event);
            if (event?.target === "search") {
                if (searchInput) {
                    if (event.action === "focus") {
                        searchInput.focus();
                    } else if (event.action === "unfocus") {
                        searchInput.blur();
                    }
                }
            }
            if (event !== null) {
                $shouldFocusFind = null;
            }
        });

        searchInput.onfocus = (evt) => {
            $singleKeyShortcutsEnabled = false;
            $isFindFocused = true;
        };
        searchInput.onblur = (evt) => {
            $singleKeyShortcutsEnabled = true;
            $isFindFocused = false;
        };

        // Detect size changes in scroll container for the menu
        const resizeObserver = new ResizeObserver(onMenuResize).observe(menu);

        height = window.innerHeight;
        window.onresize = throttle(() => {
            onResize();
        }, 200);

        onResize(); // run once
    });

    let canvas: HTMLCanvasElement;

    $: if (
        song &&
        canvas &&
        sidebarWidth &&
        displayTitle &&
        $currentThemeObject
    ) {
        console.log("title", title);
        // Too early - song is changed, but not the title
        isTitleOverflowing =
            titleElement?.scrollWidth > titleElement?.clientWidth;

        resetMarquee();

        // Get bottom coordinates of top container
        const topContainer = sidebar.querySelector(".top");

        if (topContainer) {
            $sidebarTogglePos = {
                x: topContainer.getBoundingClientRect().right - 15,
                y: topContainer.getBoundingClientRect().bottom - 10,
            };
        }
    }

    function clearMarquee() {
        const context = canvas.getContext("2d");
        if (!context) return;
        context.clearRect(0, 0, canvas.width, canvas.height);
    }
    let animation;
    function resetMarquee() {
        if (animation !== undefined) cancelAnimationFrame(animation);

        const context = canvas.getContext("2d");
        if (!canvas || !context) return;
        const speed = 2; // Adjust the speed as needed

        context.font =
            "bold 36px -apple-system, Avenir, Helvetica, Arial, sans-serif";
        context.fillStyle = $currentThemeObject["text-active"];
        let gap = 100;
        let textWidth = context.measureText(displayTitle).width;

        let x = (canvas.width - textWidth) / 2; // Initial x-coordinate for the text
        let x2 = x + context.measureText(displayTitle).width + gap;
        console.log("x", x, "x2", x2);
        let started = false;
        var lastFrameTime = 0;
        let yPos = 35;

        function animate(elapsedTime) {
            if (!canvas) return;
            let textWidth = context.measureText(displayTitle).width;
            let isOverflowing = textWidth > canvas.width;
            if (isOverflowing) {
                if (!started) {
                    started = true;
                    setTimeout(() => {
                        animation = requestAnimationFrame(animate);
                    }, 200);
                } else {
                    animation = requestAnimationFrame(animate);
                }
                // calculate the delta since the last frame
                var delta = elapsedTime - (lastFrameTime || 0);

                // if we *don't* already have a first frame, and the
                // delta is less than 33ms (30fps in this case) then
                // don't do anything and return
                if (lastFrameTime && delta < 20) {
                    return;
                }

                context.clearRect(0, 0, canvas.width, canvas.height);

                // else we have a frame we want to draw at 30fps...

                // capture the last frame draw time so we can work out
                // a delta next time.
                lastFrameTime = elapsedTime;

                // now do the frame update and render work
                // ...
                context.fillText(displayTitle, x, yPos);
                context.fillText(displayTitle, x2, yPos);

                x -= speed;
                x2 -= speed;

                // Reset x-coordinate when the text goes off the left side of the canvas
                if (x < -context.measureText(displayTitle).width) {
                    x = x2 + context.measureText(displayTitle).width + gap;
                }
                if (x2 < -context.measureText(displayTitle).width) {
                    x2 = x + context.measureText(displayTitle).width + gap;
                }
            } else {
                context.clearRect(0, 0, canvas.width, canvas.height);

                context.fillText(
                    displayTitle,
                    (canvas.width - textWidth) / 2,
                    yPos,
                );
            }
        }
        animate(0);
    }

    /**
     * Draws artwork from artworkSrc and artworkFormat, otherwise shows the placeholder image
     * Also animates changes between tracks by sliding the artwork to the left
     */
    function drawArtwork(isPrevious) {
        if (artworkCanvas) {
            const context = artworkCanvas.getContext("2d");
            if (!context) return;

            context.clearRect(0, 0, artworkCanvas.width, artworkCanvas.height);
            if (song) {
                const artwork = artworkSrc;
                console.log("artwork", artwork);
                if (artwork) {
                    const img = new Image();
                    img.src = artwork;
                    img.onload = () => {
                        context.drawImage(
                            img,
                            0,
                            0,
                            artworkCanvas.width,
                            artworkCanvas.height,
                        );
                    };
                } else {
                    // Show placeholder
                    if (!placeholderArtwork) {
                        placeholderArtwork = new Image();
                        placeholderArtwork.src = "icon.png";
                        placeholderArtwork.onload = () => {
                            context.clearRect(
                                0,
                                0,
                                artworkCanvas.width,
                                artworkCanvas.height,
                            );
                            context.drawImage(
                                placeholderArtwork,
                                45,
                                45,
                                120,
                                120,
                            );
                        };
                    } else {
                        context.clearRect(
                            0,
                            0,
                            artworkCanvas.width,
                            artworkCanvas.height,
                        );
                        context.drawImage(placeholderArtwork, 45, 45, 120, 120);
                    }
                }
                // If previousArtworkSrc differs from artworkSrc, animate the artwork
                if (artwork && previousArtworkSrc !== artworkSrc) {
                    animateArtwork(
                        context,
                        previousArtworkSrc,
                        artworkSrc,
                        artworkCanvas.width,
                        artworkCanvas.height,
                        isPrevious ? "right" : "left",
                    );
                }
            } else {
                const img = new Image();
                img.src = "icon.png";
                img.onload = () => {
                    context.clearRect(
                        0,
                        0,
                        artworkCanvas.width,
                        artworkCanvas.height,
                    );

                    context.drawImage(img, 45, 45, 120, 120);
                };
            }
        }
    }

    function animateArtwork(
        ctx,
        src1,
        src2,
        width,
        height,
        direction = "left",
    ) {
        const img1 = new Image();
        const img2 = new Image();
        let animationId;
        let x = 0;
        let startTime;

        img1.src = src1;
        img2.src = src2;

        img1.onload = () => {
            ctx.drawImage(
                img1,
                0,
                0,
                artworkCanvas.width,
                artworkCanvas.height,
            );
            img2.onload = startAnimation;
        };

        function easeInOutQuad(t) {
            return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
        }

        function startAnimation() {
            startTime = performance.now();
            const duration = 200; // 1 second animation duration

            const step = (currentTime) => {
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);
                const easedProgress = easeInOutQuad(progress);

                const x = easedProgress * width;

                const opacity = 1 - progress; // Fade out the first image, fade in the second image

                ctx.clearRect(0, 0, width, height);

                // Draw the first image with fading out
                ctx.globalAlpha = opacity;
                ctx.drawImage(
                    img1,
                    direction === "left" ? -x : x,
                    0,
                    width,
                    height,
                );

                // Draw the second image with fading in
                ctx.globalAlpha = 1 - opacity;
                ctx.drawImage(
                    img2,
                    direction === "left" ? width - x : -width + x,
                    0,
                    width,
                    height,
                );

                if (progress < 1) {
                    requestAnimationFrame(step);
                } else {
                    ctx.clearRect(0, 0, width, height);
                    ctx.drawImage(img2, 0, 0, width, height);
                }
            };

            requestAnimationFrame(step);
        }
    }

    // Playback speed
    let isPlaybackSpeedControlOpen = false;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<sidebar
    class:has-current-song={song}
    class:empty={!song}
    class:hovered={isMiniPlayerHovered}
    class:visible={$isSidebarOpen}
    transition:fly={{ duration: 200, x: -200 }}
    bind:this={sidebar}
    on:mouseenter|preventDefault|stopPropagation={onMiniPlayerMouseOver}
    on:mouseleave={onMiniPlayerMouseOut}
    data-tauri-drag-region
>
    <!-- <div class="knob">
    <Knob bind:value={volumeKnob} max={100} min={0} pixelRange={200} />
  </div> -->
    <div class="top" data-tauri-drag-region>
        <h1 class="app-title" on:click={openInfoWindow}>Musicat</h1>
        <div class="top-header" data-tauri-drag-region>
            <div class="search-container">
                <input
                    bind:this={searchInput}
                    class="search"
                    id="search"
                    type="text"
                    autocomplete="off"
                    spellcheck="false"
                    placeholder="{$LL.sidebar.search()} ({$os === 'macos'
                        ? 'Cmd + F'
                        : 'Ctrl + F'})"
                    bind:value={$query.query}
                    on:keydown={onSearchInputKeyDown}
                />
                <div class="search-icon">
                    {#if $query.query.length}
                        <Icon
                            icon="mingcute:close-circle-fill"
                            size={15}
                            onClick={() => {
                                $query.query = "";
                            }}
                        />
                    {:else}
                        <Icon icon="ion:search" size={16} />
                    {/if}
                </div>
            </div>
        </div>

        <div
            class="menu-outer"
            class:top-border={showMenuTopScrollShadow}
            class:top-shadow={showMenuTopScrollShadow}
            class:bottom-shadow={showMenuBottomScrollShadow}
            bind:this={menuOuterContainer}
        >
            {#if showMenuTopScrollShadow}
                <div transition:fade={{ duration: 150 }} class="top-shadow" />
            {/if}
            {#if showMenuBottomScrollShadow}
                <div
                    transition:fly={{ duration: 150, y: 20 }}
                    class="bottom-shadow"
                />
            {/if}
            <div
                bind:this={menuInnerScrollArea}
                class="menu-inner"
                on:scroll={onMenuScroll}
            >
                <menu bind:this={menu}>
                    <items>
                        <item
                            class:selected={$uiView === "library" &&
                                !$selectedPlaylistFile}
                            on:click={() => {
                                $isSmartQueryBuilderOpen = false;
                                $selectedPlaylistFile = null;
                                $selectedSmartQuery = null;
                                $query.orderBy = $query.libraryOrderBy;
                                $uiView = "library";
                            }}
                        >
                            <Icon
                                icon="fluent:library-20-filled"
                                size={15}
                                color={$uiView === "library" &&
                                !$selectedPlaylistFile
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.library()}</item
                        >
                        <item
                            class:selected={$uiView === "albums"}
                            on:click={() => {
                                $uiView = "albums";
                                $selectedPlaylistFile = null;
                                $selectedSmartQuery = null;
                            }}
                        >
                            <Icon
                                icon="ic:round-album"
                                size={15}
                                color={$uiView === "albums"
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.albums()}</item
                        >

                        <item
                            class:selected={$uiView === "favourites"}
                            on:click={() => {
                                $uiView = "favourites";
                                $selectedPlaylistFile = null;
                                $selectedSmartQuery = null;
                                $selectedSmartQuery =
                                    SmartQueries.favourites.value;
                            }}
                        >
                            <Icon
                                icon="clarity:heart-solid"
                                size={15}
                                color={$uiView === "favourites"
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.favorites()}</item
                        >

                        {#if $toDeletePlaylist?.tracks.length}
                            <item
                                class:selected={$uiView === "to-delete"}
                                on:click={() => {
                                    $uiView = "to-delete";
                                    $query.orderBy = "none";
                                    $query.reverse = false;
                                    $selectedSmartQuery = null;
                                }}
                            >
                                <Icon
                                    icon="ant-design:delete-outlined"
                                    size={15}
                                    color={$uiView === "to-delete"
                                        ? $currentThemeObject["accent"]
                                        : "currentColor"}
                                />{$LL.sidebar.toDelete()}</item
                            >
                        {/if}

                        <item
                            class:selected={$uiView === "playlists" &&
                                $selectedPlaylistFile}
                            on:click={() => {
                                // expand playlists
                                isPlaylistsExpanded = !isPlaylistsExpanded;
                            }}
                            on:mouseenter={() => {
                                if ($draggedSongs.length > 0)
                                    isPlaylistsExpanded = true;
                            }}
                        >
                            <Icon
                                icon="mdi:playlist-music"
                                size={15}
                                color={$uiView === "library" &&
                                $selectedPlaylistFile
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.playlists()}
                            <div
                                class="chevron"
                                class:expanded={isPlaylistsExpanded}
                            >
                                <Icon icon="lucide:chevron-down" size={14} />
                            </div>
                        </item>

                        {#if isPlaylistsExpanded}
                            <div class="playlists">
                                {#if $userPlaylists.sort((a, b) => {
                                    return a.title.localeCompare(b.title);
                                })}
                                    {#each $userPlaylists as playlist (playlist.path)}
                                        <div
                                            animate:flip={{
                                                duration: 300,
                                                easing: cubicInOut,
                                            }}
                                            class="playlist"
                                            class:dragover={draggingOverPlaylist ===
                                                playlist}
                                            class:hover={hoveringOverPlaylistId ===
                                                playlist?.title}
                                            class:selected={$selectedPlaylistFile?.path ===
                                                playlist.path}
                                            on:click={() => {
                                                $uiView = "playlists";
                                                // Opening a playlist will reset the query
                                                $query = {
                                                    ...$query,
                                                    orderBy: "none",
                                                    reverse: false,
                                                    query: "",
                                                };
                                                $selectedPlaylistFile =
                                                    playlist;
                                                $selectedSmartQuery = null;
                                            }}
                                            on:mouseleave|preventDefault|stopPropagation={onMouseLeavePlaylist}
                                            on:mouseenter|preventDefault|stopPropagation={() =>
                                                onMouseOverPlaylist(playlist)}
                                            on:mouseup|preventDefault|stopPropagation={() =>
                                                onDropSongsToPlaylist(playlist)}
                                        >
                                            {#if isRenamingPlaylist && playlistToEdit.title === playlist.title}
                                                <Input
                                                    bind:value={updatedPlaylistName}
                                                    onEnterPressed={() => {
                                                        onRenamePlaylist(
                                                            playlist,
                                                        );
                                                    }}
                                                    fullWidth
                                                    minimal
                                                    autoFocus
                                                />
                                            {:else}
                                                <p>{playlist.title}</p>
                                            {/if}
                                            {#if isRenamingPlaylist && playlistToEdit.title === playlist.title}
                                                <Icon
                                                    icon="mingcute:close-circle-fill"
                                                    size={14}
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        isRenamingPlaylist = false;
                                                    }}
                                                />
                                            {:else}
                                                <div
                                                    class="playlist-options"
                                                    class:visible={showPlaylistMenu &&
                                                        playlistToEdit ===
                                                            playlist}
                                                >
                                                    <Icon
                                                        icon="charm:menu-kebab"
                                                        color="#898989"
                                                        size={14}
                                                        onClick={(e) => {
                                                            menuX = e.clientX;
                                                            menuY = e.clientY;
                                                            playlistToEdit =
                                                                playlist;
                                                            showPlaylistMenu =
                                                                !showPlaylistMenu;
                                                        }}
                                                    />
                                                </div>
                                            {/if}
                                        </div>
                                    {/each}
                                {/if}
                                <div class="new-playlist">
                                    <Input
                                        bind:value={newPlaylistTitle}
                                        placeholder="New playlist"
                                        onEnterPressed={onCreatePlaylist}
                                        fullWidth
                                        minimal
                                    />
                                </div>
                            </div>
                        {/if}
                        <item
                            class:selected={$uiView === "smart-query"}
                            on:click={() => {
                                $smartQueryInitiator = "sidebar";
                                isSmartPlaylistsExpanded =
                                    !isSmartPlaylistsExpanded;
                            }}
                        >
                            <Icon
                                icon="ic:round-star-outline"
                                size={15}
                                color={$uiView === "smart-query"
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.smartPlaylists()}
                            <div
                                class="chevron"
                                class:expanded={isSmartPlaylistsExpanded}
                            >
                                <Icon icon="lucide:chevron-down" size={14} />
                            </div></item
                        >

                        {#if isSmartPlaylistsExpanded}
                            <div class="playlists">
                                {#each Object.values(SmartQueries) as smartQuery (smartQuery.value)}
                                    <div
                                        animate:flip={{
                                            duration: 300,
                                            easing: cubicInOut,
                                        }}
                                        class="playlist"
                                        class:hover={hoveringOverSmartPlaylistId ===
                                            smartQuery.value}
                                        class:selected={$selectedSmartQuery ===
                                            smartQuery.value}
                                        on:click={() => {
                                            $uiView = "smart-query";
                                            $query.orderBy = "none";
                                            $query.reverse = false;
                                            $selectedSmartQuery =
                                                smartQuery.value;
                                            $selectedPlaylistFile = null;
                                        }}
                                        on:mouseleave|preventDefault|stopPropagation={onMouseLeaveSmartPlaylist}
                                        on:mouseenter|preventDefault|stopPropagation={() =>
                                            onMouseOverSmartPlaylist(
                                                smartQuery.value,
                                            )}
                                    >
                                        <p>{smartQuery.name}</p>
                                    </div>
                                {/each}
                                {#each $savedSmartQueries as query (query.id)}
                                    <div
                                        animate:flip={{
                                            duration: 300,
                                            easing: cubicInOut,
                                        }}
                                        class="playlist"
                                        class:selected={$selectedSmartQuery ===
                                            `~usq:${query.id}`}
                                        class:hover={hoveringOverSmartPlaylistId ===
                                            query.id}
                                        on:click={() => {
                                            $uiView = "smart-query";
                                            $selectedSmartQuery = `~usq:${query.id}`;
                                        }}
                                        on:mouseleave|preventDefault|stopPropagation={onMouseLeaveSmartPlaylist}
                                        on:mouseenter|preventDefault|stopPropagation={() =>
                                            onMouseOverSmartPlaylist(query.id)}
                                    >
                                        {#if isRenamingSmartPlaylist && smartPlaylistToEdit.id === query.id}
                                            <Input
                                                bind:value={updatedSmartPlaylistName}
                                                onEnterPressed={() => {
                                                    onRenameSmartPlaylist(
                                                        query,
                                                    );
                                                }}
                                                fullWidth
                                                minimal
                                                autoFocus
                                            />
                                        {:else}
                                            <p>{query.name}</p>
                                        {/if}
                                        {#if isRenamingSmartPlaylist && smartPlaylistToEdit.id === query.id}
                                            <Icon
                                                icon="mingcute:close-circle-fill"
                                                size={14}
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    isRenamingSmartPlaylist = false;
                                                }}
                                            />
                                        {:else}
                                            <div
                                                class="playlist-options"
                                                class:visible={showSmartPlaylistMenu &&
                                                    smartPlaylistToEdit.id ===
                                                        query.id}
                                            >
                                                <Icon
                                                    icon="charm:menu-kebab"
                                                    color="#898989"
                                                    size={14}
                                                    onClick={(e) => {
                                                        menuX = e.clientX;
                                                        menuY = e.clientY;
                                                        smartPlaylistToEdit =
                                                            query;
                                                        showSmartPlaylistMenu =
                                                            !showSmartPlaylistMenu;
                                                    }}
                                                />
                                            </div>
                                        {/if}
                                    </div>
                                {/each}

                                <!-- {#if $savedSmartQueries}
                                    {#each $savedSmartQueries as query}
                                        <option value={`~usq:${query.name}`}
                                            >{query.name}</option
                                        >
                                    {/each}
                                {/if} -->
                            </div>
                        {/if}
                        {#if $userSettings.isArtistsToolkitEnabled}
                            <item
                                class:selected={$uiView === "your-music"}
                                on:click={() => {
                                    $uiView = "your-music";
                                    $isTagCloudOpen = false;
                                    $selectedPlaylistFile = null;
                                    $selectedSmartQuery = null;
                                }}
                            >
                                <Icon
                                    icon="mdi:music-clef-treble"
                                    size={15}
                                    color={$uiView === "your-music"
                                        ? $currentThemeObject["accent"]
                                        : "currentColor"}
                                />{$LL.sidebar.artistsToolkit()}</item
                            >
                        {/if}
                        <item
                            class:selected={$uiView === "internet-archive"}
                            on:click={() => {
                                $selectedPlaylistFile = null;
                                $selectedSmartQuery = null;
                                $uiView = "internet-archive";
                            }}
                        >
                            <Icon
                                icon="fe:music"
                                size={15}
                                color={$uiView === "internet-archive"
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />Internet Archive</item
                        >
                        <item
                            class:selected={$uiView === "map"}
                            on:click={() => {
                                $selectedPlaylistFile = null;
                                $selectedSmartQuery = null;
                                $query.orderBy = $query.libraryOrderBy;
                                $uiView = "map";
                            }}
                        >
                            <Icon
                                icon="mdi:map"
                                size={15}
                                color={$uiView === "map"
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.map()}</item
                        >
                        <item
                            class:selected={$uiView === "analytics"}
                            on:click={() => {
                                $selectedPlaylistFile = null;
                                $selectedSmartQuery = null;
                                $query.orderBy = $query.libraryOrderBy;
                                $uiView = "analytics";
                            }}
                        >
                            <Icon
                                icon="gridicons:line-graph"
                                size={15}
                                color={$uiView === "analytics"
                                    ? $currentThemeObject["accent"]
                                    : "currentColor"}
                            />{$LL.sidebar.stats()}</item
                        >
                    </items>
                </menu>
            </div>
            <div
                bind:this={scrollbar}
                class="scrollbar"
                class:show={isScrollbarVisible}
                style="transform: translateY({menuScrollPos}px);"
            />
        </div>
    </div>

    {#if showPlaylistMenu}
        <div class="playlist-menu">
            <Menu
                x={menuX}
                y={menuY}
                onClickOutside={() => {
                    showPlaylistMenu = false;
                    isConfirmingPlaylistDelete = false;
                }}
                fixed
            >
                <MenuOption
                    isDestructive={true}
                    isConfirming={isConfirmingPlaylistDelete}
                    onClick={deletePlaylist}
                    text="Delete playlist"
                    confirmText="Click again to confirm"
                />
                <MenuOption
                    onClick={() => {
                        updatedPlaylistName = playlistToEdit.title;
                        isRenamingPlaylist = true;
                        showPlaylistMenu = false;
                    }}
                    text="Rename playlist"
                />
            </Menu>
        </div>
    {/if}

    {#if showSmartPlaylistMenu}
        <div class="playlist-menu">
            <Menu
                x={menuX}
                y={menuY}
                onClickOutside={() => {
                    showSmartPlaylistMenu = false;
                    isConfirmingSmartPlaylistDelete = false;
                }}
                fixed
            >
                <MenuOption
                    isDestructive={true}
                    isConfirming={isConfirmingSmartPlaylistDelete}
                    onClick={deleteSmartPlaylist}
                    text="Delete smart playlist"
                    confirmText="Click again to confirm"
                />
                <MenuOption
                    onClick={() => {
                        updatedSmartPlaylistName = smartPlaylistToEdit.name;
                        isRenamingSmartPlaylist = true;
                        showSmartPlaylistMenu = false;
                    }}
                    text="Rename playlist"
                />
            </Menu>
        </div>
    {/if}

    {#if $currentIAFile && $isIAPlaying}
        <div class="ia-mode" transition:fade={{ duration: 200 }}>
            <p>
                {@html $LL.sidebar.iaMode()}
            </p>
        </div>
    {/if}
    <div class="track-info">
        <!-- <hr /> -->

        <div class="track-info-content">
            <!-- svelte-ignore a11y-mouse-events-have-key-events -->

            {#if !$isMiniPlayer}
                <div
                    class="sidebar-toggle"
                    class:visible={$isSidebarOpen}
                    use:tippy={{
                        content: "Toggle the sidebar.",
                        placement: "right",
                    }}
                >
                    <Icon
                        icon="tabler:layout-sidebar-left-collapse"
                        size={22}
                        onClick={(e) => {
                            $isSidebarOpen = false;
                            $sidebarManuallyOpened = false;
                            $sidebarTogglePos = { x: e.clientX, y: e.clientY };
                        }}
                    />
                </div>
            {/if}

            <!-- svelte-ignore a11y-mouse-events-have-key-events -->
            <div
                bind:this={miniToggleBtn}
                class="mini-toggle"
                class:hovered={isMiniToggleHovered}
                on:mouseover={onMiniToggleMouseOver}
                on:mouseout={onMiniToggleMouseOut}
                use:tippy={{
                    theme: $isMiniPlayer ? "hidden" : "",
                    content: "Toggle the mini player.",
                    placement: "right",
                }}
            >
                <Icon
                    icon={$isMiniPlayer
                        ? "gg:arrows-expand-up-right"
                        : "gg:arrows-expand-down-left"}
                    onClick={() => toggleMiniPlayer()}
                    boxed
                />
            </div>
            <img alt="cd gif" class="cd-gif" src="images/cd6.gif" />

            <div class="info">
                {#if song}
                    {#if sidebarWidth && displayTitle}
                        <div
                            class="marquee-container"
                            on:mousedown={() => {
                                $draggedSongs = [song];
                            }}
                        >
                            <canvas
                                class="show"
                                bind:this={canvas}
                                width={sidebarWidth * 2.5}
                                height="50"
                            ></canvas>
                        </div>
                    {/if}
                    {#if artist}
                        <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                        <p
                            class="artist"
                            on:click={() => {
                                $isWikiOpen = !$isWikiOpen;
                            }}
                            use:optionalTippy={{
                                show: !$isMiniPlayer,
                                content: $LL.sidebar.openWikiTooltip({
                                    artist,
                                }),
                                placement: "right",
                            }}
                        >
                            {artist}
                        </p>
                    {/if}
                    {#if !title && !album && !artist}
                        <button
                            class="add-metadata-btn"
                            on:click={openTrackInfo}
                            >{$LL.sidebar.addMetadataHint()}</button
                        >
                    {/if}
                    {#if album}
                        <small>{album}</small>
                    {/if}
                {:else}
                    <p class="is-placeholder">{$LL.sidebar.takeControl()}</p>
                {/if}

                {#if codec}
                    <div class="file" class:empty={!title && !album && !artist}>
                        <p>{codec}</p>
                        <!-- {#if bitrate}<p>{bitrate} bit</p>{/if} -->
                        <p>{(Number(sampleRate) / 1000).toFixed(1)} Khz</p>
                        <p class="with-icon">
                            <span>
                                <Icon
                                    icon={stereo ? "fad:stereo" : "fad:mono"}
                                    size={12}
                                /></span
                            >{stereo ? "stereo" : "mono"}
                        </p>
                    </div>
                {/if}
            </div>
        </div>
    </div>

    {#if song}
        <div class="artwork-container">
            <div class="artwork-frame">
                <canvas
                    class="artwork"
                    bind:this={artworkCanvas}
                    width={210}
                    height={210}
                />
            </div>
        </div>
    {/if}
    <div class="bottom" data-tauri-drag-region>
        <div class="seekbar">
            <Seekbar
                {duration}
                onSeek={(time) => seekTime.set(time)}
                playerTime={$playerTime}
            />
            <div
                class="time-controls"
                class:speed-control-expanded={isPlaybackSpeedControlOpen}
            >
                <p class="elapsed-time">
                    <span class="elapsed">{elapsedTime} </span>
                </p>
                <PlaybackSpeed bind:selected={isPlaybackSpeedControlOpen} />
                <p class="elapsed-time">
                    <span class="elapsed">{durationText} </span>
                </p>
            </div>
            <transport>
                <Icon
                    class="transport-side shuffle {$isShuffleEnabled
                        ? 'active'
                        : 'inactive'}"
                    icon="ph:shuffle-bold"
                    onClick={() => {
                        $isShuffleEnabled = !$isShuffleEnabled;
                    }}
                />
                <Icon
                    class="transport-middle"
                    icon="fe:backward"
                    size={36}
                    disabled={$current.index <= 0}
                    onClick={() => audioPlayer.playPrevious()}
                />
                <Icon
                    class="transport-middle"
                    size={42}
                    onClick={() => audioPlayer.togglePlay()}
                    icon={$isPlaying ? "fe:pause" : "fe:play"}
                />
                <Icon
                    class="transport-middle"
                    size={36}
                    icon="fe:forward"
                    disabled={$queue.length === 0 ||
                        $current.index === $queue?.length - 1}
                    onClick={() => audioPlayer.playNext()}
                />
                <Icon
                    class="transport-side favourite {$current.song?.isFavourite
                        ? 'active'
                        : 'inactive'}"
                    icon={$current.song?.isFavourite
                        ? "clarity:heart-solid"
                        : "clarity:heart-line"}
                    onClick={() => {
                        favouriteCurrentSong();
                    }}
                />
            </transport>

            <div class="other-controls">
                <div class="track-info-icon">
                    <Icon
                        icon="mdi:information"
                        onClick={() => {
                            $rightClickedTrack = song;
                            $popupOpen = "track-info";
                        }}
                    />
                </div>

                <VolumeSlider />

                <div
                    class="visualizer-icon"
                    use:tippy={{
                        content: "waveform, loop region, marker editor",
                        placement: "top",
                    }}
                >
                    <Icon
                        class={$isWaveformOpen ? "active" : "inactive"}
                        icon="ph:wave-sine-duotone"
                        onClick={() => ($isWaveformOpen = !$isWaveformOpen)}
                    />
                </div>
            </div>
        </div>
    </div></sidebar
>

<style lang="scss">
    $mini_y_breakpoint: 460px;
    $xsmall_y_breakpoint: 320px;
    $sidebar_primary_color: transparent;
    $sidebar_secondary_color: transparent;

    :global {
        sidebar {
            .transport-middle {
                color: var(--transport-control) !important;

                &:hover {
                    color: var(--transport-control-hover) !important;
                }
            }

            .transport-side {
                &.favourite.active {
                    color: var(--transport-favorite);

                    &:hover {
                        color: var(--transport-favorite-hover);
                    }
                }

                &.shuffle.active {
                    color: var(--transport-shuffle);

                    &:hover {
                        color: var(--transport-shuffle-hover);
                    }
                }
            }

            .visualizer-icon {
                .active {
                    color: var(--icon-tertiary) !important;

                    &:hover {
                        color: var(--icon-tertiary-hover) !important;
                    }
                }
            }
        }
    }

    sidebar {
        position: relative;
        display: grid;
        grid-template-rows: minmax(198px, 1fr) auto 1fr; // top, menu, track info, empty space (bottom stuff is absolute)
        flex-direction: column;
        justify-content: flex-end;
        align-items: flex-end;
        height: 100vh;
        max-width: 210px;
        min-width: 210px;
        /* border-right: 1px solid #ececec1c; */
        background-color: $sidebar_primary_color;
    }

    hr {
        width: 100%;
        border-top: 1px solid rgba(141, 139, 139, 0.077);
        border-bottom: none;
        border-left: none;
        border-right: none;
    }

    .menu-outer {
        height: 100%;
        position: relative;
        overflow: hidden;

        &.top-shadow {
            mask-image: linear-gradient(
                to bottom,
                transparent 0%,
                black 18%,
                black 100%
            );
        }
        &.bottom-shadow {
            mask-image: linear-gradient(
                to bottom,
                black 0%,
                black 85%,
                transparent 100%
            );
        }
        &.top-shadow.bottom-shadow {
            mask-image: linear-gradient(
                to bottom,
                transparent 0%,
                black 15%,
                black 85%,
                transparent 100%
            );
        }

        &.top-border {
            border-top: 0.7px solid #ffffff12;
        }

        .top-shadow {
            display: none;
            pointer-events: none;
            background: linear-gradient(
                to bottom,
                hsl(320, 4.92%, 11.96%) 0%,
                hsla(320, 4.92%, 11.96%, 0.988) 2.6%,
                hsla(320, 4.92%, 11.96%, 0.952) 5.8%,
                hsla(320, 4.92%, 11.96%, 0.898) 9.7%,
                hsla(320, 4.92%, 11.96%, 0.828) 14.3%,
                hsla(320, 4.92%, 11.96%, 0.745) 19.5%,
                hsla(320, 4.92%, 11.96%, 0.654) 25.3%,
                hsla(320, 4.92%, 11.96%, 0.557) 31.6%,
                hsla(320, 4.92%, 11.96%, 0.458) 38.5%,
                hsla(320, 4.92%, 11.96%, 0.361) 45.9%,
                hsla(320, 4.92%, 11.96%, 0.268) 53.9%,
                hsla(320, 4.92%, 11.96%, 0.184) 62.2%,
                hsla(320, 4.92%, 11.96%, 0.112) 71.1%,
                hsla(320, 4.92%, 11.96%, 0.055) 80.3%,
                hsla(320, 4.92%, 11.96%, 0.016) 90%,
                hsla(320, 4.92%, 11.96%, 0) 100%
            );
            height: 40px;
            width: 100%;
            position: absolute;
            top: 0;
            right: 0;
            left: 0;
            z-index: 4;
        }
        .bottom-shadow {
            display: none;
            pointer-events: none;
            background: linear-gradient(
                to top,
                hsl(320, 4.92%, 11.96%) 0%,
                hsla(320, 4.92%, 11.96%, 0.988) 2.6%,
                hsla(320, 4.92%, 11.96%, 0.952) 5.8%,
                hsla(320, 4.92%, 11.96%, 0.898) 9.7%,
                hsla(320, 4.92%, 11.96%, 0.828) 14.3%,
                hsla(320, 4.92%, 11.96%, 0.745) 19.5%,
                hsla(320, 4.92%, 11.96%, 0.654) 25.3%,
                hsla(320, 4.92%, 11.96%, 0.557) 31.6%,
                hsla(320, 4.92%, 11.96%, 0.458) 38.5%,
                hsla(320, 4.92%, 11.96%, 0.361) 45.9%,
                hsla(320, 4.92%, 11.96%, 0.268) 53.9%,
                hsla(320, 4.92%, 11.96%, 0.184) 62.2%,
                hsla(320, 4.92%, 11.96%, 0.112) 71.1%,
                hsla(320, 4.92%, 11.96%, 0.055) 80.3%,
                hsla(320, 4.92%, 11.96%, 0.016) 90%,
                hsla(320, 4.92%, 11.96%, 0) 100%
            );
            height: 40px;
            width: 100%;
            position: absolute;
            bottom: 0;
            right: 0;
            left: 0;
            z-index: 4;
        }
    }

    .menu-inner {
        height: 100%;
        overflow: scroll;
        position: relative;
        box-sizing: border-box;
        /* hide scrollbar */
        -ms-overflow-style: none;
        scrollbar-width: none;
        background-color: $sidebar_secondary_color;
        &::-webkit-scrollbar {
            /* hide scrollbar */
            display: none;
        }
    }

    .scrollbar {
        position: absolute;
        right: 2px;
        top: 10px;
        min-height: 70px;
        width: 4px;
        border-radius: 4px;
        background-color: #474747;
        transition: opacity 0.2s ease-in-out;
        opacity: 0;

        &.show {
            opacity: 1;
        }
    }

    menu {
        width: 100%;
        margin: 0;
        position: relative;
        user-select: none;
        padding: 0 0.5em 0.5em 0.5em;
        margin-block-start: 0;
        &::-webkit-scrollbar {
            /* hide scrollbar */
            display: none;
        }
        items {
            display: flex;
            flex-direction: column;
            border-radius: 3px;
            gap: 2px;
        }

        item {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            text-transform: uppercase;
            text-align: left;
            width: max-content;
            padding: 0.17em 0.5em 0.17em 0.5em;
            font-size: 12px;
            font-weight: 600;
            letter-spacing: 0.4px;
            color: var(--text-inactive, initial);
            width: 100%;
            border-radius: 3px;
            box-sizing: border-box;
            border: 1px solid transparent;

            cursor: default;
            &.selected {
                color: var(--text-active, initial);
                font-weight: bold;
                .chevron {
                    visibility: visible;
                }
            }

            &:hover:not(.selected) {
                background-color: var(--sidebar-node-inactive-hover-bg);
                color: var(--sidebar-node-inactive-hover-text);
                opacity: var(--sidebar-node-inactive-hover-opacity);

                .chevron {
                    visibility: visible;
                }
            }

            &:not(.selected) {
                &:active {
                    color: var(--text-secondary, initial);
                }
            }

            .chevron {
                visibility: hidden;
                &.expanded {
                    visibility: visible;
                    transform: rotate(180deg);
                }
            }
        }
    }

    .top {
        width: 100%;
        height: 100%;
        position: sticky;
        overflow: hidden;
        padding-top: 2em;
        /* border-bottom-right-radius: 5px; */
        display: flex;
        flex-direction: column;
        /* background-color: $sidebar_secondary_color; */
        background-color: $sidebar_secondary_color;
        top: 0;
        z-index: 3;
        transition: height 1s ease-in-out;
        border-bottom: 0.7px solid var(--panel-separator);

        .top-header {
            /* height: 80px; */
            position: sticky;
            top: 0;
        }

        .app-title {
            font-family: "2Peas";
            width: fit-content;
            font-size: 1.7em;
            position: absolute;
            top: 0;
            right: 1em;
            user-select: none;
            margin: 0.3em 0;
            /* transition: height 1s ease-in-out; */
            color: var(--header-text);
            opacity: var(--header-opacity);
            cursor: default;
            &:hover {
                opacity: 0.5;
            }
        }
    }

    .search-container {
        padding: 1em;
        width: 100%;
        position: relative;
        background-color: $sidebar_secondary_color;
        user-select: none;

        .search {
            margin: 0;
            width: 100%;
            height: 30px;
            border-radius: 3px;
            padding-left: 5px;
            font-size: 13px;
            color: var(--text-active, initial);
            border: 1px solid
                color-mix(in srgb, var(--inverse) 80%, transparent);
            backdrop-filter: blur(8px);
            z-index: 10;
            background-color: transparent;
            &::placeholder {
                color: var(--text-inactive, initial);
            }
            &:focus {
                outline: var(--input-focus-outline);
                background-color: var(--sidebar-search-focus-bg);
                &::placeholder {
                    color: var(--text-inactive, initial);
                }
            }
        }
        .search-icon {
            position: absolute;
            right: 15px;
            top: 0px;
            bottom: 0;
            height: fit-content;
            padding: 5px;
            margin: auto 0;
        }
    }

    .track-info {
        width: 100%;
        height: 210px;
        /* height: 150px; */
        /* min-height: 150px; */
        width: 100%;
        position: sticky;
        /* top: 110px; */
        top: 0px;
        cursor: default;
        user-select: none;
        pointer-events: none;
        z-index: 2;
        overflow: hidden;
    }
    .ia-mode {
        position: absolute;
        left: 0;
        right: 0;
        bottom: 0;
        height: 520px;
        background-color: var(--sidebar-player-disabled-bg);
        backdrop-filter: blur(8px);
        color: var(--sidebar-player-disabled-text);
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 1em;
        z-index: 20;
    }

    .track-info-content {
        width: 100%;
        height: fit-content;
        position: sticky;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;

        .sidebar-toggle {
            position: absolute;
            top: 14px;
            left: 12px;
            padding: 3px;
            pointer-events: all;
        }
    }

    .mini-toggle {
        pointer-events: all;
        font-size: 20px;

        position: absolute;
        top: 10px;
        right: 8px;
        padding: 3px;

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }
    }
    .info {
        background-color: $sidebar_primary_color;
        color: var(--text, initial);
        width: 100%;
        padding: 0.5em 0.3em;

        .marquee-container {
            height: 20px;
            width: 100%;
            pointer-events: all;
            margin-bottom: 0.3em;
            cursor: grab;
            mask-image: linear-gradient(
                to right,
                transparent 0%,
                white 15%,
                white 85%,
                transparent 100%
            );

            &:hover {
                background-color: var(--sidebar-info-title-hover-bg);
                border-radius: 5px;
                mask-image: none;
                border: 1px dashed var(--sidebar-info-title-hover-border);
            }

            canvas {
                width: 100%;
                height: 100%;
                visibility: hidden;

                &.show {
                    visibility: visible;
                }
            }
        }

        .artist {
            white-space: nowrap;
            font-weight: 500;
            font-size: 0.9em;
            opacity: 0.9;
            width: fit-content;
            margin: auto;
            padding: 0 5px;
            z-index: 1;
            text-overflow: ellipsis;
            overflow: hidden;
            pointer-events: all;
            @media screen and (min-width: 211px) and (min-height: 211px) {
                &:hover {
                    background-color: var(--sidebar-info-artist-hover-bg);
                    border-radius: 5px;
                }
                &:active {
                    background-color: var(--sidebar-info-artist-active-bg);
                }
            }
        }
        .title {
            white-space: nowrap;
            font-weight: bold;
            visibility: hidden;

            mask-image: linear-gradient(
                to right,
                transparent 0%,
                $sidebar_secondary_color 10%,
                $sidebar_secondary_color 90%,
                transparent 100%
            );
            &.show {
                visibility: visible;
            }
        }
        small {
            white-space: nowrap;
            opacity: 0.7;
        }
        * {
            margin: 0;
        }

        .is-placeholder {
            font-family: "Snake", Courier, monospace;
            font-size: 2em;
            line-height: 1.2em;
            margin: 0 1em;
        }
        .add-metadata-btn {
            border-radius: 4px;
            font-size: 13px;
            color: rgb(166, 140, 207);
            margin-top: 5px;
            height: 25px;
            padding: 2px 10px;
            pointer-events: all;
        }
    }

    .file {
        /* position: absolute; */
        bottom: 0px;
        /* background-color: $sidebar_secondary_color; */

        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
        padding: 4px 0;
        user-select: none;
        pointer-events: none;
        width: 100%;
        color: var(--text-secondary);

        p {
            background-color: color-mix(
                in srgb,
                var(--inverse) 5%,
                transparent
            );
            padding: 0em 0.6em;
            margin: 0;
            border-radius: 2px;
            font-size: 0.62em;
            line-height: 1.5em;
            max-height: 15px;
            font-weight: 600;
            border: 1px solid
                color-mix(in srgb, var(--type-bw-inverse) 10%, transparent);

            /* border-bottom: 1px dashed rgb(49, 49, 49); */
            font-family: monospace;
            text-transform: uppercase;
            display: inline;
            &.with-icon {
                padding: 0em 0.6em 0em 0.3em;
                display: inline-flex;
                align-items: center;
                gap: 2px;
            }
        }
    }
    .spectrum {
    }

    .cd-gif {
        margin-top: 1.3em;
        margin-bottom: 0.5em;
        width: 25px;
        height: auto;
        margin-left: 10px;
        align-self: center;
        z-index: 0;
    }

    .artwork-container {
        padding: 0em;
        width: 100%;
        height: 210px;
        position: sticky;
        bottom: 140px;
        margin: auto;
        pointer-events: none;
        opacity: 1;
        box-sizing: content-box;
        z-index: 1;

        .artwork-frame {
            width: 100%;
            height: 100%;
            box-sizing: border-box;
            display: flex;
            align-items: center;
            justify-content: center;
            .artwork {
                width: 100%;
                height: 100%;
                object-fit: cover;
            }
            .artwork-placeholder {
                opacity: 0.8;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 1em;
                z-index: -1;
                img {
                    width: 80%;
                }
            }
        }
    }

    .bottom {
        width: 100%;
        position: absolute;
        bottom: 0;
        z-index: 3;
        height: 140px;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        /* background-color: #242026bc; */
    }

    @keyframes marquee {
        0% {
            transform: translate(0%, 0);
        }
        100% {
            transform: translate(-100%, 0);
        }
    }

    img {
        width: 100%;
        height: 100%;
    }

    .seekbar {
        width: 100%;
        padding: 0 1em;
        display: flex;
        flex-direction: column;

        .time-controls {
            width: 100%;
            display: flex;
            justify-content: space-between;
            transition: all 0.3s ease-in-out;

            &.speed-control-expanded {
                .elapsed-time {
                    flex-shrink: 1;
                }
            }
        }

        .elapsed-time {
            opacity: 0.5;
            font-size: 12px;
            margin: 0;
            padding: 0;
            width: fit-content;
            border-radius: 5px;
            user-select: none;
            letter-spacing: 0.4px;
            .elapsed {
                font-weight: 500;
                color: var(--text-active);
            }
        }
    }

    transport {
        padding-bottom: 1em;
        width: 100%;
        z-index: 2;
        display: flex;
        justify-content: space-between;
    }

    .other-controls {
        padding: 0 0 1em;
        width: 100%;
        display: flex;
        align-items: center;
        gap: 10px;

        .visualizer-icon {
            position: relative;
        }
        .track-info-icon,
        .visualizer-icon {
            display: flex;
        }
        @media only screen and (max-height: 210px) and (max-width: 210px) {
            padding: 0 2em 1em;
            .track-info-icon,
            .visualizer-icon {
                display: none;
            }
        }
    }

    .playlist-menu {
        position: fixed;
        z-index: 22;
    }

    .playlists {
        display: flex;
        flex-direction: column;

        .playlist {
            display: flex;
            flex-direction: row;
            height: 28px;
            align-items: center;
            margin: 0 0 0 1.4em;
            padding: 0.5em 0;
            position: relative;

            &.dragover {
                background-color: var(--sidebar-item-drag-bg);
                border-radius: 5px;
            }

            &.hover {
                border-radius: 5px;
                background-color: var(--sidebar-item-hover-bg);
                p {
                    color: var(--sidebar-item-hover-text);
                }
                .playlist-options {
                    display: flex;
                }
            }

            &.selected {
                p {
                    color: var(--text-active);
                    font-weight: bold;
                }

                &::before {
                    content: "";
                    width: 2px;
                    left: -4px;
                    position: absolute;
                    height: 70%;
                    background-color: var(--sidebar-item-selected-pipe-bg);
                    border-radius: 4px;
                }
            }
        }

        .playlist-options {
            display: none;

            &.visible {
                display: flex;
            }
        }

        p {
            flex-grow: 3;
            display: inline-block;
            text-align: left;
            text-align: left;
            font-size: 0.9em;
            padding: 0 0.5em;
            overflow: hidden;
            white-space: nowrap;
            text-overflow: ellipsis;
            letter-spacing: 0.2px;
            color: var(--text-inactive);
            margin: 0;
            cursor: default;
            pointer-events: none;
        }

        .new-playlist {
            display: flex;
            margin: 0 0 0 1.6em;
        }
    }

    @media only screen and (max-height: 385px) {
        .top {
            padding-top: 0em;
            border-bottom: none;
            visibility: hidden;
        }
    }

    sidebar.hovered {
        @media only screen and (max-height: 210px) and (max-width: 210px) {
            .track-info,
            .bottom,
            .mini-toggle {
                opacity: 1 !important;
            }

            .artwork-container {
                opacity: 0.1 !important;
            }
        }
    }

    .empty {
        grid-template-rows: 1fr auto 140px;
        .track-info {
            background: none;
        }

        // COVER INFO OVER ARTWORK
        @media only screen and (max-height: 548px) {
            .track-info {
                backdrop-filter: blur(1px);
            }
        }

        @media only screen and (max-height: 500px) {
            grid-template-rows: 1fr auto 140px;
            .top-header,
            .file {
                display: none;
            }
        }

        @media only screen and (max-height: 210px) and (max-width: 210px) {
            .top {
                padding-top: 0em;
            }
            .is-placeholder {
                display: none;
            }
            .track-info {
                width: 100vw;
            }
        }
    }

    .has-current-song {
        @media only screen and (min-height: 821px) {
            grid-template-rows: 1fr auto 310px;
        }
        @media only screen and (max-height: 870px) {
            .app-title {
                display: none;
            }
        }

        .file.empty {
            @media only screen and (max-height: 765px) {
                display: none;
            }
        }

        @media only screen and (max-height: 785px) {
            .file:not(.empty) {
                display: none;
            }
        }

        @media only screen and (max-height: 700px) {
            .top-header {
                display: none;
            }
        }
        @media only screen and (max-height: 660px) {
            /* grid-template-rows: 1fr 210px 1fr; */
            .track-info {
                background: linear-gradient(
                    to bottom,
                    hsla(240, 10.71%, 10.98%, 0.75) 0%,
                    hsla(240, 10.71%, 10.98%, 0.74) 8.3%,
                    hsla(240, 10.71%, 10.98%, 0.714) 16.5%,
                    hsla(240, 10.71%, 10.98%, 0.672) 24.4%,
                    hsla(240, 10.71%, 10.98%, 0.618) 32.2%,
                    hsla(240, 10.71%, 10.98%, 0.556) 39.7%,
                    hsla(240, 10.71%, 10.98%, 0.486) 47%,
                    hsla(240, 10.71%, 10.98%, 0.412) 54.1%,
                    hsla(240, 10.71%, 10.98%, 0.338) 60.9%,
                    hsla(240, 10.71%, 10.98%, 0.264) 67.4%,
                    hsla(240, 10.71%, 10.98%, 0.194) 73.6%,
                    hsla(240, 10.71%, 10.98%, 0.132) 79.6%,
                    hsla(240, 10.71%, 10.98%, 0.078) 85.2%,
                    hsla(240, 10.71%, 10.98%, 0.036) 90.5%,
                    hsla(240, 10.71%, 10.98%, 0.01) 95.4%,
                    hsla(240, 10.71%, 10.98%, 0) 100%
                );
            }
        }

        // COVER INFO OVER ARTWORK
        @media only screen and (max-height: 548px) {
            grid-template-rows: 1fr auto 140px;

            .track-info {
                border-top: none;
                backdrop-filter: blur(1px);
            }
        }

        @media only screen and (max-height: 220px) {
            .cd-gif,
            .search-container {
                display: none;
            }
            .track-info,
            .info {
                background-color: transparent;
            }
            .artwork-container {
                display: flex !important;
                top: 0;
                bottom: 0;
                position: absolute;
                height: 100%;
            }
            .track-info {
                padding: 0 0.4em;
            }
            .track-info,
            .bottom,
            .mini-toggle {
                /* top: 10px; */
                transition: opacity 0.2s ease-in;
            }

            .add-metadata-btn {
                display: none;
            }
        }
        @media only screen and (max-height: 260px) {
            .track-info {
                width: 210px;
                small {
                    display: none;
                }
            }

            .add-metadata-btn {
                display: none;
            }
        }

        @media only screen and (max-height: 210px) and (min-width: 211px) {
        }

        // MINI PLAYER ONLY
        @media only screen and (max-height: 210px) and (max-width: 210px) {
            .top {
                padding-top: 0em;
            }
            .track-info,
            .bottom,
            .mini-toggle {
                opacity: 0 !important;
            }
            .mini-toggle {
                top: 0px;
                right: 0px;
            }
            .track-info-content {
                margin-top: 1.9em;
                margin-bottom: 1em;
            }

            .other-controls {
                padding: 0.5em 2em 1em;
            }

            .seekbar {
                padding: 0.5em 1em;
                margin-bottom: -0.5em;
            }

            .artwork-container {
                opacity: 1 !important;
            }
        }

        @media only screen and (max-height: 260px) and (max-width: 211px) {
            .mini-toggle {
                position: fixed;
            }
        }
    }
</style>
