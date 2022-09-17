<script lang="ts">
    import hotkeys from "hotkeys-js";
    import { cloneDeep, debounce, isEqual } from "lodash-es";
    import type {
        ArtistContentItem,
        ArtistFileItem,
        ArtistLinkItem,
        ContentFileType,
        ContentItem,
        Song,
        SongProject
    } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        draggedScrapbookItems,
        emptyDropEvent,
        os,
        songDetailsUpdater
    } from "../../data/store";
    import { autoWidth } from "../../utils/AutoWidth";
    import LyricsTab from "./LyricsTab.svelte";
    import MusicTab from "./MusicTab.svelte";
    import OtherTab from "./OtherTab.svelte";

    import { getSongFromFile } from "../../data/LibraryImporter";
    import {
        droppedFiles,
        fileDropHandler,
        hoveredFiles
    } from "../../data/store";
    import { getContentFileType } from "../../utils/FileUtils";
    import { readTextFile } from "@tauri-apps/api/fs";
    import toast from "svelte-french-toast";

    export let songProject: SongProject;
    export let song: Song; // We might need to create a project based on this song
    export let onSelectSong;

    let songProjectClone: SongProject = cloneDeep(songProject);
    $: isProject = songProject?.id;

    // TODO move the music stuff into MusicTab?
    let bpmTicker = 1;
    let bpmInterval;

    // Eg. 127 beats per minute = 1 beat every 0.4724409449 seconds
    function startBPMTicker(bpm: number) {
        if (bpmTicker === 1 && !bpmInterval) {
            bpmInterval = setInterval(() => {
                if (songProjectClone?.bpm !== undefined) {
                    bpmTicker++;
                } else {
                    stopBpmTicker();
                }
            }, 60000 / bpm / 2);
        }
    }

    function stopBpmTicker() {
        clearInterval(bpmInterval);
        bpmInterval = null;
        bpmTicker = 1;
    }

    function onBpmFocus() {
        if (songProjectClone?.bpm < 400) {
            startBPMTicker(songProjectClone?.bpm);
        }
    }

    let hasChanged = false;

    /**
     * Reset this component to match new song selection.
     */
    $: songProjectClone = songProject;

    // Automatically save to database on any changes

    // $: {
    //     // Will only have ID once it's a song project
    //     // So normal Songs won't be saved automatically,
    //     // needs to be converted into a project first

    //     if (
    //         songProject?.id !== undefined &&
    //         songProject.id !== songProjectClone?.id
    //     ) {
    //         reset();
    //     } else if (
    //         (songProject?.id !== undefined &&
    //             songProject.id === songProjectClone?.id &&
    //             !isEqual(songProject, songProjectClone)) ||
    //         $songDetailsUpdater
    //     ) {
    //         console.log("putting clone: ;", songProjectClone);
    //         if (songProjectClone) {
    //             db.songProjects.put(songProjectClone);
    //         }
    //     } else {
    //         reset();
    //     }
    // }

    // Instead of the above just have listeners for things that have changed
    // and then we don't need to worry about which direction the changes are in.
    // i.e we know a user has initiated the action

    function saveSongProject() {
        console.log("saving clone to db:", songProjectClone);
        if (songProjectClone) {
            db.songProjects.put(songProjectClone);
        }
    }

    function onTitleUpdated(evt) {
        songProjectClone.title = evt.target.value;
        saveSongProject();
    }

    function onAlbumUpdated(evt) {
        songProjectClone.album = evt.target.value;
        saveSongProject();
    }

    function onComposerUpdated(evt) {
        songProjectClone.musicComposedBy = [evt.target.value];
        saveSongProject();
    }

    function onLyricistUpdated(evt) {
        songProjectClone.lyricsWrittenBy = [evt.target.value];
        saveSongProject();
    }

    function onBpmUpdated(e) {
        if (e.target.value) {
            songProjectClone.bpm = e.target.value;
            if (e.target.value > 0 && e.target.value < 400) {
                stopBpmTicker();
                startBPMTicker(e.target.value);
            } else {
                stopBpmTicker();
            }
        } else {
            songProjectClone.bpm = null;
            stopBpmTicker();
        }
        saveSongProject();
    }

    function onKeyUpdated(evt) {
        songProjectClone.key = evt.target.value;
        saveSongProject();
    }

    /**
     * Add a recording from a file path
     * Note: the path is treated as a key, no duplicates allowed
     * @param filePath
     */
    async function addRecording(filePath: string) {
        console.log("adding recording", filePath);
        const filename = filePath.split("/").pop();

        if (
            songProjectClone?.recordings
                .map((r) => r.song.path)
                .includes(filePath)
        ) {
            toast.error(
                `${filename} already exists in ${songProjectClone.title}'s recordings`
            );
            return;
        }
        console.log("filename", filename);
        const song = await getSongFromFile(filePath, filename);
        console.log("song", song);

        if (!songProjectClone?.recordings) songProjectClone.recordings = [];
        songProjectClone.recordings.push({
            recordingType: "master",
            song
        });
        songProjectClone.recordings = songProjectClone.recordings;
        saveSongProject();
    }

    function deleteRecording(recordingIdx) {
        songProjectClone.recordings?.splice(recordingIdx, 1);
        songProjectClone = songProjectClone;
        saveSongProject();
    }

    function onLyricsUpdated(lyrics: string) {
        songProjectClone.lyrics = lyrics;
        saveSongProject();
    }

    function addContentItem(item: ArtistContentItem) {
        const files = songProjectClone?.otherContentItems.filter(
            (i) => i.type === "file"
        ) as ArtistFileItem[];
        const links = songProjectClone?.otherContentItems.filter(
            (i) => i.type === "link"
        ) as ArtistLinkItem[];

        if (
            (item.type == "file" &&
                files.map((r) => r.path).includes(item.path)) ||
            (item.type == "link" && links.map((r) => r.url).includes(item.url))
        ) {
            toast.error(
                `${item.name} already exists in ${songProjectClone.title}`
            );
            return;
        }
        songProjectClone.otherContentItems?.push(item);
        songProjectClone.otherContentItems = songProjectClone.otherContentItems;
        saveSongProject();
    }

    function deleteContentItem(rightClickedItemIdx) {
        songProjectClone.otherContentItems?.splice(rightClickedItemIdx, 1);
        songProjectClone.otherContentItems = songProjectClone.otherContentItems;
        saveSongProject();
    }

    const tabs: Array<"music" | "lyrics" | "other"> = [
        "music",
        "lyrics",
        "other"
    ];
    let selectedTab: "music" | "lyrics" | "other" = tabs[0];

    export let onDeleteSongProject;

    const fontSizes = [
        "font-12",
        "font-14",
        "font-20",
        "font-24",
        "font-32",
        "font-48"
    ];

    let currentFontSizeIdx = 1;
    $: currentFontSize = fontSizes[currentFontSizeIdx];

    function increaseFontSize() {
        currentFontSizeIdx =
            currentFontSizeIdx <= fontSizes.length - 2
                ? currentFontSizeIdx + 1
                : fontSizes.length - 1;
    }
    function decreaseFontSize() {
        currentFontSizeIdx =
            currentFontSizeIdx > 0 ? currentFontSizeIdx - 1 : 0;
    }
    let fullScreenLyrics = false;
    function toggleFullScreenLyrics() {
        fullScreenLyrics = !fullScreenLyrics;
    }

    // Shortcuts

    let modifier = $os === "Darwin" ? "cmd" : "ctrl";
    hotkeys(`${modifier}+=`, function (event, handler) {
        increaseFontSize();
    });
    hotkeys(`${modifier}+-`, function (event, handler) {
        decreaseFontSize();
    });

    // Lifecycle

    onMount(() => {
        containerRect = container.getBoundingClientRect();
    });

    onDestroy(() => {
        hotkeys.unbind(`${modifier}+=`);
        hotkeys.unbind(`${modifier}+-`);
    });

    async function addOther(filePath: string, type: ContentFileType) {
        // Check if already exists, if so show a message
        const filename = filePath.split("/").pop();

        if (!songProjectClone?.otherContentItems)
            songProjectClone.otherContentItems = [];
        const toAdd: ArtistFileItem = {
            name: filename,
            tags: [],
            type: "file",
            fileType: type,
            path: filePath
        };
        addContentItem(toAdd);
    }

    /**
     * Adding an existing link item that was already in the Scrapbook.
     * It also already likely has the attached metadata (eg Youtube video title, img)
     * so no need to fetch it again.
     * @param item
     */
    async function addLinkItemFromScrapbook(item: ArtistLinkItem) {
        if (!songProjectClone?.otherContentItems)
            songProjectClone.otherContentItems = [];
        addContentItem(item);
    }

    function onMouseEnter() {
        console.log("Entered song-details: hovered files: ", $hoveredFiles);
        if ($hoveredFiles.length > 0) {
            $fileDropHandler = "song-details";
        }
    }

    function onMouseLeave() {}

    /**
     * Handle file drop (applies to both scrapbook and file drop from system)
     * @param files the file paths to add
     */
    async function handleFileDrop(files: string[]) {
        console.log("drop:", files);
        for (const droppedFile of files) {
            const fileType = getContentFileType(droppedFile);
            switch (fileType.type) {
                case "audio":
                    if (selectedTab === "music") {
                        addRecording(droppedFile);
                    } else if (selectedTab === "other") {
                        addOther(droppedFile, fileType);
                    }
                    break;
                case "txt":
                    if (selectedTab === "lyrics") {
                        const readText = await readTextFile(droppedFile);
                        if (readText) {
                            // TODO confirm if want to append or replace?
                            songProjectClone.lyrics = readText;
                        } else {
                            console.error("Empty/invalid lyrics file?");
                        }
                    } else {
                        addOther(droppedFile, fileType);
                    }
                    break;
                case "image":
                case "video":
                    addOther(droppedFile, fileType);
                    break;
            }
        }
        $droppedFiles = [];
        $hoveredFiles = [];
    }

    async function handleLinkDrop(links: ArtistLinkItem[]) {
        for (const droppedLink of links) {
            await addLinkItemFromScrapbook(droppedLink);
        }
        $droppedFiles = [];
        $hoveredFiles = [];
    }

    $: {
        if (
            $droppedFiles &&
            $droppedFiles.length > 0 &&
            $fileDropHandler === "song-details"
        ) {
            handleFileDrop($droppedFiles);
            isDragging = false;
        }
    }

    let isDragging = false;
    let container: HTMLElement;
    let containerRect;

    async function onDragEnter(evt: DragEvent) {
        isDragging = true;
        evt.dataTransfer.dropEffect = "copy";
        $fileDropHandler = "song-details";
    }

    function onDragLeave(evt) {
        if (
            evt.clientY < containerRect.top ||
            evt.clientY >= container.clientHeight + containerRect.top ||
            evt.clientX < containerRect.left ||
            evt.clientX >= container.clientWidth + containerRect.left
        ) {
            console.log("dragleave", evt.dataTransfer.items);
            isDragging = false;
        }
    }

    async function getDropDataAsString(
        item: DataTransferItem
    ): Promise<string> {
        return new Promise((resolve, reject) => {
            try {
                item.getAsString((str) => {
                    resolve(str);
                });
            } catch (err) {
                reject(err);
            }
        });
    }

    async function onDrop(evt: DragEvent) {
        /*
        Unfortunately this doesn't trigger when using system file drop in Tauri (fileDropEnabled). 
        At the moment there is no support for both drag&drop and HTML drag&drop, we need to choose. 
        If we disable the system event, then we won't get access to the full path 
        because browsers don't expose this due to security reasons

        So we use a hack (below this function) - we use the empty system event that's triggered to 
        handle a drop, grabbing the dragged item from the store. 
        */
        console.log("drop", evt);

        // If we have support for
        // const items = evt.dataTransfer.items;
        // for (let i = 0; i < items.length; i++) {
        //     try {
        //         const content = await getDropDataAsString(items[i]);
        //         const parsed = JSON.parse(content);
        //         if (parsed && parsed.type) {
        //             const item = parsed as ArtistContentItem;

        //             switch (item.type) {
        //                 case "file":
        //                     if (item.fileType.type === "audio") {
        //                         // Add as recording
        //                         await addRecording(item.path);
        //                     }
        //             }
        //         }
        //     } catch (err) {
        //         console.error("error doing drop: ", err);
        //     }
        // }
    }

    $: {
        if (
            $droppedFiles &&
            $droppedFiles.length > 0 &&
            $fileDropHandler === "song-details"
        ) {
            handleFileDrop($droppedFiles);
        }

        if (
            $emptyDropEvent &&
            $draggedScrapbookItems &&
            $draggedScrapbookItems.length > 0
        ) {
            const scrapbookItems = $draggedScrapbookItems.filter(
                (i) => i.type === "file" || i.type === "link"
            ) as ArtistContentItem[];
            isDragging = false;
            // Only handle if global cursor position is in this component
            if (
                containerRect &&
                songProject?.id &&
                $emptyDropEvent.x >= containerRect.left &&
                $emptyDropEvent.x <=
                    containerRect.left + container.clientWidth &&
                $emptyDropEvent.y >= containerRect.top &&
                $emptyDropEvent.y <= containerRect.top + container.clientHeight
            ) {
                const files = scrapbookItems.filter(
                    (i) => i.type === "file"
                ) as ArtistFileItem[];
                const links = scrapbookItems.filter(
                    (i) => i.type === "link"
                ) as ArtistLinkItem[];

                files.length && handleFileDrop(files.map((f) => f.path));
                links.length && handleLinkDrop(links);

                $emptyDropEvent = null;
                $draggedScrapbookItems = [];
            }
        }
    }
