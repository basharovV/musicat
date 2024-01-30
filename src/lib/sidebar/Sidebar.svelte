<script lang="ts">
    import { window as tauriWindow } from "@tauri-apps/api";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import {
        appWindow,
        currentMonitor,
        LogicalSize,
        PhysicalPosition
    } from "@tauri-apps/api/window";
    import { liveQuery } from "dexie";
    import hotkeys from "hotkeys-js";
    import { throttle } from "lodash-es";
    import * as musicMetadata from "music-metadata-browser";
    import { onMount } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { flip } from "svelte/animate";
    import { cubicInOut } from "svelte/easing";
    import { fade, fly } from "svelte/transition";
    import type { Playlist } from "../../App";
    import { db } from "../../data/db";
    import { lookForArt } from "../../data/LibraryImporter";
    import {
        currentSong,
        currentSongArtworkSrc,
        currentSongIdx,
        draggedSongs,
        isFindFocused,
        isFullScreenVisualiser,
        isInfoPopupOpen,
        isMiniPlayer,
        isPlaying,
        isShuffleEnabled,
        isSmartQueryBuilderOpen,
        isSmartQueryUiOpen,
        isTrackInfoPopupOpen,
        os,
        playlist,
        queriedSongs,
        query,
        rightClickedTrack,
        selectedPlaylistId,
        shouldFocusFind,
        singleKeyShortcutsEnabled,
        smartQueryInitiator,
        uiView,
        userSettings,
        volume
    } from "../../data/store";
    import audioPlayer from "../player/AudioPlayer";
    import Input from "../ui/Input.svelte";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Seekbar from "./Seekbar.svelte";
    import SpectrumAnalyzer from "../player/SpectrumAnalyzer.svelte";
    import "../tippy.css";
    import Icon from "../ui/Icon.svelte";

    // Env
    let isArtistToolkitEnabled = true;

    // What to show in the sidebar
    let title;
    let fileName;
    $: displayTitle = title ?? fileName;
    let artist;
    let album;
    let artworkFormat;
    let artworkBuffer: Buffer;
    let artworkSrc;
    let codec;

    let duration;

    $: if (codec === "MPEG 1 Layer 3") {
        codec = "MP3";
    }

    let bitrate;
    let sampleRate;
    let stereo;

    // Shortcuts
    hotkeys("space", function (event, handler) {
        // Prevent the default refresh event under WINDOWS system
        event.preventDefault();
        audioPlayer.togglePlay();
    });

    currentSong.subscribe(async (song) => {
        if (song) {
            const metadata = await musicMetadata.fetchFromUrl(
                convertFileSrc(song.path)
            );
            console.log("metadata", metadata);
            title = metadata.common?.title?.length
                ? metadata.common.title
                : null;
            fileName = song.file;
            artist = metadata.common.artist;
            album = metadata.common.album;
            codec = metadata.format.codec;
            stereo = metadata.format.numberOfChannels === 2;
            bitrate = metadata.format.bitsPerSample;
            sampleRate = metadata.format.sampleRate;
            duration = metadata.format.duration;
            if (metadata.common.picture?.length) {
                artworkFormat = metadata.common.picture[0].format;
                artworkBuffer = metadata.common.picture[0].data;
                artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                    "base64"
                )}`;
                console.log("artworkSrc", artworkSrc);
                $currentSongArtworkSrc = {
                    src: artworkSrc,
                    format: artworkFormat,
                    size: {
                        width: metadata.common.picture[0]["width"],
                        height: metadata.common.picture[0]["height"]
                    }
                };
            } else {
                artworkSrc = null;
                const artwork = await lookForArt(song.path, song.file);
                if (artwork) {
                    artworkSrc = artwork.artworkSrc;
                    artworkFormat = artwork.artworkFormat;
                    $currentSongArtworkSrc = {
                        src: artworkSrc,
                        format: artworkFormat,
                        size: {
                            width: 200,
                            height: 200
                        }
                    };
                }
            }
        }
    });

    function togglePlayPause() {
        if (!audioPlayer.getCurrentAudioFile().src) {
            audioPlayer.shouldPlay = true;
            $playlist = $queriedSongs;
        } else {
            audioPlayer.togglePlay();
        }
    }

    function playNext() {
        audioPlayer.playSong($queriedSongs[++$currentSongIdx]);
    }

    function playPrev() {
        if ($currentSongIdx === 1) {
            return;
        }
        audioPlayer.playSong($queriedSongs[--$currentSongIdx]);
    }

    function openTrackInfo() {
        if ($currentSong) {
            $rightClickedTrack = $currentSong;
            $isTrackInfoPopupOpen = true;
        }
    }

    function openInfoWindow() {
        // const webview = new WebviewWindow("theUniqueLabel", {
        //   url: "path/to/page.html",
        // });
        console.log("clicked");
        $isInfoPopupOpen = true;
    }
    $: {
        console.log("info popup:", $isInfoPopupOpen);
    }
    let height = 0;
    let width = 0;
    let hasDecorations = false;

    async function onResize() {
        // Check if is miniplayer mode
        height = window.innerHeight;
        width = window.innerWidth;
        hasDecorations = await tauriWindow.getCurrent().isDecorated();
        if (!$isMiniPlayer && height <= 220 && width <= 210) {
            $isMiniPlayer = true;
            console.log("setting to false");
            await tauriWindow.getCurrent().setDecorations(false);
        } else if ($isMiniPlayer && (height > 220 || width > 210)) {
            $isMiniPlayer = false;
            console.log("setting to true");
            await tauriWindow.getCurrent().setDecorations(true);
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

            await tauriWindow.getCurrent().hide();
            await tauriWindow.getCurrent().setSize(new LogicalSize(210, 210));
            const monitor = await currentMonitor();
            const windowSize = await tauriWindow.getCurrent().innerSize();
            switch ($userSettings.miniPlayerLocation) {
                case "bottom-left":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x + paddingPx,
                                monitor.position.y +
                                    monitor.size.height -
                                    windowSize.height -
                                    paddingPx
                            )
                        );
                    break;
                case "bottom-right":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x +
                                    monitor.size.width -
                                    windowSize.width -
                                    paddingPx,
                                monitor.position.y +
                                    monitor.size.height -
                                    windowSize.height -
                                    paddingPx
                            )
                        );
                    break;
                case "top-left":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x + paddingPx,
                                monitor.position.y +
                                    ($os === "Darwin"
                                        ? paddingPx + 40
                                        : paddingPx)
                            )
                        );
                    break;
                case "top-right":
                    await tauriWindow
                        .getCurrent()
                        .setPosition(
                            new PhysicalPosition(
                                monitor.position.x +
                                    monitor.size.width -
                                    windowSize.width -
                                    paddingPx,
                                monitor.position.y +
                                    ($os === "Darwin"
                                        ? paddingPx + 40
                                        : paddingPx)
                            )
                        );
                    break;
            }

            await tauriWindow.getCurrent().show();
            await tauriWindow.getCurrent().setAlwaysOnTop(true);
            isMiniPlayerHovered = false; // By default we want to show the pretty artwork
        } else {
            await tauriWindow.getCurrent().hide();
            if (widthToRestore && heightToRestore) {
                await tauriWindow
                    .getCurrent()
                    .setSize(new LogicalSize(widthToRestore, heightToRestore));
            } else {
                await tauriWindow
                    .getCurrent()
                    .setSize(new LogicalSize(1100, 750));
            }

            await tauriWindow.getCurrent().center();
            await tauriWindow.getCurrent().show();
            await tauriWindow.getCurrent().setAlwaysOnTop(false);
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
        console.log("scrollTop");
        // Check scroll area size, add shadows if necessary
        if (menuInnerScrollArea) {
            showMenuTopScrollShadow =
                menuInnerScrollArea.scrollTop > 0 &&
                menuInnerScrollArea.scrollHeight >
                    menuInnerScrollArea.clientHeight;
            showMenuBottomScrollShadow =
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
    }

    // Playlists

    $: playlists = liveQuery(async () => {
        return db.playlists.toArray();
    });

    let newPlaylistTitle = "";
    let updatedPlaylistName = ""; // also for renaming existing playlists
    let isPlaylistsExpanded = false;
    let showPlaylistMenu = false;
    let menuX = 0;
    let menuY = 0;
    let isConfirmingPlaylistDelete = false;
    let playlistToEdit: Playlist = null;
    let draggingOverPlaylist: Playlist = null;
    let hoveringOverPlaylistId: number = null;
    let isRenamingPlaylist = false;

    function onCreatePlaylist() {
        db.playlists.add({
            title: newPlaylistTitle,
            tracks: []
        });
        newPlaylistTitle = "";
    }

    async function deletePlaylist() {
        if (!isConfirmingPlaylistDelete) {
            isConfirmingPlaylistDelete = true;
            return;
        }
        await db.playlists.delete(playlistToEdit.id);
        showPlaylistMenu = false;
        isConfirmingPlaylistDelete = false;
    }

    async function onRenamePlaylist(playlist: Playlist) {
        playlist.title = updatedPlaylistName;
        await db.playlists.put(playlist);
        updatedPlaylistName = "";

        isRenamingPlaylist = false;
    }

    function onMouseOverPlaylist(playlist: Playlist) {
        if ($draggedSongs.length && draggingOverPlaylist?.id !== playlist?.id) {
            draggingOverPlaylist = playlist;
            hoveringOverPlaylistId = null;
        } else {
            hoveringOverPlaylistId = playlist?.id;
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

    async function onDropSongsToPlaylist(playlistId: string) {
        if ($draggedSongs.length) {
            const playlist = await db.playlists.get(playlistId);
            const songsToAdd = $draggedSongs.map((s) => s.id);
            console.log("dragged songs", $draggedSongs);
            console.log("playlist", songsToAdd);
            playlist.tracks = [...playlist.tracks, ...songsToAdd];

            console.log("playlist", playlist);
            await db.playlists.put(playlist);
            toast.success(
                `${
                    $draggedSongs.length > 1
                        ? $draggedSongs.length + " songs"
                        : $draggedSongs[0].title
                } added to ${playlist.title}`,
                {
                    position: "bottom-center"
                }
            );
            $draggedSongs = [];
        }
    }

    async function favouriteCurrentSong() {
        if (!$currentSong) return;
        await db.songs.update($currentSong, {
            isFavourite: !$currentSong.isFavourite
        });
        $currentSong = $currentSong;
    }
    let sidebar;
    let sidebarWidth = 0;

    let titleElement: HTMLParagraphElement;
    let isTitleOverflowing = false; // to show marquee

    onMount(() => {
        height = window.innerHeight;
        window.onresize = throttle(() => {
            onResize();
        }, 200);
        sidebarWidth = sidebar?.clientWidth;
        onResize(); // run once

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
    });

    let canvas: HTMLCanvasElement;

    $: if ($currentSong && canvas && sidebarWidth && displayTitle) {
        console.log("title", title);
        // Too early - song is changed, but not the title
        isTitleOverflowing =
            titleElement?.scrollWidth > titleElement?.clientWidth;

        resetMarquee();
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
        context.fillStyle = "white";
        let gap = 100;
        let textWidth = context.measureText(displayTitle).width;

        let x = (canvas.width - textWidth) / 2; // Initial x-coordinate for the text
        let x2 = x + context.measureText(displayTitle).width + gap;
        console.log("x", x, "x2", x2);
        let started = false;
        var lastFrameTime = 0;

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
                context.fillText(displayTitle, x, 25);
                context.fillText(displayTitle, x2, 25);

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
                    25
                );
            }
        }
        animate(0);
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<sidebar
    class:has-current-song={$currentSong}
    class:empty={!$currentSong}
    class:hovered={isMiniPlayerHovered}
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
        <div class="top-header" data-tauri-drag-region>
            <h1
                class="app-title"
                style={process.env.NODE_ENV === "development"
                    ? "opacity: 1"
                    : ""}
                on:click={openInfoWindow}
            >
                Musicat{process.env.NODE_ENV === "development" ? " 🚧" : ""}
            </h1>
            <div class="search-container">
                <input
                    bind:this={searchInput}
                    class="search"
                    id="search"
                    type="text"
                    autocomplete="off"
                    spellcheck="false"
                    placeholder="Search ({$os === 'Darwin'
                        ? 'Cmd + F'
                        : 'Ctrl + F'})"
                    bind:value={$query.query}
                    on:keydown={onSearchInputKeyDown}
                />
                <div class="search-icon">
                    <Icon icon="ion:search" color="#737373" />
                </div>
            </div>
        </div>

        <div
            class="menu-outer"
            class:top-border={showMenuTopScrollShadow}
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
                                !$selectedPlaylistId}
                            on:click={() => {
                                $isSmartQueryBuilderOpen = false;
                                $selectedPlaylistId = null;
                                $uiView = "library";
                            }}
                        >
                            <Icon
                                icon="fluent:library-20-filled"
                                size={15}
                                color={$uiView === "library" &&
                                !$selectedPlaylistId
                                    ? "#45fffcf3"
                                    : "currentColor"}
                            />Library</item
                        >
                        <item
                            class:selected={$uiView === "albums"}
                            on:click={() => {
                                $uiView = "albums";
                            }}
                        >
                            <Icon
                                icon="ic:round-album"
                                size={15}
                                color={$uiView === "albums"
                                    ? "#45fffcf3"
                                    : "currentColor"}
                            />Albums</item
                        >

                        <item
                            class:selected={$uiView === "library" &&
                                $selectedPlaylistId}
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
                                $selectedPlaylistId
                                    ? "#45fffcf3"
                                    : "currentColor"}
                            />Playlists
                            <div
                                class="chevron"
                                class:expanded={isPlaylistsExpanded}
                            >
                                <Icon icon="lucide:chevron-down" size={14} />
                            </div>
                        </item>

                        {#if isPlaylistsExpanded}
                            <div class="playlists">
                                {#if $playlists}
                                    {#each $playlists as playlist (playlist.id)}
                                        <div
                                            animate:flip={{
                                                duration: 300,
                                                easing: cubicInOut
                                            }}
                                            class="playlist"
                                            class:dragover={draggingOverPlaylist ===
                                                playlist}
                                            class:hover={hoveringOverPlaylistId ===
                                                playlist.id}
                                            class:selected={$selectedPlaylistId ===
                                                playlist.id}
                                            on:click={() => {
                                                $uiView = "playlists";
                                                $selectedPlaylistId =
                                                    playlist.id;
                                            }}
                                            on:mouseleave|preventDefault|stopPropagation={onMouseLeavePlaylist}
                                            on:mouseenter|preventDefault|stopPropagation={() =>
                                                onMouseOverPlaylist(playlist)}
                                            on:mouseup|preventDefault|stopPropagation={() =>
                                                onDropSongsToPlaylist(
                                                    playlist.id
                                                )}
                                        >
                                            {#if isRenamingPlaylist && playlistToEdit.id === playlist.id}
                                                <Input
                                                    bind:value={updatedPlaylistName}
                                                    onEnterPressed={() => {
                                                        onRenamePlaylist(
                                                            playlist
                                                        );
                                                    }}
                                                    fullWidth
                                                    minimal
                                                    autoFocus
                                                />
                                            {:else}
                                                <p>{playlist.title}</p>
                                            {/if}
                                            {#if isRenamingPlaylist && playlistToEdit.id === playlist.id}
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
                                                    class="playlist-menu"
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

                        {#if showPlaylistMenu}
                            <div class="menu">
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
                                            updatedPlaylistName =
                                                playlistToEdit.title;
                                            isRenamingPlaylist = true;
                                            showPlaylistMenu = false;
                                        }}
                                        text="Rename playlist"
                                    />
                                </Menu>
                            </div>
                        {/if}
                        <item
                            class:selected={$uiView === "smart-query"}
                            on:click={() => {
                                $smartQueryInitiator = "sidebar";
                                $uiView = "smart-query";
                            }}
                        >
                            <Icon
                                icon="fluent:search-20-filled"
                                size={15}
                                color={$uiView === "smart-query"
                                    ? "#45fffcf3"
                                    : "currentColor"}
                            />Smart Playlists</item
                        >
                        {#if isArtistToolkitEnabled}
                            <item
                                class:selected={$uiView === "your-music"}
                                on:click={() => {
                                    $uiView = "your-music";
                                }}
                            >
                                <Icon
                                    icon="mdi:music-clef-treble"
                                    size={15}
                                    color={$uiView === "your-music"
                                        ? "#45fffcf3"
                                        : "currentColor"}
                                />Artist's toolkit</item
                            >
                        {/if}
                        <item
                            class:selected={$uiView === "map"}
                            on:click={() => {
                                $uiView = "map";
                            }}
                        >
                            <Icon
                                icon="mdi:map"
                                size={15}
                                color={$uiView === "map"
                                    ? "#45fffcf3"
                                    : "currentColor"}
                            />Map</item
                        >
                        <item
                            class:selected={$uiView === "analytics"}
                            on:click={() => {
                                $uiView = "analytics";
                            }}
                        >
                            <Icon
                                icon="gridicons:line-graph"
                                size={15}
                                color={$uiView === "analytics"
                                    ? "#45fffcf3"
                                    : "currentColor"}
                            />Stats</item
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
    <div class="track-info">
        <!-- <hr /> -->

        <div class="track-info-content">
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
                    placement: "right"
                }}
            >
                <Icon
                    icon={$isMiniPlayer
                        ? "gg:arrows-expand-up-right"
                        : "gg:arrows-expand-down-left"}
                    onClick={() => toggleMiniPlayer()}
                />
            </div>
            <img alt="cd gif" class="cd-gif" src="images/cd6.gif" />

            <div class="info">
                {#if $currentSong}
                    {#if sidebarWidth && displayTitle}
                        <div class="marquee-container">
                            <canvas
                                class="show"
                                bind:this={canvas}
                                width={sidebarWidth * 2.5}
                                height="50"
                            ></canvas>
                        </div>
                    {/if}
                    {#if artist}
                        <p class="artist">{artist}</p>
                    {/if}
                    {#if !title && !album && !artist}
                        <button
                            class="add-metadata-btn"
                            on:click={openTrackInfo}>Add metadata</button
                        >
                    {/if}
                    {#if album}
                        <small>{album}</small>
                    {/if}
                {:else}
                    <p class="is-placeholder">Take control of your library</p>
                {/if}

                {#if codec}
                    <div class="file" class:empty={!title && !album && !artist}>
                        <p>{codec}</p>
                        {#if bitrate}<p>{bitrate} bit</p>{/if}
                        <p>{(Number(sampleRate) / 1000).toFixed(1)} Khz</p>
                        <p>{stereo ? "stereo" : "mono"}</p>
                    </div>
                {/if}
            </div>
        </div>
    </div>

    {#if $currentSong}
        <div class="artwork-container">
            <div class="artwork-frame">
                {#if artworkSrc && artworkFormat}
                    <img
                        alt="Artwork"
                        type={artworkFormat}
                        class="artwork"
                        src={artworkSrc}
                        async
                    />
                {:else}
                    <div class="artwork-placeholder">
                        <img alt="placeholder" src="icon.png" />
                        <!-- <iconify-icon icon="mdi:music-clef-treble" /> -->
                        <!-- <small>No art</small> -->
                    </div>
                {/if}
            </div>
        </div>
    {/if}
    <div class="bottom" data-tauri-drag-region>
        <div class="seekbar">
            <Seekbar {duration} />
        </div>
        <transport>
            <Icon
                class="transport-side"
                icon="ph:shuffle-bold"
                color={!$isShuffleEnabled ? "#606060" : "#e1ff00"}
                onClick={() => {
                    $isShuffleEnabled = !$isShuffleEnabled;
                }}
            />
            <Icon
                class="transport-middle"
                icon="fe:backward"
                size={36}
                disabled={$currentSongIdx === 0}
                onClick={() => audioPlayer.playPrevious()}
            />
            <Icon
                class="transport-middle"
                size={42}
                onClick={togglePlayPause}
                icon={$isPlaying ? "fe:pause" : "fe:play"}
            />
            <Icon
                class="transport-middle"
                size={36}
                icon="fe:forward"
                disabled={$currentSongIdx === $playlist?.length - 1}
                onClick={() => audioPlayer.playNext()}
            />
            <Icon
                class="transport-side favourite {$currentSong?.isFavourite
                    ? 'active'
                    : 'inactive'}"
                color={$currentSong?.isFavourite ? "#59cd7a" : "grey"}
                icon={$currentSong?.isFavourite
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
                        $rightClickedTrack = $currentSong;
                        $isTrackInfoPopupOpen = true;
                    }}
                    color="#474747"
                />
            </div>

            <div class="volume">
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={$volume}
                    class="slider"
                    id="myRange"
                />
            </div>

            <div class="visualizer-icon">
                <Icon
                    icon="ph:wave-sine-duotone"
                    onClick={() =>
                        ($isFullScreenVisualiser = !$isFullScreenVisualiser)}
                    color="#474747"
                />
            </div>
        </div>
    </div>
</sidebar>

<style lang="scss">
    $thumb_size: 22px;
    $mini_y_breakpoint: 460px;
    $xsmall_y_breakpoint: 320px;
    $sidebar_primary_color: transparent;
    $sidebar_secondary_color: #242026;
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
        border-right: 1px solid #ececec1c;
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

        &.top-border {
            border-top: 0.7px solid #ffffff12;
        }

        .top-shadow {
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
        padding: 1em;
        &::-webkit-scrollbar {
            /* hide scrollbar */
            display: none;
        }
        margin-block-start: 0;
        items {
            display: flex;
            flex-direction: column;
            border-radius: 3px;
            gap: 3px;
        }

        item {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            text-transform: uppercase;
            font-weight: bold;
            text-align: left;
            width: fit-content;
            padding: 0.3em 0.5em;
            font-size: 12px;
            letter-spacing: 0.2px;
            color: rgb(143, 144, 147);
            width: 100%;
            border-radius: 3px;
            box-sizing: border-box;
            border: 1px solid transparent;

            cursor: default;
            &.selected {
                color: white;
                iconify-icon {
                    color: #45fffcf3;
                }
                .chevron {
                    visibility: visible;
                }
            }

            &:hover:not(.selected) {
                opacity: 0.5;

                .chevron {
                    visibility: visible;
                }
            }

            &:not(.selected) {
                &:active {
                    color: rgb(130, 130, 130);
                }
            }

            iconify-icon {
                margin-right: 5px;
                font-size: 15px;
                text-align: center;
                vertical-align: middle;
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
        display: flex;
        flex-direction: column;
        background-color: $sidebar_secondary_color;
        top: 0;
        z-index: 3;
        transition: height 1s ease-in-out;

        .top-header {
            /* height: 80px; */
            position: sticky;
            top: 0;
        }

        .app-title {
            font-family: "2Peas";
            width: fit-content;
            font-size: 2em;
            opacity: 0.2;
            user-select: none;
            margin: 1em auto 0;
            transition: height 1s ease-in-out;
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
            color: rgb(197, 193, 193);
            backdrop-filter: blur(8px);
            z-index: 10;
            &::placeholder {
                color: rgb(103, 100, 100);
            }
            &:focus {
                /* outline: 1px solid #5123dd; */
                background-color: #504c4c;
                &::placeholder {
                    color: rgb(151, 147, 147);
                }
            }
            background-color: transparent;

            border: 1px solid rgb(63, 63, 63);
        }
        .search-icon {
            position: absolute;
            right: 15px;
            top: 0px;
            bottom: 0;
            height: fit-content;
            padding: 5px;
            margin: auto 0;
            > iconify-icon {
                font-size: 17px;
                color: #737373;
                pointer-events: none;
            }
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
        border-top: 0.7px solid #ffffff23;
        z-index: 2;
        overflow: hidden;
    }

    .track-info-content {
        width: 100%;
        height: fit-content;
        position: sticky;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
    }

    .mini-toggle {
        pointer-events: all;
        font-size: 20px;

        position: absolute;
        top: 15px;
        right: 8px;
        color: rgb(115, 115, 115);
        padding: 3px;

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }

        &.hovered {
            background-color: rgba(0, 0, 0, 0.457);
            border-radius: 4px;
        }
    }
    .info {
        background-color: $sidebar_primary_color;
        color: white;
        width: 100%;
        padding: 0.8em 0.3em;

        .marquee-container {
            height: 20px;
            width: 100%;
            mask-image: linear-gradient(
                to right,
                transparent 0%,
                $sidebar_secondary_color 15%,
                $sidebar_secondary_color 85%,
                transparent 100%
            );

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
            z-index: 1;
            text-overflow: ellipsis;
            overflow: hidden;
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
        color: rgb(125, 125, 125);

        p {
            background-color: rgba(85, 85, 85, 0.162);
            padding: 0em 0.6em;
            margin: 0;
            border-radius: 2px;
            font-size: 0.67em;
            line-height: 1.5em;
            font-weight: 600;
            border-top: 1px solid rgb(49, 49, 49);
            /* border-bottom: 1px dashed rgb(49, 49, 49); */
            font-family: monospace;
            text-transform: uppercase;
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
        border-top: 0.7px solid #ffffff23;
        border-bottom: 0.7px solid #ffffff23;
        z-index: 1;

        .artwork-frame {
            width: 100%;
            height: 100%;
            box-sizing: border-box;
            /* border-radius: 3px; */
            /* border: 1px solid rgb(94, 94, 94); */
            display: flex;
            align-items: center;
            justify-content: center;
            > img {
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
                iconify-icon {
                    /* margin-top: 0.7em; */
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
        background-color: #242026bc;
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
    }
    transport {
        /* background-color: rgb(255, 255, 255); */
        padding: 0em 1em 1em 1em;
        width: 100%;
        color: white;
        z-index: 2;
        display: flex;
        justify-content: space-between;

        .transport-middle {
            align-self: center;
            font-size: 42px;
        }

        :not(.off)[icon="ph:shuffle-bold"] {
            color: #e1ff00;
        }

        .transport-side {
            align-self: center;
            font-size: 20px;
            &.favourite.active {
                color: #59cd7a;
            }
            &.favourite.inactive {
                color: grey;
            }
        }
    }

    .other-controls {
        padding: 0 1em 1em;
        width: 100%;
        display: flex;
        align-items: center;
        gap: 10px;

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

    .volume {
        width: 100%;
        display: flex;
        align-items: center;

        input {
            -webkit-appearance: none;
            width: 100%;
            height: 5px;
            background: #474747d4;
            outline: none;
            opacity: 1;
            border-radius: 3px;
            -webkit-transition: 0.2s;
            transition: opacity 0.2s;

            &::-webkit-slider-thumb {
                -webkit-appearance: none;
                appearance: none;
                width: $thumb_size;
                height: $thumb_size;
                background: url("/images/volume-up.svg");
            }

            &::-moz-range-thumb {
                width: $thumb_size;
                height: $thumb_size;
                background: #04aa6d;
            }
        }
    }

    iconify-icon {
        font-size: 40px;
        &.disabled {
            pointer-events: none;
            color: #474747;
        }

        // Like disabled but clickable
        &.off {
            color: #474747;
        }

        &:hover {
            opacity: 0.5;
        }

        &:active {
            color: rgb(141, 47, 47);
            opacity: 1;
        }
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
                background-color: #5123dd;
                border-radius: 5px;
            }

            &.hover {
                border-radius: 5px;
                background-color: #392f5d3b;
                .playlist-menu {
                    display: flex;
                }
            }

            &.selected {
                p {
                    color: white;
                    font-weight: bold;
                }

                &::before {
                    content: "";
                    width: 2px;
                    left: -4px;
                    position: absolute;
                    height: 70%;
                    background-color: #5123dd;
                    border-radius: 4px;
                }
            }
        }

        .playlist-menu {
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
            color: rgb(159, 160, 165);
            margin: 0;
            cursor: default;
            pointer-events: none;
        }

        .new-playlist {
            display: flex;
            margin: 0 0 0 1.6em;
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
            .track-info-content {
                margin-top: 1.5em;
            }

            .other-controls {
                padding: 0.5em 2em 1em;
            }

            .seekbar {
                padding: 0.5em 0;
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
