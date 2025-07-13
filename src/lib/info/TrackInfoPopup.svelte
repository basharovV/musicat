<script lang="ts">
    import { dirname, join, pictureDir } from "@tauri-apps/api/path";
    import { open } from "@tauri-apps/plugin-dialog";
    import { open as fileOpen } from "@tauri-apps/plugin-shell";
    import { basename } from "@tauri-apps/api/path";
    import { remove as removeFile } from "@tauri-apps/plugin-fs";

    import hotkeys from "hotkeys-js";
    import type { Song } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { fade, fly } from "svelte/transition";
    import {
        artworkDirectory,
        os,
        popupOpen,
        rightClickedAlbum,
        rightClickedTrack,
        rightClickedTracks,
    } from "../../data/store";
    import { focusTrap } from "../../utils/FocusTrap";
    import "../tippy.css";

    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import {
        type EnricherResult,
        fetchAlbumArt,
    } from "../data/LibraryEnrichers";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";

    import { Image } from "@tauri-apps/api/image";
    import { Buffer } from "buffer";
    import LL from "../../i18n/i18n-svelte";
    import MetadataSection from "./MetadataSection.svelte";
    import CountrySection from "./CountrySection.svelte";
    import { searchArtworkOnBrave } from "../menu/search";
    import { clickOutside } from "../../utils/ClickOutside";

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

    // The preliminary artwork to be set (from an image on disk)
    let artworkToSetSrc = null;
    let artworkToSetFormat = null;
    let artworkToSetData: Uint8Array = null;
    let isArtworkSet:
        | "delete-file"
        | "delete-metadata"
        | "replace-file"
        | "replace-metadata"
        | false = false;
    let artworkFileToSet = null;

    let unlisten;

    let previousAlbum = ($rightClickedTrack || $rightClickedTracks[0]).album;

    let metadata: MetadataSection;
    let hasMetadataChanges: false;

    type ActionType = "artwork";

    let loadingType: ActionType = null;
    let result: EnricherResult;
    let resultType: ActionType;

    $: hasChanges = !!isArtworkSet || hasMetadataChanges;

    function completeMetadataEvent(event) {
        event.artwork_file = artworkFileToSet ? artworkFileToSet : "";
        event.artwork_data = artworkToSetData ?? [];
        event.artwork_data_mime_type = artworkToSetFormat;
        event.delete_artwork = isArtworkSet === "delete-metadata";

        return event;
    }

    function deleteArtwork() {
        if (artworkFileName) {
            isArtworkSet = "delete-file";
        } else if (artworkToSetSrc) {
            artworkToSetSrc = null;
            artworkToSetFormat = null;
        } else {
            isArtworkSet = "delete-metadata";
        }
    }

    async function getArtwork() {
        if ($rightClickedTrack === null && !$rightClickedTracks.length) {
            return;
        }
        console.log("getting artwork", $rightClickedTrack, $rightClickedTracks);

        let path = ($rightClickedTrack || $rightClickedTracks[0]).path;
        if (path) {
            const songWithArtwork = await invoke<Song>("get_song_metadata", {
                event: { path, isImport: false, includeFolderArtwork: true },
            });
            console.log(songWithArtwork);

            if (!songWithArtwork) {
                toast.error(
                    `Error reading file ${path}. Check permissions, or if the file is used by another program.`,
                    { className: "app-toast" },
                );
                metadata.data = { mappedMetadata: [], tagType: null };
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
                    artworkSrc = convertFileSrc(songWithArtwork.artwork.src);
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
            if (
                previousAlbum ===
                ($rightClickedTrack || $rightClickedTracks[0]).album
            ) {
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

    function getDurationText(durationInSeconds: number) {
        return `${(~~(durationInSeconds / 60))
            .toString()
            .padStart(2, "0")}:${(~~(durationInSeconds % 60))
            .toString()
            .padStart(2, "0")}`;
    }

    function hasArtworkToSet() {
        return (
            artworkFileToSet ||
            artworkToSetData ||
            isArtworkSet === "delete-metadata"
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
                isArtworkSet =
                    isArtworkSet === "delete-file" || artworkFileName
                        ? "replace-file"
                        : "replace-metadata";
                artworkFileToSet = selected;
                artworkToSetData = null;
                artworkToSetSrc = src;
                artworkToSetFormat = type;
            }
        }
    }

    async function reset() {
        updateArtwork();
        previousAlbum = ($rightClickedTrack || $rightClickedTracks[0]).album;
        metadata?.resetMetadata();
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
        isArtworkSet = false;
        artworkFileToSet = null;

        await getArtwork();
    }

    // Shortcuts
    let modifier = $os === "macos" ? "cmd" : "ctrl";
    hotkeys(`${modifier}+enter`, "track-info", (event, handler) => {
        if (hasChanges) {
            metadata?.writeMetadata();
        }
    });
    hotkeys("esc", "track-info", () => {
        onClose();
    });

    $: {
        if ($rightClickedTrack || $rightClickedTracks[0]) {
            reset();
        }
    }

    $: isMultiMode = $rightClickedTracks?.length;

    // File(s) table
    let tableOuterContainer;
    let tableInnerScrollArea;
    let showTableTopScrollShadow = false;
    let showTableBottomScrollShadow = false;

    function onTableResize() {
        console.log("scrollTop");
        // Check scroll area size, add shadows if necessary
        if (tableInnerScrollArea) {
            showTableTopScrollShadow =
                tableInnerScrollArea.scrollTop > 0 &&
                tableInnerScrollArea.scrollHeight >
                    tableInnerScrollArea.clientHeight;
            showTableBottomScrollShadow =
                tableInnerScrollArea.scrollHeight >
                tableInnerScrollArea.clientHeight;
        }
    }

    function onTableScroll(e) {
        const tableHeight = tableOuterContainer.clientHeight;
        const scrollPercentage =
            tableInnerScrollArea.scrollTop /
            (tableInnerScrollArea.scrollHeight -
                tableInnerScrollArea.clientHeight);
        // Show top shadow
        showTableTopScrollShadow =
            tableInnerScrollArea.scrollTop > 0 &&
            tableInnerScrollArea.scrollHeight >
                tableInnerScrollArea.clientHeight;
    }

    async function onPaste(event: ClipboardEvent) {
        if (!artworkFocused) {
            return;
        }

        let img: Image;
        let arrayBuffer: ArrayBuffer;

        let mimeType: string;
        if (event.clipboardData.items[0]) {
            try {
                mimeType = event.clipboardData.items[0].type;
                arrayBuffer = await event.clipboardData.items[0]
                    .getAsFile()
                    .arrayBuffer();
                console.log("mimeType", mimeType);
            } catch (err) {
                console.error(err);
            }
        }

        // try {
        //     img = await readImage();
        //     console.log("paste", await img.size());
        // } catch (err) {
        //     console.error(err);
        // }

        if (arrayBuffer && mimeType) {
            // const rgba = await img.rgba();
            const b64 = Buffer.from(arrayBuffer).toString("base64");
            // Convert to base64 for src
            const base64 = `data:${mimeType};base64, ${b64}`;
            artworkToSetSrc = base64;
            artworkToSetFormat = mimeType;
            artworkToSetData = new Uint8Array(arrayBuffer);
            artworkFileToSet = null;
            isArtworkSet =
                isArtworkSet === "delete-file" || artworkFileName
                    ? "replace-file"
                    : "replace-metadata";
        }
        // const src = "asset://localhost/" + folder + artworkFileName;
    }

    async function save() {
        if (
            $rightClickedAlbum &&
            $rightClickedAlbum.tracksIds.length ==
                $rightClickedTracks?.length &&
            (isArtworkSet === "delete-file" || isArtworkSet === "replace-file")
        ) {
            await invoke("delete_files", {
                event: {
                    files: [artworkFilePath],
                },
            });
        }

        await metadata.writeMetadata();
    }

    function searchArtwork() {
        if ($rightClickedTrack) {
            searchArtworkOnBrave($rightClickedTrack);
        } else if ($rightClickedAlbum) {
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

        onTableResize();
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
            $rightClickedTrack || $rightClickedTracks[0],
        );

        if (result.success) {
            toast.success("Found album art and written to album!", {
                position: "top-right",
            });
            updateArtwork();
        } else if (result.error) {
            toast.error("Couldn't find album art", {
                position: "top-right",
            });
        }

        loadingType = null;
    }
</script>

<container use:focusTrap use:clickOutside={onClose}>
    <header>
        <div class="close">
            <Icon icon="mingcute:close-circle-fill" onClick={onClose} />
            <small>ESC</small>
        </div>
        <div class="button-container">
            <ButtonWithIcon
                disabled={!hasChanges}
                onClick={save}
                text={`Overwrite file${isMultiMode ? "s" : ""}`}
            />
            <small>Cmd + ENTER</small>
        </div>

        <div class="title-container">
            <h2>
                {$rightClickedTracks?.length
                    ? `${$rightClickedTracks?.length} tracks selected`
                    : "Track info"}
            </h2>
            <small class="subtitle">{$LL.trackInfo.subtitle()}</small>
        </div>
    </header>
    <div class="top">
        <div>
            <section class="file-section boxed" bind:this={tableOuterContainer}>
                <h5 class="section-title">
                    <Icon icon="bi:file-earmark-play" size={26} />File info
                </h5>

                {#if $rightClickedTrack || $rightClickedTracks.length}
                    {#if showTableTopScrollShadow}
                        <div in:fade={{ duration: 150 }} class="top-shadow" />
                    {/if}
                    {#if showTableBottomScrollShadow}
                        <div
                            in:fly={{ duration: 150, y: 20 }}
                            class="bottom-shadow"
                        />
                    {/if}
                    <div
                        class="file-info-container"
                        bind:this={tableInnerScrollArea}
                        on:scroll={onTableScroll}
                    >
                        <table class="file-info">
                            <thead>
                                <tr>
                                    <td>{$LL.trackInfo.file()}</td>
                                    <td>{$LL.trackInfo.codec()}</td>
                                    <td>{$LL.trackInfo.tagType()}</td>
                                    <td>{$LL.trackInfo.duration()}</td>
                                    <td>{$LL.trackInfo.sampleRate()}</td>
                                    <td>{$LL.trackInfo.bitRate()}</td>
                                </tr>
                            </thead>
                            {#each $rightClickedTrack ? [$rightClickedTrack] : $rightClickedTracks as track}
                                <tr>
                                    <td class="file-path">
                                        <div
                                            on:click={() =>
                                                fileOpen(
                                                    track.path.replace(
                                                        track.file,
                                                        "",
                                                    ),
                                                )}
                                        >
                                            <span>
                                                <Icon
                                                    icon="bi:file-earmark-play"
                                                    size={12}
                                                />
                                            </span>
                                            {track.file}
                                        </div>
                                    </td>
                                    <td>
                                        <p>{track.fileInfo.codec}</p>
                                    </td>
                                    <td>
                                        <p>{track.fileInfo.tagType}</p>
                                    </td>
                                    <td>
                                        <p>
                                            {getDurationText(
                                                track.fileInfo.duration,
                                            )}
                                        </p>
                                    </td>
                                    <td>
                                        <p>{track.fileInfo.sampleRate} hz</p>
                                    </td>
                                    <td>
                                        {#if track.fileInfo.bitDepth}
                                            <p>
                                                {track.fileInfo.bitDepth}
                                                {$LL.trackInfo.bit()}
                                            </p>
                                        {/if}
                                    </td>
                                </tr>
                            {/each}
                        </table>
                    </div>
                {:else}
                    <p>{$LL.trackInfo.noMetadata()}</p>
                {/if}
            </section>

            <CountrySection />
        </div>

        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <section class="artwork-section">
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                contenteditable
                class="artwork-container"
                class:focused={artworkFocused}
                on:mouseenter={() => (artworkFocused = true)}
                on:mouseleave={() => (artworkFocused = false)}
                use:tippy={{
                    content: $LL.trackInfo.artworkTooltip(),
                    placement: "bottom",
                    trigger: "focusin",
                }}
            >
                <div class="artwork-frame">
                    {#if (!isArtworkSet || !isArtworkSet.startsWith("delete")) && ((artworkToSetSrc && artworkToSetFormat) || (previousArtworkSrc && previousArtworkFormat) || (artworkSrc && artworkFormat))}
                        <img
                            alt=""
                            class="artwork"
                            src={artworkToSetSrc ||
                                previousArtworkSrc ||
                                artworkSrc}
                        />
                        {#if artworkFocused && artworkSrc && artworkFormat}
                            <div class="artwork-options">
                                <Icon
                                    icon={artworkToSetSrc
                                        ? "mingcute:close-circle-fill"
                                        : "ant-design:delete-outlined"}
                                    onClick={deleteArtwork}
                                />
                                <Icon
                                    icon="material-symbols:folder"
                                    onClick={openArtworkFilePicker}
                                />
                            </div>
                        {/if}
                    {:else}
                        <div class="artwork-placeholder">
                            {#if artworkFocused}
                                <div
                                    class="artwork-options"
                                    on:click={openArtworkFilePicker}
                                >
                                    <Icon icon="material-symbols:folder" />
                                </div>
                            {:else}
                                <Icon icon="mdi:music-clef-treble" />
                            {/if}
                        </div>
                    {/if}
                </div>
            </div>
            {#if isArtworkSet}
                <small>{$LL.trackInfo.artworkReadyToSave()}</small>
            {:else if artworkFileName}
                <small>{artworkFileName}</small>
            {:else if artworkSrc}
                <small>{$LL.trackInfo.encodedInFile()}</small>
            {:else}
                <small class="notfound">{$LL.trackInfo.noArtwork()}</small>
            {/if}
            <span
                use:tippy={{
                    allowHTML: true,
                    content: $LL.trackInfo.artworkTooltipBody(),
                    placement: "left",
                }}
                ><Icon icon="ic:round-info" /><small
                    >{$LL.trackInfo.aboutArtwork()}</small
                ></span
            >
            <div class="find-art-btn">
                <ButtonWithIcon
                    disabled={isArtworkSet || artworkSrc}
                    icon={loadingType === "artwork"
                        ? "line-md:loading-loop"
                        : "ic:twotone-downloading"}
                    text="Fetch art"
                    theme="transparent"
                    onClick={fetchArtwork}
                />
                <ButtonWithIcon
                    icon="mdi:search-web"
                    text="Search art"
                    theme="transparent"
                    onClick={searchArtwork}
                />
            </div>
        </section>
    </div>
    <div class="bottom">
        <MetadataSection
            bind:this={metadata}
            bind:hasChanges={hasMetadataChanges}
            completeEvent={completeMetadataEvent}
            {hasArtworkToSet}
            {reset}
            {rollbackArtwork}
        />
    </div>
</container>

<style lang="scss">
    :global {
        .info {
            section.boxed {
                border: 1px solid var(--popup-section-border);
                border-radius: 5px;
                min-width: 575px;
                position: relative;
                background-color: var(--popup-section-bg);
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
                padding: 0 10px;
                letter-spacing: 1px;
                font-weight: 400;
                left: 3em;
                right: 0;
                width: fit-content;
                margin: 0.5em 0;
                text-align: start;
                color: var(--popup-section-title-text);
                text-transform: uppercase;
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
        max-width: 750px;
        min-width: 750px;
        margin: auto;
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
        display: flex;
        flex-direction: column;
        align-items: center;
        position: sticky;
        top: -1px;
        padding: 0.4em 0;
        width: 100%;
        background-color: var(--popup-header-bg-many);
        border-bottom: 1px solid var(--popup-header-border);
        backdrop-filter: blur(10px);
        z-index: 20;

        .title-container {
            > .subtitle {
                opacity: 0.5;
                display: block;
                margin: 0;
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
                small {
                    display: none;
                }
            }
            .close {
                top: 1.5em;
            }
        }

        @media only screen and (min-width: 701px) {
            .title-container {
                display: flex;
                flex-direction: column;
                align-items: center;
                text-align: center;
                h2 {
                    margin: 0.2em;
                }
                small {
                    margin: 0;
                }
            }
        }
    }

    .artwork-container {
        padding: 0em;
        width: 120px;
        height: 120px;
        border-radius: 4px;
        overflow: hidden;
        cursor: pointer;
        caret-color: transparent;
        border: 1px solid
            color-mix(in srgb, var(--background) 60%, var(--inverse));

        &:hover {
            border: 1px solid rgb(from var(--inverse) r g b / 0.517);
        }

        &.focused {
            background-image: linear-gradient(
                    90deg,
                    silver 50%,
                    transparent 50%
                ),
                linear-gradient(90deg, silver 50%, transparent 50%),
                linear-gradient(0deg, silver 50%, transparent 50%),
                linear-gradient(0deg, silver 50%, transparent 50%);
            background-repeat: repeat-x, repeat-x, repeat-y, repeat-y;
            background-size:
                15px 2px,
                15px 2px,
                2px 15px,
                2px 15px;
            background-position:
                left top,
                right bottom,
                left bottom,
                right top;
            animation: border-dance 2s infinite linear;

            .artwork-frame {
                img {
                    transform: scale(0.96);
                }
            }
        }

        @keyframes border-dance {
            0% {
                background-position:
                    left top,
                    right bottom,
                    left bottom,
                    right top;
            }

            100% {
                background-position:
                    left 15px top,
                    right 15px bottom,
                    left bottom 15px,
                    right top 15px;
            }
        }

        .artwork-frame {
            width: 100%;
            height: 100%;
            box-sizing: border-box;
            user-select: none;
            /* border-radius: 3px; */
            /* border: 1px solid rgb(94, 94, 94); */
            display: flex;
            align-items: center;
            justify-content: center;
            position: relative;
            img {
                height: 100%;
                width: 100%;
            }

            .artwork-options {
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: row;
                align-items: center;
                justify-content: center;
                gap: 2em;
                background-color: var(--background);
            }

            .artwork-placeholder {
                opacity: 0.2;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 1em;
            }
        }
    }

    .button-container {
        position: absolute;
        top: 0.8em;
        right: 15px;

        small {
            position: absolute;
            top: 3.2em;
            right: 0;
            left: 0;
            opacity: 0.3;
            font-size: 0.8em;
        }
    }

    .close {
        position: absolute;
        top: 1.5em;
        left: 1em;
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
        padding: 1em 1em 0;
        display: grid;
        grid-template-columns: 1fr 130px;
        width: 100%;
    }

    .bottom {
        padding: 0 1em 1em;
        width: 100%;
    }

    .file-section {
        padding: 4px;

        .section-title {
            top: -10px;
            margin: 0;
        }

        .top-shadow {
            font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
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
            opacity: 0.5;
        }
        .bottom-shadow {
            font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
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
            opacity: 0.5;
        }
        .file-info-container {
            max-height: 200px;
            overflow-y: auto;
            position: relative;
            padding-bottom: 1em;
            padding-top: 0.5em;
        }

        .file-info {
            table-layout: auto;
            display: table;
            justify-content: center;
            flex-wrap: wrap;
            gap: 1em;
            text-align: left;
            width: 100%;
            font-size: 13px;
            -webkit-border-horizontal-spacing: 0px;
            -webkit-border-vertical-spacing: 0px;
            border-collapse: collapse;
            /* border: 1px solid rgb(97, 92, 92); */

            > thead {
                > tr {
                }
                td {
                    opacity: 0.5;
                }
            }
            tr {
                > td {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    border-right: 5px solid transparent;

                    p {
                        background-color: var(--popup-track-data-field-bg);
                        padding: 0.2em 0.5em;
                        width: fit-content;
                        border-radius: 4px;
                        color: var(--text);
                        user-select: none;
                        cursor: default;
                        margin: 0;
                    }
                }

                .file-path {
                    margin: auto;
                    font-family: -apple-system, Avenir, Helvetica, Arial,
                        sans-serif;
                    cursor: default;
                    min-width: 170px;
                    max-width: 170px;

                    span {
                        vertical-align: middle;
                        position: relative;
                        display: flex;
                    }

                    & > div {
                        overflow: hidden;
                        text-overflow: ellipsis;
                        white-space: nowrap;
                        font-size: 13px;
                        width: 100%;
                        vertical-align: middle;
                        display: flex;
                        gap: 4px;
                    }

                    &:hover {
                        text-decoration: underline;
                    }
                }
            }
        }
    }

    .artwork-section {
        padding: 0 0 0 1em;
        > small {
            color: var(--popup-track-artwork-found);
            white-space: nowrap;
            text-overflow: ellipsis;
            overflow: hidden;
            display: block;

            &.notfound {
                color: var(--popup-track-artwork-notfound);
            }
        }
        span {
            cursor: default;
            margin: 0.3em auto 0;
            width: fit-content;
            display: flex;
            gap: 5px;
            align-items: center;
            color: var(--popup-track-artwork-about);
            user-select: none;
            &:hover {
                opacity: 0.7;
            }

            &:active {
                opacity: 0.5;
            }
        }
        .find-art-btn {
            margin-top: 1em;
        }
    }
</style>