</script>

<container
    bind:this={container}
    class:full-screen={fullScreenLyrics}
    class:dragging={isDragging}
    on:mouseenter={onMouseEnter}
    on:mouseleave={onMouseLeave}
    on:dragenter={onDragEnter}
    on:dragleave={onDragLeave}
    dropzone="copy"
>
    <header>
        {#if songProjectClone}
            <div class="info">
                <span>
                    <p>"</p>
                    <input
                        use:autoWidth
                        value={songProjectClone.title}
                        on:input={onTitleUpdated}
                        placeholder="?"
                    />
                    <p>"</p></span
                >
            </div>
            {#if isProject}
                <iconify-icon
                    class="delete-icon"
                    icon="ant-design:delete-outlined"
                    on:click={() => {
                        onDeleteSongProject(songProject);
                    }}
                />
            {/if}
        {:else}
            <p>Select a song on the left</p>
        {/if}
    </header>

    {#if songProjectClone}
        <div class="details">
            <div>
                <p>in album:</p>
                <input
                    value={songProjectClone.album}
                    on:input={onAlbumUpdated}
                    placeholder="add an album"
                />
            </div>
            <div>
                <p>music written by:</p>
                <input
                    value={songProjectClone.musicComposedBy}
                    on:input={onComposerUpdated}
                    placeholder="add a composer"
                />
            </div>
            <div>
                <p>lyrics written by:</p>
                <input
                    value={songProjectClone.lyricsWrittenBy}
                    on:input={onLyricistUpdated}
                    placeholder="add a lyricist"
                />
            </div>
        </div>

        <div class="music-info">
            <div>
                <p class:bpm={bpmTicker % 2 === 0}>BPM:</p>
                <input
                    value={songProjectClone.bpm ?? null}
                    on:input={debounce(onBpmUpdated, 300)}
                    on:focus={onBpmFocus}
                    on:blur={stopBpmTicker}
                    placeholder="bpm"
                    max="300"
                    min="20"
                    autocomplete="off"
                    spellcheck="false"
                />
            </div>
            <div>
                <p>key:</p>
                <input
                    value={songProjectClone.key ?? null}
                    on:input={onKeyUpdated}
                    placeholder="key"
                    autocomplete="off"
                    spellcheck="false"
                />
            </div>
        </div>
        <div class="content-container">
            <div class="content-header">
                <div class="content-tabs">
                    {#each tabs as tab}
                        <div
                            class="tab"
                            class:selected={selectedTab === tab}
                            on:click={() => {
                                selectedTab = tab;
                            }}
                        >
                            <p>{tab}</p>
                        </div>
                    {/each}
                </div>
                <div class="lyrics-options">
                    <iconify-icon
                        icon="mdi:format-font-size-decrease"
                        on:click={decreaseFontSize}
                    />
                    <iconify-icon
                        icon="mdi:format-font-size-increase"
                        on:click={increaseFontSize}
                    />
                    <iconify-icon
                        icon="icon-park-outline:full-screen-one"
                        on:click={toggleFullScreenLyrics}
                    />
                </div>

                {#if selectedTab !== "other"}
                    <div class="bg-gradient" />
                {/if}
            </div>
            <content class={selectedTab}>
                {#if selectedTab === "music"}
                    <MusicTab
                        recordings={songProjectClone?.recordings}
                        {addRecording}
                        {deleteRecording}
                        {songProject}
                        {song}
                        {onSelectSong}
                        showDropPlaceholder={isDragging}
                    />
                {:else if selectedTab === "lyrics"}
                    <LyricsTab
                        fontSize={currentFontSize}
                        isFullScreen={fullScreenLyrics}
                        lyrics={songProjectClone?.lyrics}
                        {onLyricsUpdated}
                        enabled={songProjectClone?.id !== undefined}
                    />
                {:else if selectedTab === "other"}
                    <OtherTab
                        items={songProjectClone.otherContentItems}
                        enabled={songProjectClone?.id !== undefined}
                        showDropPlaceholder={isDragging}
                        addContentItem={handleFileDrop}
                        {deleteContentItem}
                    />
                {/if}
            </content>
        </div>
    {/if}
</container>

<style lang="scss">
    container {
        display: grid;
        grid-template-rows: auto auto auto 1fr;
        height: 100%;
        &.dragging {
            * {
                pointer-events: none !important;
            }
        }

        &.full-screen {
            position: fixed;
            top: 0;
            bottom: 0;
            right: 0;
            left: 0;
            width: 100%;
            height: 100%;
            z-index: 20;
            background-color: #252126;

            grid-template-rows: 1fr;
            .details,
            .music-info {
                display: none;
            }
            header {
                padding: 2em;
                position: absolute;
            }

            .lyrics-options {
                position: absolute;
                top: 5px;
                right: 0;
            }
            .delete-icon {
                display: none;
            }
            .content-tabs {
                margin-top: 6em;
            }
            content {
                top: 7em;
                &.lyrics {
                    top: 0;
                }
            }
            .bg-gradient {
                display: none;
            }
        }

        header {
            display: grid;
            grid-template-columns: 1fr auto;
            align-items: center;
            padding: 2em;
            gap: 1em;

            h2 {
                font-family: "Snake";
                font-size: 3em;
                margin: 0;
            }

            .info {
                display: flex;
                flex-direction: column;
                span {
                    font-family: "Snake";
                    display: inline-flex;
                    p {
                        font-size: 3em;
                        margin: 0;
                    }
                    input {
                        font-family: "Snake";
                        font-size: 3em;
                        padding: 0;
                        min-width: 20px;
                    }
                }
            }
        }

        .details {
            padding: 0 2em 1em 2em;
            > div {
                display: flex;
                flex-direction: row;
                align-items: center;
                width: 100%;
                p {
                    opacity: 0.7;
                    margin: 0;
                    white-space: nowrap;
                }
            }
        }

        .music-info {
            padding: 0 2em 0 2em;
            display: flex;
            flex-direction: row;
            justify-content: flex-start;
            border-top: 1px solid rgba(255, 255, 255, 0.093);
            border-bottom: 1px solid rgba(255, 255, 255, 0.093);

            > div {
                padding: 0.3em 0;
                display: inline-flex;
                align-items: center;
                p {
                    opacity: 0.7;
                    margin: 0;
                    position: relative;

                    &.bpm::before {
                        content: "•";
                        position: absolute;
                        color: cyan;
                        left: -20px;
                        top: -2px;
                        font-size: 30px;
                    }
                }
                input {
                    max-width: 60px;
                }
            }
        }

        .content-container {
            position: relative;
            height: 100%;
            overflow: hidden;

            .content-header {
                position: sticky;
                top: 0;
                width: 100%;
                display: flex;
                align-items: center;
                justify-content: space-between;
                z-index: 3;
            }

            .bg-gradient {
                height: 80px;
                width: 100%;
                top: 0;
                position: absolute;
                z-index: -1;
                background: linear-gradient(
                    to bottom,
                    #252126 0%,
                    #242126c4 60%,
                    transparent 100%
                );
            }
        }

        content {
            position: absolute;
            top: 2em;
            left: 0;
            right: 0;
            bottom: 0;
            z-index: 0;
        }

        .content-tabs {
            display: flex;
            justify-content: center;
            gap: 1em;
            padding: 0.3em 2em;
            > .tab {
                padding: 0.5em 0;
                cursor: default;
                opacity: 0.6;
                p {
                    margin: 0;
                    text-transform: capitalize;
                }
                &:hover {
                    opacity: 0.5;
                    /* border-bottom: 2px solid rgba(255, 255, 255, 0.487); */
                }
                &:active {
                    opacity: 0.4;
                }

                &.selected {
                    opacity: 1;
                    border-bottom: 2px solid white;
                }
            }
        }

        .lyrics-options {
            display: flex;
            flex-direction: row;
            gap: 3px;
            align-items: center;
            margin-right: 20px;

            iconify-icon {
                padding: 6px;
                font-size: 20px;
                border-radius: 4px;
                &:hover {
                    background-color: rgba(0, 0, 0, 0.457);
                }
            }
        }
    }
    p {
        cursor: default;
    }

    input {
        width: 100%;
        align-items: center;
        padding: 0.2em 0.5em;
        font-size: 14px;
        outline: none;
        background: none;
        border: none;

        &::placeholder {
            color: rgb(105, 105, 105);
        }
    }
</style>
