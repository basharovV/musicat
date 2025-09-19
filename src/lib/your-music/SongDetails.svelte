<script lang="ts">
    import hotkeys from "hotkeys-js";
    import { cloneDeep, debounce } from "lodash-es";
    import type {
        ArtistContentItem,
        ArtistFileItem,
        ArtistLinkItem,
        ArtistProject,
        ContentFileType,
        Song,
        SongProject,
    } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        draggedScrapbookItems,
        emptyDropEvent,
        os,
        songbookFileSavedTime,
    } from "../../data/store";
    import { autoWidth } from "../../utils/AutoWidth";
    import LyricsTab from "./LyricsTab.svelte";
    import MusicTab from "./MusicTab.svelte";
    import OtherTab from "./OtherTab.svelte";

    import { invoke } from "@tauri-apps/api/core";
    import { readTextFile } from "@tauri-apps/plugin-fs";
    import toast from "svelte-french-toast";
    import {
        droppedFiles,
        fileDropHandler,
        hoveredFiles,
    } from "../../data/store";
    import { getContentFileType } from "../../utils/FileUtils";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import Icon from "../ui/Icon.svelte";
    import Input from "../ui/Input.svelte";
    import KeySelector from "../ui/KeySelector.svelte";
    import LL from "../../i18n/i18n-svelte";
    import {
        renameSongProject,
        saveFrontmatterToSongProject,
        writeChordMarkToSongProject,
    } from "../../data/ArtistsToolkitData";

    export let songProject: SongProject;
    export let onSelectSong;

    let songProjectClone: SongProject = cloneDeep(songProject);
    $: isProject = songProject?.songFilepath;

    // TODO move the music stuff into MusicTab?
    let bpmTicker = 1;
    let bpmInterval;

    // Eg. 127 beats per minute = 1 beat every 0.4724409449 seconds
    function startBPMTicker(bpm: number) {
        if (bpmTicker === 1 && !bpmInterval) {
            bpmInterval = setInterval(
                () => {
                    if (songProjectClone?.bpm !== undefined) {
                        bpmTicker++;
                    } else {
                        stopBpmTicker();
                    }
                },
                60000 / bpm / 2,
            );
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

    // Instead of the above just have listeners for things that have changed
    // and then we don't need to worry about which direction the changes are in.
    // i.e we know a user has initiated the action

    function saveSongProject() {
        console.log("saving clone to db:", songProjectClone);
        if (songProjectClone) {
            db.songProjects.put(songProjectClone);
        }
    }

    let isConfirmingSongDelete = true;
    let menuPos = { x: 0, y: 0 };
    let showMenu = false;

    function onDeleteSongConfirm(e: MouseEvent) {
        showMenu = !showMenu;
        menuPos = { x: e.clientX, y: e.clientY };
    }

    function deleteSong() {
        const title = songProject.title;
        onDeleteSongProject(songProject);
        showMenu = false;

        toast.success(`Deleted ${title}`);
    }

    async function onTitleUpdated(evt) {
        // Update folder name
        await renameSongProject(
            songProject.artist,
            songProjectClone.title,
            evt.target.value,
        );
        songProjectClone.title = evt.target.value;
    }

    async function onAlbumUpdated(album) {
        songProjectClone.album = album;
        await saveFrontmatterToSongProject(songProjectClone);
    }

    let composerInput;
    let lyricistInput;
    let composerAutocomplete;
    let lyricistAutocomplete;

    export let artist: ArtistProject;

    function onAddComposer() {
        if (composerAutocomplete?.length) {
            songProjectClone.musicComposedBy.push(composerAutocomplete);
        } else {
            songProjectClone.musicComposedBy.push(composerInput);
        }
        songProjectClone.musicComposedBy = songProjectClone.musicComposedBy;

        saveFrontmatterToSongProject(songProjectClone);
        composerInput = "";
        composerAutocomplete = "";
    }

    function onAddLyricist() {
        if (lyricistAutocomplete?.length) {
            songProjectClone.lyricsWrittenBy.push(lyricistAutocomplete);
        } else {
            songProjectClone.lyricsWrittenBy.push(lyricistInput);
        }
        songProjectClone.lyricsWrittenBy = songProjectClone.lyricsWrittenBy;

        saveFrontmatterToSongProject(songProjectClone);
        lyricistInput = "";
        lyricistAutocomplete = "";
    }

    function onComposerUpdated(evt) {
        composerInput = evt.target.value;
        composerAutocomplete = composerInput.length
            ? artist.members.find((a) => a.startsWith(composerInput))
            : "";
        console.log("match", composerAutocomplete);
    }

    function onLyricistUpdated(evt) {
        lyricistInput = evt.target.value;
        lyricistAutocomplete = lyricistInput.length
            ? artist.members.find((a) => a.startsWith(lyricistInput))
            : "";
        console.log("match", lyricistAutocomplete);
    }

    function removeLastComposer() {
        if (songProjectClone.musicComposedBy.length) {
            songProjectClone.musicComposedBy.splice(
                songProjectClone.musicComposedBy.length - 1,
                1,
            );
            songProjectClone.musicComposedBy = songProjectClone.musicComposedBy;

            saveFrontmatterToSongProject(songProjectClone);
        }
    }
    function removeLastLyricist() {
        if (songProjectClone.lyricsWrittenBy.length) {
            songProjectClone.lyricsWrittenBy.splice(
                songProjectClone.lyricsWrittenBy.length - 1,
                1,
            );
            songProjectClone.lyricsWrittenBy = songProjectClone.lyricsWrittenBy;

            saveFrontmatterToSongProject(songProjectClone);
        }
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

        saveFrontmatterToSongProject(songProjectClone);
    }

    function onKeyUpdated(key) {
        console.log("key updated", key);

        songProjectClone.key = key;

        saveFrontmatterToSongProject(songProjectClone);
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
                `${filename} already exists in ${songProjectClone.title}'s recordings`,
            );
            return;
        }
        console.log("filename", filename);
        const song = await invoke<Song>("get_song_metadata", {
            event: {
                path: filePath,
                isImport: false,
                includeFolderArtwork: false,
                includeRawTags: false,
            },
        });
        if (!song) return;
        console.log("song", song);

        if (!songProjectClone?.recordings) songProjectClone.recordings = [];
        songProjectClone.recordings.push({
            recordingType: "master",
            song,
        });
        songProjectClone.recordings = songProjectClone.recordings;
        saveSongProject();
    }

    function deleteRecording(recordingIdx) {
        songProjectClone.recordings?.splice(recordingIdx, 1);
        songProjectClone = songProjectClone;
        saveSongProject();
    }

    async function onLyricsUpdated(lyrics: string) {
        songProjectClone.lyrics = lyrics;
        // Write to file
        try {
            await writeChordMarkToSongProject(songProjectClone, lyrics);
            $songbookFileSavedTime = Date.now();
        } catch (e) {
            console.error(e);
        }
        // saveSongProject();
    }

    function addContentItem(item: ArtistContentItem) {
        const files = songProjectClone?.otherContentItems.filter(
            (i) => i.type === "file",
        ) as ArtistFileItem[];
        const links = songProjectClone?.otherContentItems.filter(
            (i) => i.type === "link",
        ) as ArtistLinkItem[];

        if (
            (item.type == "file" &&
                files.map((r) => r.path).includes(item.path)) ||
            (item.type == "link" && links.map((r) => r.url).includes(item.url))
        ) {
            toast.error(
                `${item.name} already exists in ${songProjectClone.title}`,
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

    interface Tab {
        name: string;
        value: string;
    }

    const tabs: Array<Tab> = [
        { name: $LL.artistsToolkit.songDetails.tabs.lyrics(), value: "lyrics" },
        { name: $LL.artistsToolkit.songDetails.tabs.files(), value: "files" },
        { name: $LL.artistsToolkit.songDetails.tabs.other(), value: "other" },
    ];

    let selectedTab: Tab = tabs[0];

    export let onDeleteSongProject;

    // Lifecycle

    onMount(() => {
        containerRect = container.getBoundingClientRect();
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
            path: filePath,
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
        isDragging = false;
        for (const droppedFile of files) {
            const fileType = getContentFileType(droppedFile);
            switch (fileType.type) {
                case "audio":
                    if (selectedTab.value === "files") {
                        addRecording(droppedFile);
                    } else if (selectedTab.value === "other") {
                        addOther(droppedFile, fileType);
                    }
                    break;
                case "txt":
                    if (selectedTab.value === "lyrics") {
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
        item: DataTransferItem,
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
                (i) => i.type === "file" || i.type === "link",
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
                    (i) => i.type === "file",
                ) as ArtistFileItem[];
                const links = scrapbookItems.filter(
                    (i) => i.type === "link",
                ) as ArtistLinkItem[];

                files.length && handleFileDrop(files.map((f) => f.path));
                links.length && handleLinkDrop(links);

                $emptyDropEvent = null;
                $draggedScrapbookItems = [];
            }
        }
    }

    let isAddingLyricist = false;

    $: if ($songbookFileSavedTime !== null) {
        // Set to null after 2 seconds
        // setTimeout(() => ($songbookFileSavedTime = null), 2000);
    }

    // hh:mm from unix timestamp
    $: lastSavedTime = $songbookFileSavedTime
        ? `${new Date($songbookFileSavedTime).toLocaleTimeString()}`
        : null;
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<container
    bind:this={container}
    class:dragging={isDragging}
    on:mouseenter={onMouseEnter}
    on:mouseleave={onMouseLeave}
    on:dragenter={onDragEnter}
    on:dragleave={onDragLeave}
    dropzone="copy"
>
    {#if songProjectClone}
        <header>
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
                <Icon
                    icon="ant-design:delete-outlined"
                    onClick={(e) => {
                        onDeleteSongConfirm(e);
                    }}
                />
            {/if}
        </header>
    {:else}
        <div class="no-song-selected">
            <h1>Welcome to your Songbook!</h1>
            <h2>- organise your music and lyrics ideas</h2>
            <h2>- track progress of songs</h2>
            <h2>- for multiple artists and projects</h2>
            <!-- svelte-ignore a11y-missing-attribute -->
            <img src="/icon.png" />
        </div>
    {/if}
    {#if showMenu}
        <Menu
            x={menuPos.x}
            y={menuPos.y}
            onClickOutside={() => {
                showMenu = false;
            }}
            position="manual"
        >
            <MenuOption
                text="Delete artist"
                confirmText="Are you sure?"
                isDestructive
                isConfirming={isConfirmingSongDelete}
                onClick={deleteSong}
            />
        </Menu>
    {/if}
    {#if songProjectClone}
        <div class="credits-details">
            <div class="credits">
                <div>
                    <p class="label">music by:</p>
                    <div class="members">
                        {#each songProjectClone.musicComposedBy as composer}
                            <div class="member">
                                <p>{composer}</p>
                            </div>
                            ,
                        {/each}
                    </div>
                    <Input
                        value={composerInput}
                        onChange={onComposerUpdated}
                        autoCompleteValue={composerAutocomplete}
                        onEnterPressed={onAddComposer}
                        onBackspacePressed={removeLastComposer}
                        minimal
                        tabBehavesAsEnter
                        placeholder="add a composer"
                    />
                </div>
                <div>
                    <p class="label">lyrics by:</p>
                    <div class="members">
                        {#each songProjectClone.lyricsWrittenBy as lyricist}
                            <div class="member">
                                <p>{lyricist}</p>
                            </div>
                            ,
                        {/each}
                    </div>
                    {#if songProjectClone.lyricsWrittenBy.length === 0 || isAddingLyricist}
                        <Input
                            value={lyricistInput}
                            onChange={onLyricistUpdated}
                            autoCompleteValue={lyricistAutocomplete}
                            onEnterPressed={onAddLyricist}
                            onBackspacePressed={removeLastLyricist}
                            minimal
                            tabBehavesAsEnter
                            placeholder="add a lyricist"
                        />
                    {:else if songProjectClone.lyricsWrittenBy.length > 0}
                        <div
                            class="add-tag"
                            on:click={(e) => (isAddingLyricist = true)}
                        >
                            <Icon icon="ic:baseline-plus" size={20} />
                        </div>
                    {/if}
                </div>
            </div>
            <div class="details">
                <div>
                    <p class="label">in album:</p>
                    <Input
                        value={songProjectClone.album}
                        onChange={onAlbumUpdated}
                        placeholder="add an album"
                        minimal
                    />
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
                        <KeySelector
                            value={songProjectClone.key ?? null}
                            {onKeyUpdated}
                        />
                    </div>
                </div>
            </div>
        </div>
        <div class="content-tabs">
            {#each tabs as tab}
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <div
                    class="tab"
                    class:selected={selectedTab === tab}
                    on:click={() => {
                        selectedTab = tab;
                    }}
                >
                    <p>{tab.name}</p>
                </div>
            {/each}

            {#if lastSavedTime}
                <div class="last-saved">
                    <Icon icon="charm:tick" size={16} />
                    <p>Last saved: {lastSavedTime}</p>
                </div>
            {/if}
        </div>
        <div class="content-container">
            <content class={selectedTab.value}>
                {#if selectedTab.value === "files"}
                    <MusicTab
                        recordings={songProjectClone?.recordings}
                        {addRecording}
                        {deleteRecording}
                        {songProject}
                        {onSelectSong}
                        showDropPlaceholder={isDragging}
                    />
                {:else if selectedTab.value === "lyrics"}
                    <LyricsTab
                        lyrics={songProjectClone?.lyrics}
                        {onLyricsUpdated}
                        enabled={songProjectClone?.songFilepath !== undefined}
                    />
                {:else if selectedTab.value === "other"}
                    <OtherTab
                        items={songProjectClone.otherContentItems}
                        enabled={songProjectClone?.songFilepath !== undefined}
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

        header {
            display: grid;
            grid-template-columns: 1fr auto;
            align-items: center;
            padding: 2em 2em 1em 2em;
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

        .credits-details {
            display: flex;
            flex-direction: row;
            flex-wrap: wrap;
            border-bottom: 1px solid
                color-mix(in srgb, var(--type-bw-inverse) 8%, transparent);
        }
        .credits,
        .details {
            padding: 0 2em 1em 2em;
            > div {
                display: flex;
                flex-direction: row;
                align-items: center;
                width: fit-content;
                p {
                    margin: 0;
                    white-space: nowrap;
                    font-weight: 500;
                    &.label {
                        opacity: 0.7;
                        font-weight: normal;
                    }
                }
            }
        }

        .members {
            position: relative;
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 3px;
            height: 100%;
            .member {
                p {
                    padding: 0.3em;
                    color: var(--text);
                    margin: 0;
                }
            }
        }

        .music-info {
            display: flex;
            flex-direction: row;
            justify-content: flex-start;

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
            display: grid;
            grid-template-rows: 1fr;

            .content-header {
                width: 100%;
                display: flex;
                align-items: center;
                justify-content: space-between;
                z-index: 3;
                pointer-events: visiblePainted;
            }
        }

        content {
            z-index: 0;
            position: relative;
            height: 100%;
            overflow: hidden;
        }

        .content-tabs {
            width: fit-content;
            display: flex;
            width: 100%;
            justify-content: flex-start;
            gap: 1em;
            padding: 0.3em 2em;
            border-bottom: 1px solid
                color-mix(in srgb, var(--type-bw-inverse) 8%, transparent);
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
                }
                &:active {
                    opacity: 0.4;
                }

                &.selected {
                    opacity: 1;
                    border-bottom: 2px solid white;
                }
            }
            .last-saved {
                display: flex;
                gap: 5px;
                margin: 0;
                align-self: center;
                justify-content: flex-end;
                flex: 1;
                color: var(--text-secondary);
                p {
                    margin: 0;
                    opacity: 0.6;
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
        color: var(--text);

        &::placeholder {
            color: rgb(105, 105, 105);
        }
    }

    .no-song-selected {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding-top: 2em;
        font-family: "Snake";
        h1 {
            color: var(--text);
            margin-bottom: 0.5em;
        }
        p,
        h1,
        h2,
        h3 {
            text-align: center;
        }
        h2 {
            font-size: 3em;
            margin: 0.3em;
            color: var(--text-secondary);
        }
        img {
            width: 30%;
            margin: 1em auto;
            display: flex;
            opacity: 0.7;
        }
    }
</style>
