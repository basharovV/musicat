<script lang="ts">
    import { basename, dirname } from "@tauri-apps/api/path";
    import { open } from "@tauri-apps/plugin-dialog";

    import hotkeys from "hotkeys-js";
    import type { MetadataEntry, Song, TagType, ToImport } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import toast from "svelte-french-toast";
    import { fade } from "svelte/transition";
    import {
        artworkDirectory,
        current,
        lastWrittenSongs,
        os,
        playerTime,
        popupOpen,
        rightClickedAlbum,
        rightClickedTracks,
    } from "../../data/store";
    import { focusTrap } from "../../utils/FocusTrap";
    import "../tippy.css";

    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import {
        type FetchArtworkResult,
        fetchAlbumArt,
    } from "../data/LibraryEnrichers";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";

    import { Image } from "@tauri-apps/api/image";
    import { writeFile } from "@tauri-apps/plugin-fs";
    import { Buffer } from "buffer";
    import LL from "../../i18n/i18n-svelte";
    import { clickOutside } from "../../utils/ClickOutside";
    import { searchArtworkOnBrave } from "../menu/search";
    import { animateHeight } from "../ui/AnimatedHeight";
    import Tabs from "../ui/Tabs.svelte";
    import ArtworkTab from "./ArtworkTab.svelte";
    import CountrySection from "./CountrySection.svelte";
    import FileTab from "./FileTab.svelte";
    import MetadataSection from "./MetadataSection.svelte";
    import audioPlayer from "../player/AudioPlayer";
    import { get } from "svelte/store";
    import { getAlbumId, reImport } from "../../data/LibraryUtils";
    import { getImageExtension } from "../../utils/FileUtils";

    // The artwork for this track(s)
    let artworkBuffer: Buffer;
    let artworkFileName;
    let artworkFilePath;
    let artworkFocused = false;
    let artworkFormat;
    let artworkSrc;

    // When scrolling through tracks, re-use artworks
    let previousArtworkFormat;
    let previousArtworkSrc;

    // The preliminary artwork to be set
    // (from an image on disk, or paste)
    let artworkToSetSrc = null;
    let artworkToSetFormat = null;
    let artworkToSetData: Uint8Array = null;

    /**
     * The preliminary action to be taken on the artwork
     * (replace or delete)
     */
    let artworkSetAction: "delete" | "replace" | null = null;

    /**
     * Artwork will be shown if:
     * - There is already existing artwork
     * - There is new artwork to be set
     */
    $: shouldDisplayArtwork =
        (!artworkSetAction || !artworkSetAction.startsWith("delete")) &&
        ((artworkToSetSrc && artworkToSetFormat) ||
            (previousArtworkSrc && previousArtworkFormat) ||
            (artworkSrc && artworkFormat));

    $: displayArtworkSrc =
        artworkToSetSrc ||
        (isMultiArt && "/images/multiart.png") ||
        previousArtworkSrc ||
        artworkSrc;

    let artworkFileToSet = null;

    let unlisten;

    let previousAlbum = $rightClickedTracks[0].album;

    let metadataSection: MetadataSection;
    let metadata: { mappedMetadata: MetadataEntry[]; tagType: TagType };
    let hasMetadataChanges: false;

    type ActionType = "artwork";

    let loadingType: ActionType = null;
    let result: FetchArtworkResult;
    let resultType: ActionType;

    $: hasChanges = !!artworkSetAction || hasMetadataChanges;

    function completeMetadataEvent(event, writeType: WriteMetadataType) {
        if (writeType === "tags") {
            event.artwork_file = "";
            event.artwork_data = [];
            event.artwork_data_mime_type = "";
            event.delete_artwork = false;
        } else {
            event.artwork_file = artworkFileToSet ? artworkFileToSet : "";
            event.artwork_data = artworkToSetData ?? [];
            event.artwork_data_mime_type = artworkToSetFormat;
            event.delete_artwork = artworkSetAction === "delete";
        }

        return event;
    }

    function deleteArtwork() {
        if (artworkToSetSrc) {
            // Undo existing set
            artworkToSetSrc = null;
            artworkToSetFormat = null;
            artworkToSetData = null;
            artworkSetAction = null;
        } else {
            // Or prepare to delete
            artworkSetAction = "delete";
        }
    }

    async function getArtwork() {
        if (!$rightClickedTracks.length) {
            return;
        }
        console.log("getting artwork", $rightClickedTracks);

        let path = $rightClickedTracks[0].path;
        if (path) {
            const songWithArtwork = await invoke<Song>("get_song_metadata", {
                event: {
                    path,
                    isImport: false,
                    includeFolderArtwork: true,
                    includeRawTags: false,
                },
            });
            console.log(songWithArtwork);

            if (!songWithArtwork) {
                toast.error(
                    `Error reading file ${path}. Check permissions, or if the file is used by another program.`,
                    { className: "app-toast" },
                );
                metadata = { mappedMetadata: [], tagType: null };
            }

            if (songWithArtwork?.artwork) {
                console.log("artwork");
                artworkFormat = songWithArtwork.artwork.format;
                if (songWithArtwork.artwork.data.length) {
                    artworkBuffer = Buffer.from(songWithArtwork.artwork.data);
                    artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                        "base64",
                    )}`;
                } else if (songWithArtwork.artwork.src) {
                    artworkSrc =
                        convertFileSrc(songWithArtwork.artwork.src) +
                        "?v=" +
                        new Date().getTime(); // cache bust
                    artworkFilePath = songWithArtwork.artwork.src;
                    artworkFileName = await basename(
                        songWithArtwork.artwork.src,
                    );
                }

                previousArtworkFormat = artworkFormat;
                previousArtworkSrc = artworkSrc;
            } else {
                artworkSrc = null;
            }

            // Avoid a 'flash' while scrolling through same album (with same art from same source)
            if (previousAlbum === $rightClickedTracks[0].album) {
                previousArtworkFormat = artworkFormat;
                previousArtworkSrc = artworkSrc;
            } else {
                previousArtworkFormat = null;
                previousArtworkSrc = null;
            }
        } else {
            // artworkSrc = null;
        }
    }

    function hasArtworkToSet() {
        return (
            artworkFileToSet ||
            artworkToSetData ||
            artworkSetAction === "delete"
        );
    }

    function onClose() {
        $popupOpen = null;
    }

    async function openArtworkFilePicker() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: $artworkDirectory,
            filters: [
                {
                    name: "default",
                    extensions: ["jpeg", "jpg", "png"],
                },
            ],
        });
        if (Array.isArray(selected)) {
            // user selected multiple files
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            // user selected a single file

            // save current directory
            $artworkDirectory = await dirname(selected);

            const src = convertFileSrc(selected);
            const response = await fetch(src);

            if (response.status === 200) {
                const type = response.headers.get("Content-Type");
                artworkFormat = type;
                artworkSrc = src;
                artworkSetAction = "replace";
                console.log("Setting src", src);
                artworkFileToSet = selected;
                setArtworkFromBuffer(await response.arrayBuffer(), type);
            }
        }
    }

    async function reset() {
        updateArtwork();
        previousAlbum = $rightClickedTracks[0].album;
        metadataSection?.resetMetadata();
    }

    function rollbackArtwork() {
        artworkToSetFormat = null;
        artworkToSetSrc = null;
        artworkToSetData = null;
    }

    async function updateArtwork() {
        artworkFormat = null;
        artworkSrc = null;
        artworkToSetFormat = null;
        artworkToSetSrc = null;
        artworkToSetData = null;
        artworkFileName = null;
        artworkFilePath = null;
        artworkSetAction = null;
        artworkFileToSet = null;

        await getArtwork();
    }

    // Shortcuts
    let modifier = $os === "macos" ? "cmd" : "ctrl";
    hotkeys.filter = (event) => true;

    hotkeys(`${modifier}+enter`, "track-info", (event, handler) => {
        if (hasChanges) {
            metadataSection?.writeMetadata();
        }
    });
    hotkeys("esc", "track-info", (event) => {
        event.stopImmediatePropagation();
        onClose();
    });

    $: {
        if ($rightClickedTracks[0]) {
            reset();
        }
    }

    async function onPaste(event: ClipboardEvent) {
        if (!artworkFocused) {
            return;
        }

        let img: Image;
        let arrayBuffer: ArrayBuffer;

        let mimeType: string;
        // Find file
        if (event.clipboardData.files.length > 0) {
            try {
                const file = event.clipboardData.files.item(0);
                mimeType = file.type;
                arrayBuffer = await file.arrayBuffer();
            } catch (err) {
                console.error(err);
            }
        }

        if (arrayBuffer && mimeType) {
            setArtworkFromBuffer(arrayBuffer, mimeType);
            artworkSetAction = "replace";
        }
    }

    function setArtworkFromBuffer(imageBuffer: ArrayBuffer, mimeType: string) {
        const b64 = Buffer.from(imageBuffer).toString("base64");
        // Convert to base64 for src
        const base64 = `data:${mimeType};base64, ${b64}`;
        artworkToSetSrc = base64;
        artworkToSetFormat = mimeType;
        artworkToSetData = new Uint8Array(imageBuffer);
    }

    type WriteMetadataType = "tags" | "artwork";

    /**
     * Send an event to the backend to write the new metadata, overwriting any existing tags.
     */
    export async function writeMetadata(type: WriteMetadataType = "tags") {
        const toWrite = metadata.mappedMetadata
            .filter((m) => m.originalValue !== m.value) // only changed values
            .map((m) => {
                if (m.originalValue?.length && !m.value) {
                    m.value = null;
                }
                return m;
            })
            .map(({ id, value }) => ({
                id: id,
                value: value?.length ? value : null,
            }));

        const writtenTracks = $rightClickedTracks;
        const tracks = [];
        const songIdToOldAlbumId = {};

        for (const song of writtenTracks) {
            tracks.push(
                completeMetadataEvent(
                    {
                        song_id: song.id,
                        metadata: type === "artwork" ? [] : toWrite,
                        tag_type: metadata.tagType,
                        file_path: song.path,
                    },
                    type,
                ),
            );

            songIdToOldAlbumId[song.id] = getAlbumId(song);
        }
        console.log("Writing: ", tracks);

        if (tracks.length) {
            const toImport = await invoke<ToImport>("write_metadatas", {
                event: {
                    tracks,
                },
            });

            console.log("To reimport: ", toImport);

            if (toImport) {
                if (toImport.error) {
                    console.error("Error writing metadata: ", toImport.error);
                    // Show error
                    toast.error(toImport.error);
                    // Roll back to current artwork
                    rollbackArtwork();
                } else {
                    reImport(toImport, writtenTracks, songIdToOldAlbumId);

                    toast.success("Successfully written metadata!", {
                        position: "top-right",
                    });
                }
            }

            $lastWrittenSongs = writtenTracks;

            const id = $current.song?.id;

            if (id && writtenTracks.some((t) => t.id == id)) {
                // Update sidebar infos
                $current = $current;

                // If we changed the artwork tag, this offsets the audio data in the file,
                // so we need to reload the song and seek to the current position
                // (will cause audible gap)
                if (hasArtworkToSet()) {
                    audioPlayer.setSeek(get(playerTime));
                }
            }

            await reset();
        } else {
            toast.error("No songs to write metadata into!", {
                position: "top-right",
            });
        }
    }

    function areAllTracksInSameFolder() {
        if ($rightClickedTracks?.length > 1) {
            const firstTrackFolder = $rightClickedTracks[0].path
                .split("/")
                .slice(0, -1)
                .join("/");
            return $rightClickedTracks.every((track) => {
                const folder = track.path.split("/").slice(0, -1).join("/");
                console.log("folder", folder);
                return folder === firstTrackFolder;
            });
        }
        return true;
    }

    $: isMultiArt =
        $rightClickedTracks?.length > 1 && !areAllTracksInSameFolder();

    async function save() {
        await writeMetadata("tags");
    }
    async function saveArtworkToFile() {
        await writeMetadata("artwork");
    }

    async function saveArtworkToFolder() {
        if (!areAllTracksInSameFolder()) {
            console.error(
                "Can't save folder artwork. Tracks are not all in the same folder",
            );
            toast.error(
                "Can't save artwork. Tracks need to all be in the same folder",
            );
            return;
        }
        console.log("save artwork to folder", artworkSetAction);
        switch (artworkSetAction) {
            case "delete":
                await invoke("delete_files", {
                    event: {
                        files: [artworkFilePath],
                    },
                });
                break;
            case "replace":
                let artPath = artworkFilePath;
                if (!artPath) {
                    const filePath = $rightClickedTracks[0].path;
                    const folder = filePath.split("/").slice(0, -1).join("/");
                    artPath = `${folder}/cover.${getImageExtension(artworkToSetFormat)}`;
                }
                console.log(
                    "save artwork to folder",
                    artPath,
                    artworkToSetData,
                );
                await writeFile(artPath, artworkToSetData);
        }
        toast.success("Artwork saved!", {
            position: "top-right",
        });

        await updateArtwork();
    }

    function searchArtwork() {
        if ($rightClickedAlbum) {
            searchArtworkOnBrave($rightClickedAlbum);
        } else {
            searchArtworkOnBrave($rightClickedTracks[0]);
        }
    }

    let previousScope;

    onMount(async () => {
        previousScope = hotkeys.getScope();
        hotkeys.setScope("track-info");
        document.addEventListener("paste", onPaste);
    });

    onDestroy(() => {
        if (unlisten) {
            unlisten();
        }
        hotkeys.unbind(`${modifier}+enter`, "track-info");
        hotkeys.unbind("esc", "track-info");
        hotkeys.deleteScope("track-info", previousScope);
        $popupOpen = null;
        document.removeEventListener("paste", onPaste);
    });

    async function fetchArtwork() {
        if (loadingType === "artwork") {
            return;
        }

        loadingType = "artwork";
        resultType = "artwork";
        result = null;

        result = await fetchAlbumArt(
            $rightClickedAlbum,
            $rightClickedTracks[0],
        );

        if (result.image) {
            toast.success("Found album art!", {
                position: "top-right",
            });
            // Update src
            setArtworkFromBuffer(result.image, result.mimeType);
        } else if (result.error) {
            toast.error("Couldn't find album art", {
                position: "top-right",
            });
        }

        loadingType = null;
    }

    const tabs = [
        { id: "file", label: "File Info", icon: "bi:file-earmark-play" },
        { id: "metadata", label: "Metadata", icon: "fe:music" },
        { id: "artwork", label: "Artwork", icon: "bi:file-earmark-play" },
        { id: "enrichment", label: "Enrichment", icon: "iconoir:atom" },
    ];

    let currentTab = tabs[1].id;

    $: saveTooltip = (() => {
        if (!metadata?.mappedMetadata) return "";

        const changes = metadata.mappedMetadata.filter(
            (m) => m.originalValue !== m.value,
        );

        if (!changes.length && !artworkSetAction) return "";

        const added = changes.filter(
            (m) => !m.originalValue?.length && m.value?.length,
        );
        const removed = changes.filter(
            (m) => m.originalValue?.length && !m.value?.length,
        );
        const modified = changes.filter(
            (m) => m.originalValue?.length && m.value?.length,
        );

        const badge = (type: "added" | "removed" | "modified") => {
            const styles: Record<string, string> = {
                added: "background:#1a472a;color:#69db7c;",
                removed: "background:#3b0d0d;color:#ff6b6b;",
                modified: "background:#3a2a00;color:#fcc419;",
            };
            const labels = {
                added: $LL.trackInfo.metadata.saveTooltip.added().toUpperCase(),
                removed: $LL.trackInfo.metadata.saveTooltip
                    .removed()
                    .toUpperCase(),
                modified: $LL.trackInfo.metadata.saveTooltip
                    .modified()
                    .toUpperCase(),
            };
            return `<span style="display:inline-block;padding:1px 6px;border-radius:4px;font-size:10px;font-weight:700;letter-spacing:.5px;${styles[type]}">${labels[type]}</span>`;
        };

        const row = (
            type: "added" | "removed" | "modified",
            id: string,
            from?: string,
            to?: string,
        ) => {
            const colors = {
                added: "#69db7c",
                removed: "#ff6b6b",
                modified: "#fcc419",
            };
            const color = colors[type];
            const detail =
                type === "added"
                    ? `<span style="color:#69db7c">${to}</span>`
                    : type === "removed"
                      ? `<span style="color:#ff6b6b;text-decoration:line-through">${from}</span>`
                      : `<span style="color:#aaa">${from}</span> <span style="color:#555">→</span> <span style="color:#fcc419">${to}</span>`;
            return `<li style="display:flex;align-items:center;gap:8px;padding:4px 0;border-bottom:1px solid #ffffff0f">
            ${badge(type)}
            <span style="color:${color};font-weight:600;min-width:80px;font-size:11px">${id}</span>
            <span style="color:#ccc;font-size:11px;flex:1">${detail}</span>
        </li>`;
        };

        const rows = [
            ...added.map((m) => row("added", m.id, undefined, m.value)),
            ...removed.map((m) => row("removed", m.id, m.originalValue)),
            ...modified.map((m) =>
                row("modified", m.id, m.originalValue, m.value),
            ),
        ];

        const summaryParts = [
            added.length
                ? `<span style="color:#69db7c">+${added.length} added</span>`
                : "",
            removed.length
                ? `<span style="color:#ff6b6b">−${removed.length} removed</span>`
                : "",
            modified.length
                ? `<span style="color:#74c0fc">~${modified.length} changed</span>`
                : "",
        ]
            .filter(Boolean)
            .join("<span style='color:#555;margin:0 4px'>·</span>");

        return `<div style="font-family:monospace;min-width:320px;max-width:480px;padding:0.5em 1em 0 0">
        <div style="display:flex;align-items:center;justify-content:space-between;padding-bottom:6px;margin-bottom:4px;border-bottom:2px solid #ffffff15">
            <span style="color:#fff;font-weight:700;font-size:12px;letter-spacing:.5px">${$LL.trackInfo.metadata.saveTooltip.title().toUpperCase()}</span>
            <span style="font-size:11px">${summaryParts}</span>
        </div>
        <ul style="list-style:none;margin:0;padding:0">
            ${rows.join("")}
        </ul>
        <div style="margin-top:8px;text-align:right;color:#555;font-size:10px">${$LL.trackInfo.metadata.saveTooltip.hint()}</div>
    </div>`;
    })();
</script>

<container use:focusTrap use:clickOutside={onClose}>
    <header>
        <div class="top">
            <div class="close">
                <Icon icon="mingcute:close-circle-fill" onClick={onClose} />
                <small>ESC</small>
            </div>
            <div
                class="button-container"
                class:hidden={currentTab !== "metadata"}
            >
                <ButtonWithIcon
                    disabled={!hasChanges}
                    onClick={save}
                    text={`${$LL.trackInfo.save()}`}
                    tooltip={{
                        show: hasChanges,
                        content: saveTooltip,
                        allowHTML: true,
                        placement: "bottom",
                    }}
                />
            </div>

            <div class="title-container">
                <h3>
                    {$rightClickedTracks?.length > 1
                        ? `${$rightClickedTracks?.length} tracks selected`
                        : $rightClickedTracks[0].title}
                </h3>
                <small class="subtitle">{$LL.trackInfo.subtitle()}</small>
            </div>
        </div>
        <div class="tabs">
            <Tabs {tabs} bind:currentTab />
        </div>
    </header>
    <div class="content" use:animateHeight>
        {#key currentTab}
            <div in:fade={{ duration: 150, delay: 100 }}>
                {#if currentTab === "file"}
                    <FileTab />
                {:else if currentTab === "metadata"}
                    <MetadataSection
                        bind:data={metadata}
                        bind:this={metadataSection}
                        bind:hasChanges={hasMetadataChanges}
                        {writeMetadata}
                    />
                {:else if currentTab === "artwork"}
                    <ArtworkTab
                        bind:artworkFocused
                        {shouldDisplayArtwork}
                        {displayArtworkSrc}
                        {artworkSrc}
                        {artworkFormat}
                        {artworkToSetSrc}
                        {artworkToSetFormat}
                        {deleteArtwork}
                        {openArtworkFilePicker}
                        {artworkSetAction}
                        {isMultiArt}
                        {artworkFileName}
                        {loadingType}
                        {fetchArtwork}
                        {searchArtwork}
                        {saveArtworkToFile}
                        {saveArtworkToFolder}
                    />
                {:else if currentTab === "enrichment"}
                    <CountrySection />
                {/if}
            </div>
        {/key}
    </div>
</container>

<style lang="scss">
    :global {
        .info {
            section.boxed {
                position: relative;
            }

            .section-title {
                position: absolute;
                border-radius: 4px;
                max-height: 2em;
                display: flex;
                flex-direction: row;
                gap: 5px;
                align-items: center;
                background-color: var(--popup-section-title-bg);
                z-index: 11;
                border: 1px solid rgb(from var(--inverse) r g b / 0.08);
                top: -15px;
                padding: 0 10px 0 0;
                letter-spacing: 1px;
                font-weight: 400;
                left: 1.2em;
                right: 0;
                width: fit-content;
                margin: 0.5em 0;
                text-align: start;
                color: var(--popup-section-title-text);
                text-transform: uppercase;
                > :first-child {
                    margin-left: -4px;
                }
            }
        }
    }
    :global([data-tippy-root]) {
        white-space: pre-line;
    }
    * {
        user-select: none;
    }
    container {
        width: fit-content;
        max-height: 85%;
        margin: auto;
        @media screen and (max-width: 750px) {
            margin: auto 1em;
        }
        display: flex;
        flex-direction: column;
        position: relative;
        align-items: center;
        border-radius: 5px;
        border: 1px solid color-mix(in srgb, var(--inverse) 20%, transparent);
        background-color: var(--popup-body-bg);
        box-shadow: 0px 5px 40px var(--overlay-shadow);
        backdrop-filter: blur(8px);
        overflow-y: auto;
        overflow-x: hidden;
        font-family:
            system-ui,
            -apple-system,
            Avenir,
            Helvetica,
            Arial,
            sans-serif;

        @media only screen and (max-width: 400px) {
            display: none;
        }
    }

    header {
        display: grid;
        grid-template-columns: auto 1fr auto;
        grid-template-rows: auto auto;
        align-items: center;
        position: sticky;
        top: -1px;
        width: 100%;
        backdrop-filter: blur(10px);
        z-index: 20;

        .top {
            grid-row: 1;
            grid-column: 1 /4;
            display: flex;
            align-items: center;
            width: 100%;
            background-color: var(--popup-header-bg-many);
            border-bottom: 1px solid var(--popup-header-border);
            padding: 0.6em 1em;
        }

        .title-container {
            flex: 1;
            order: 1;
            h2 {
                font-weight: 200;
            }
            > .subtitle {
                opacity: 0.5;
                display: block;
                margin: 0;
                line-height: 1.8em;
                font-family:
                    system-ui,
                    -apple-system,
                    Avenir,
                    Helvetica,
                    Arial,
                    sans-serif;
            }
        }

        @media only screen and (max-width: 700px) {
            .title-container {
                display: flex;
                flex-direction: column;
                align-items: flex-start;
                align-self: flex-start;
                margin-left: 4em;
                h2 {
                    margin: 0.5em;
                }
                small {
                    display: none;
                }
            }

            .button-container {
                top: 0.4em;
                right: 15px;
            }
            .close {
                top: 1.5em;
            }
        }

        @media only screen and (min-width: 701px) {
            .title-container {
                display: flex;
                flex-direction: column;
                h3 {
                    margin: 0;
                }
                small {
                    margin: 0;
                }
            }
        }

        .tabs {
            width: 100%;
            grid-row: 2;
            grid-column: 1 / 4;
            background: var(--popup-header-bg-lone);
        }
    }

    .content {
        margin: 1em 0 1em 0;
        min-width: 470px;
        overflow: hidden;
        transition: height 200ms cubic-bezier(0.61, 0.26, 0.46, 0.95);
    }

    .button-container {
        order: 2;
        position: relative;
        top: -0.4em;
        small {
            position: absolute;
            top: 3em;
            right: 0;
            left: 0;
            opacity: 0.3;
            font-size: 0.8em;
        }
        &.hidden {
            visibility: hidden;
        }
    }

    .close {
        order: 0;
        z-index: 20;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 10px;
        small {
            opacity: 0.3;
        }
    }

    .top {
        padding: 1.5em 1em 0;
        display: grid;
        grid-template-columns: 1fr 150px;
        column-gap: 1em;
        width: 100%;
    }

    .bottom {
        padding: 0 1em 1em;
        width: 100%;
    }
</style>
