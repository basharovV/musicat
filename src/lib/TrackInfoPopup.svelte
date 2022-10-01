<script lang="ts">
    import { readText } from "@tauri-apps/api/clipboard";
    import { open } from "@tauri-apps/api/dialog";
    import { emit, listen } from "@tauri-apps/api/event";
    import {
        register,
        unregister,
        unregisterAll
    } from "@tauri-apps/api/globalShortcut";
    import { join, pictureDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";

    import "iconify-icon";
    import { cloneDeep, isEqual, uniqBy } from "lodash-es";
    import * as musicMetadata from "music-metadata-browser";
    import { onDestroy, onMount } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { addSong, lookForArt } from "../data/LibraryImporter";
    import { isTrackInfoPopupOpen, os, rightClickedTrack } from "../data/store";
    import { getMapForTagType, getTagTypeFromCodec } from "../data/LabelMap";
    import "./tippy.css";
    import Input from "./Input.svelte";
    import { focusTrap } from "../utils/FocusTrap";
    import type { MetadataEntry, TagType } from "src/App";
    import { db } from "../data/db";
    import { optionalTippy } from "./ui/TippyAction";
    import hotkeys from "hotkeys-js";
    import { fly } from "svelte/transition";

    // optional

    function onClose() {
        $isTrackInfoPopupOpen = false;
    }

    function addDefaults(metadata: MetadataEntry[], format: TagType) {
        // Map eg. 'TITLE' (FLAC/Vorbis) -> title (Musicat generic identifier)
        const map = getMapForTagType(format, false);
        // Add empty default fields
        const defaults: MetadataEntry[] = [];
        const others: MetadataEntry[] = [];
        for (const field of Object.entries(map)) {
            // Check if this default field already exists in the file
            const existingField = metadata.find(
                (m) => field[1] === m.genericId
            );

            defaults.push({
                id: field[0],
                genericId: field[1],
                value: existingField ? existingField.value : null
            });
        }

        for (const field of metadata) {
            const inDefaults = defaults.find(
                (t) => t.genericId === field.genericId
            );
            if (!inDefaults) {
                others.push({
                    id: field.id,
                    genericId: field.genericId,
                    value: field.value
                });
            }
        }
        return { defaults, others };
    }

    let isUnsupportedFormat = false;
    function mergeDefault(
        metadata: MetadataEntry[],
        format: TagType
    ): MetadataEntry[] {
        if ($rightClickedTrack.fileInfo.codec === "PCM") {
            isUnsupportedFormat = true;
            return []; // UNSUPPORTED FORMAT, for now...
        }

        if (!format) return [];

        isUnsupportedFormat = false;
        const cloned = cloneDeep(metadata);
        const { defaults, others } = addDefaults(cloned, format);
        return uniqBy(
            [...defaults, ...others.sort((a, b) => a.id.localeCompare(b.id))],
            "id"
        );
    }

    // console.log("track", $rightClickedTrack);
    let metadata: MetadataEntry[] = mergeDefault(
        $rightClickedTrack.metadata,
        $rightClickedTrack.fileInfo.tagTypes[0]
    );

    // let metadata: MetadataEntry[] = cloneDeep($rightClickedTrack.metadata);

    $: durationText = `${(~~($rightClickedTrack.fileInfo.duration / 60))
        .toString()
        .padStart(2, "0")}:${(~~($rightClickedTrack.fileInfo.duration % 60))
        .toString()
        .padStart(2, "0")}`;

    let artworkFormat;
    let artworkBuffer: Buffer;
    let artworkSrc;

    $: tagType = $rightClickedTrack.fileInfo?.tagTypes?.length
        ? $rightClickedTrack.fileInfo.tagTypes[0]
        : getTagTypeFromCodec($rightClickedTrack.fileInfo.codec);

    let foundArtwork;
    let isArtworkSet = false;
    let artworkFileToSet = null;

    $: hasChanges =
        isArtworkSet ||
        !isEqual(metadata, mergeDefault($rightClickedTrack.metadata, tagType));

    let artworkFocused = false;

    async function getArtwork() {
        let path = $rightClickedTrack.path;
        if (path) {
            const metadata = await musicMetadata.fetchFromUrl(
                convertFileSrc(path)
            );
            if (metadata.common.picture?.length) {
                artworkFormat = metadata.common.picture[0].format;
                artworkBuffer = metadata.common.picture[0].data;
                artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                    "base64"
                )}`;
            } else {
                foundArtwork = await lookForArt(
                    $rightClickedTrack.path,
                    $rightClickedTrack.file
                );
                if (foundArtwork) {
                    artworkSrc = foundArtwork.artworkSrc;
                    artworkFormat = foundArtwork.artworkFormat;
                }
            }
        } else {
            // artworkSrc = null;
        }
    }

    /**
     * Send an event to the backend to write the new metadata, overwriting any existing tags.
     */
    function writeMetadata() {
        const toWrite = metadata
            .filter((m) => m.value !== null)
            .map((t) => ({ id: t.id, value: t.value }));
        console.log("Writing: ", toWrite);
        emit("write-metadata", {
            metadata: toWrite,
            "tag_type": tagType,
            "file_path": $rightClickedTrack.path,
            "artwork_file_to_set": artworkFileToSet ? artworkFileToSet : ""
        });
        artworkFileToSet = null;
        isArtworkSet = false;
    }

    /**
     * @param {String} imageData a uint8 array
     * @param {String} format the image format eg. image/jpeg
     */
    function setBlockPictureTag(imageData: string, format) {
        const blockPicture = {
            colour_depth: 24,
            data: imageData,
            description: "",
            format,
            height: 500,
            indexed_color: 0,
            type: "Cover (front)",
            width: 500
        };
        // metadata.push({
        //     id: "METADATA_BLOCK_PICTURE",
        //     value: blockPicture
        // });
    }

    const imageUrlToBase64 = async (url): Promise<string> => {
        const response = await fetch(url);
        const blob = await response.blob();
        return new Promise((onSuccess, onError) => {
            try {
                const reader = new FileReader();
                reader.onload = function () {
                    onSuccess(this.result as string);
                };
                reader.readAsDataURL(blob);
            } catch (e) {
                onError(e);
            }
        });
    };

    let unlisten;

    async function onImageClick(evt) {
        // Open a selection dialog for directories
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: await pictureDir()
        });
        if (Array.isArray(selected)) {
            // user selected multiple directories
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            // user selected a single directory
            // addFolder(selected)
            const src = "asset://" + selected;
            const response = await fetch(src);
            if (response.status === 200) {
                const type = response.headers.get("Content-Type");
                // artworkSrc = src
                artworkFormat = type;
                isArtworkSet = true;
                // const imageData = await (await response.body.getReader().read()).value;
                // const imageData = await imageUrlToBase64(artworkSrc);
                // console.log("body", imageData)
                artworkSrc = src;
                artworkFileToSet = selected;
                // return {
                //     artworkSrc: src,
                //     artworkFormat: "image/jpeg",
                //     artworkFilenameMatch: artworkFilename
                // }
            }
        }
    }

    onMount(async () => {
        document.addEventListener("paste", async (event: ClipboardEvent) => {
            // event.stopPropagation();
            // event.preventDefault();

            const text = await readText();
            console.log("paste", text);

            // const src = "asset://" + folder + artworkFilename;

            if (!text) {
                try {
                } catch (err) {
                    console.error(err);
                }
            }
        });

        unlisten = await listen("write-success", async (event) => {
            // console.log("Successfully written metadata!");
            toast.success("Successfully written metadata!", {
                position: "top-right"
            });
            // Re-import track
            const updatedSong = await addSong(
                $rightClickedTrack.path,
                $rightClickedTrack.file,
                true
            );
            $rightClickedTrack = updatedSong;
            await getArtwork();
            metadata = mergeDefault($rightClickedTrack.metadata, tagType);
        });
    });

    let previousAlbum = $rightClickedTrack.album;

    function reset() {
        if ($rightClickedTrack.album === previousAlbum && artworkSrc) {
            // Don't update artwork if same as previous album
        } else {
            artworkFormat = null;
            artworkSrc = null;
            foundArtwork = null;
            isArtworkSet = false;
            artworkFileToSet = null;

            getArtwork();
        }
        containsError = null;
        previousAlbum = $rightClickedTrack.album;
        metadata = mergeDefault($rightClickedTrack.metadata, tagType);
        hasChanges = !isEqual(
            metadata,
            mergeDefault($rightClickedTrack.metadata, tagType)
        );
        console.log("metadata", metadata);
    }

    // Shortcuts
    let modifier = $os === "Darwin" ? "cmd" : "ctrl";
    hotkeys(`${modifier}+enter`, function (event, handler) {
        if (hasChanges) {
            writeMetadata();
        }
    });
    hotkeys("esc", () => {
        onClose();
    });

    onDestroy(() => {
        if (unlisten) {
            unlisten();
        }
        hotkeys.unbind(`${modifier}+enter`);
        hotkeys.unbind("esc");
    });

    $: {
        if ($rightClickedTrack) {
            reset();
        }
    }

    let matchingArtists: string[] = [];

    let distinctArtists;

    let firstMatch: string = null;
    $: firstMatchRemainder = firstMatch?.startsWith(artistInput)
        ? firstMatch.substring(artistInput.length, firstMatch.length)
        : "";
    $: {
        console.log(firstMatchRemainder);
    }

    $: artistInput =
        metadata?.find((m) => m.genericId === "artist")?.value ?? "";

    let artistInputField: HTMLInputElement;

    function onArtistAutocompleteSelected() {
        if (firstMatch?.length) {
            artistInput = firstMatch;
            firstMatch = null;
        }
    }

    async function onArtistUpdated(evt) {
        artistInput = evt.target.value;
        const artistField = metadata.find((m) => m.genericId === "artist");
        if (artistField) {
            artistField.value = artistInput;
        }
        metadata = metadata;
        let matched = [];
        const artist = evt.target.value;
        if (artist && artist.trim().length > 0) {
            if (distinctArtists === undefined) {
                distinctArtists = await db.songs.orderBy("artist").uniqueKeys();
            }
            distinctArtists.forEach((a) => {
                if (a.toLowerCase().startsWith(artist.toLowerCase())) {
                    matched.push(a);
                }
            });

            firstMatch = matched.length && matched[0] ? matched[0] : "";
        } else {
            firstMatch = null;
        }
    }

    const VALIDATION_STRINGS = {
        "err:invalid-chars":
            "Invalid characters in metadata tag (hidden/unicode characters?)",
        "err:null-chars": "Hidden null characer",
        "warn:custom-tag":
            "Custom tags can't be parsed. If a custom tag can be a standard tag, use that instead."
    };

    type ValidationErrors = "err:invalid-chars" | "err:null-chars";
    type ValidationWarnings = "warn:custom-tag";

    type Validation = {
        [key: string]: {
            errors: ValidationErrors[];
            warnings: ValidationWarnings[];
        };
    };

    /**
     * Some tags have a \u0000 character dangling at the end.
     * This prevents the metadata from being ready properly, so we can
     * show a prompt to fix this with the user's permission.
     */
    let containsError: ValidationErrors = null;

    function stripNonAsciiChars() {
        metadata = metadata.map((entry) => ({
            ...entry,
            id: entry.id?.replace(/(\u0000)/g, "") ?? entry.id
        }));
        console.log("stripped ascii: ", metadata);
        writeMetadata();
    }

    /**
     * A map of tag id <-> errors that apply to this tag
     */
    $: errors =
        metadata?.reduce((errors: Validation, currentTag) => {
            containsError = null;
            if (!errors[currentTag.id]) {
                errors[currentTag.id] = {
                    errors: [],
                    warnings: []
                };
            }
            if (!errors[currentTag.id].errors) {
                errors[currentTag.id].errors = [];
            }
            if (!errors[currentTag.id].warnings) {
                errors[currentTag.id].warnings = [];
            }

            // Invalid characters - null unicode
            if (currentTag.id.match(/[\u0000]+/g)) {
                errors[currentTag.id].errors.push("err:null-chars");
                containsError = "err:null-chars"; // We want to display a prompt
            } // Invalid characters
            else if (!currentTag.id.match(/^[a-zA-Z0-9_:-]+$/g)) {
                errors[currentTag.id].errors.push("err:invalid-chars");
            }

            // Custom tag - warning
            if (currentTag.id.match(/^(TXXX:)/g)) {
                errors[currentTag.id].warnings.push("warn:custom-tag");
            }

            console.log("hasErrors", hasError("err:null-chars"));
            return errors;
        }, {}) ?? {};

    function hasError(errorType: ValidationErrors): boolean {
        return (
            errors &&
            Object.values(errors).filter((err) =>
                err.errors.includes(errorType)
            ).length > 0
        );
    }
</script>

<container use:focusTrap>
    <header>
        <div class="close">
            <iconify-icon
                icon="mingcute:close-circle-fill"
                on:click={onClose}
            />
            <small>ESC</small>
        </div>
        <div class="button-container">
            <button disabled={!hasChanges} on:click={writeMetadata}
                >Overwrite file</button
            >
            <small>Cmd + ENTER</small>
        </div>

        <div class="title-container">
            <h2>Track info</h2>
            <small class="subtitle">Use UP and DOWN keys to change tracks</small
            >
        </div>
    </header>
    <div class="top">
        <section class="file-section">
            {#if $rightClickedTrack}
                <p>
                    Edit any metadata below, then click "Overwrite file" to
                    save.
                </p>

                <p class="file-path">File path: {$rightClickedTrack.path}</p>

                <div class="file-info">
                    <p>Codec: {$rightClickedTrack.fileInfo.codec}</p>
                    <p>Duration: {durationText}</p>
                    <p>Sample rate: {$rightClickedTrack.fileInfo.sampleRate}</p>
                    {#if $rightClickedTrack.fileInfo.bitsPerSample}<p>
                            {$rightClickedTrack.fileInfo.bitsPerSample} bit audio
                        </p>
                    {/if}
                </div>
            {:else}
                <p>Song has no metadata</p>
            {/if}
        </section>

        <section class="user-section">
            <div
                contenteditable
                class="artwork-container"
                class:focused={artworkFocused}
                on:click={onImageClick}
            >
                <div class="artwork-frame">
                    {#if artworkSrc && artworkFormat}
                        <img
                            alt="Artwork"
                            type={artworkFormat}
                            class="artwork"
                            src={artworkSrc}
                        />
                    {:else}
                        <div class="artwork-placeholder">
                            <iconify-icon icon="mdi:music-clef-treble" />
                            <!-- <small>No art</small> -->
                        </div>
                    {/if}
                </div>
            </div>
            {#if isArtworkSet}
                <small>Ready to save</small>
            {:else if foundArtwork}
                <small>Found {foundArtwork.artworkFilenameMatch}</small>
            {:else if artworkSrc}
                <small>Encoded in file</small>
            {:else}
                <small>&nbsp;</small>
            {/if}
            <span
                use:tippy={{
                    allowHTML: true,
                    content:
                        "Musicat looks for artwork encoded in the file metadata, which you can overwrite by clicking this square (png and jpg supported). <br/><br/>Otherwise, it will look for a file in the album folder called <i>cover.jpg, folder.jpg</i> or <i>artwork.jpg</i> (you can change this list of filenames in Settings).",
                    placement: "left"
                }}
                ><iconify-icon icon="ic:round-info" /><small
                    >About artwork</small
                ></span
            >
        </section>
    </div>
    <div class="bottom">
        <section class="metadata-section">
            <h4>Metadata</h4>
            {#if $rightClickedTrack}
                {#if isUnsupportedFormat}
                    <p>
                        This file type is not yet supported for metadata
                        viewing/editing
                    </p>
                {:else}
                    {#if containsError === "err:null-chars"}
                        <div transition:fly={{y: -20, duration: 100}} class="error-prompt">
                            <iconify-icon icon="ant-design:warning-outlined" />
                            <p>
                                Some tags have a hidden character that prevents
                                them from being read properly.
                            </p>
                            <button on:click={stripNonAsciiChars}>Fix</button>
                        </div>
                    {/if}
                    <form>
                        {#each metadata as tag, idx}
                            {#if tag.genericId === "artist"}
                                <div class="tag">
                                    <p class="label">artist</p>
                                    <div class="line" />
                                    <div
                                        class="artist-input"
                                        use:optionalTippy={{
                                            content:
                                                "Press ENTER to autocomplete",
                                            placement: "bottom",
                                            show: firstMatch !== null,
                                            showOnCreate: true,
                                            trigger: "manual"
                                        }}
                                    >
                                        <Input
                                            value={artistInput}
                                            autoCompleteValue={firstMatch}
                                            onChange={onArtistUpdated}
                                            onEnterPressed={onArtistAutocompleteSelected}
                                            fullWidth
                                        />
                                    </div>
                                </div>
                            {:else}
                                <div
                                    use:optionalTippy={{
                                        content: errors[tag.id]?.errors
                                            .map((e) => VALIDATION_STRINGS[e])
                                            .concat(
                                                errors[tag.id]?.warnings.map(
                                                    (e) => VALIDATION_STRINGS[e]
                                                )
                                            )
                                            .join(","),
                                        placement: "bottom",
                                        theme: "error",
                                        show: errors[tag.id]?.errors.length > 0
                                    }}
                                    class="tag
                                        {errors[tag.id]?.warnings.length > 0
                                        ? 'validation-warning'
                                        : ''}
                                         {errors[tag.id]?.errors.length > 0
                                        ? 'validation-error'
                                        : ''}"
                                >
                                    <p class="label">
                                        {tag.genericId ? tag.genericId : tag.id}
                                    </p>
                                    <div class="line" />
                                    <Input bind:value={tag.value} fullWidth />
                                </div>
                            {/if}
                        {/each}
                    </form>
                {/if}
            {:else}
                <p>Song has no metadata</p>
            {/if}
        </section>
    </div>
</container>

<style lang="scss">
    :global([data-tippy-root]) {
        white-space: pre-line;
    }
    container {
        width: fit-content;
        max-height: 85%;
        max-width: 740px;
        margin: auto;
        display: flex;
        flex-direction: column;
        position: relative;
        align-items: center;
        border-radius: 5px;
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(53, 51, 51);
        background: rgba(60, 60, 63, 0.2);
        backdrop-filter: blur(8px);
        box-shadow: 0px 5px 40px rgba(0, 0, 0, 0.259);
        overflow-y: auto;
        overflow-x: hidden;
        font-family: system-ui, -apple-system, Avenir, Helvetica, Arial,
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
        background: rgba(38, 37, 37, 0.601);
        border-bottom: 1px solid rgb(53, 51, 51);
        backdrop-filter: blur(10px);
        z-index: 20;

        .title-container {
            > .subtitle {
                opacity: 0.5;
                display: block;
                margin: 0;
                font-family: system-ui, -apple-system, Avenir, Helvetica, Arial,
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
        border: 1px solid rgba(255, 255, 255, 0.198);

        &:hover {
            border: 1px solid rgba(255, 255, 255, 0.517);
        }

        &.focused {
            border: 1px solid rgb(255, 255, 255);
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
            img {
                height: 100%;
                width: 100%;
            }

            .artwork-placeholder {
                opacity: 0.2;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 1em;
                iconify-icon {
                    /* margin-top: 0.7em; */
                }
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
        top: 1.7em;
        left: 2em;
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
        grid-template-columns: 1fr auto;
        width: 100%;
    }

    .bottom {
        padding: 0 1em 1em;
        width: 100%;
    }

    .file-section {
        width: 100%;
        padding: 0em 1em;
        border-right: 1px solid rgba(255, 255, 255, 0.099);

        font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
        .file-path {
            opacity: 0.5;
            font-size: 13px;
            max-width: 400px;
            margin: auto;
            font-family: -apple-system, Avenir, Helvetica, Arial, sans-serif;
        }
        .file-info {
            display: flex;
            justify-content: center;
            flex-wrap: wrap;
            gap: 1em;
            text-align: left;
            margin-top: 1em;
            width: 100%;
            font-size: 13px;
            /* border: 1px solid rgb(97, 92, 92); */

            p {
                background-color: rgba(0, 0, 0, 0.118);
                padding: 0.2em 0.5em;
                width: fit-content;
                color: rgb(175, 187, 197);
                user-select: none;
                cursor: default;
                margin: 0;
            }
        }
    }

    .user-section {
        padding: 1em;
        > small {
            color: rgb(68, 161, 94);
        }
        span {
            cursor: default;
            margin: 0.3em auto 0;
            width: fit-content;
            display: flex;
            gap: 5px;
            align-items: center;
            color: rgb(130, 201, 223);
            user-select: none;
            &:hover {
                opacity: 0.7;
            }

            &:active {
                opacity: 0.5;
            }
        }
    }
    .metadata-section {
        margin-top: 2em;
        /* border-top: 1px solid rgb(76, 74, 74); */
        border-radius: 5px;
        position: relative;
        padding: 2em 1em;
        color: white;
        position: relative;
        width: 100%;
        /* border: 1px solid rgb(78, 73, 73); */
        background: rgba(56, 54, 60, 0.842);
        font-family: system-ui, -apple-system, Avenir, Helvetica, Arial,
            sans-serif;
        h4 {
            user-select: none;
            position: absolute;
            left: 0;
            right: 0;
            top: -13px;
            background: rgba(67, 65, 73, 0.89);
            padding: 0 1em;
            z-index: 10;
            text-transform: uppercase;
            opacity: 0.5;
            width: fit-content;
            margin: 0 auto;
            /* font-family: "2Peas"; */
        }

        > p {
            opacity: 0.5;
        }
    }

    form {
        display: grid;
        grid-template-columns: 1fr 1fr;

        @media only screen and (max-width: 700px) {
            grid-template-columns: 1fr;
            .tag {
                display: flex;
                align-items: center;
                justify-content: center;
                &:nth-child(odd) {
                    justify-content: center;
                }
                &:nth-child(even) {
                    justify-content: center;
                }
            }
        }

        @media only screen and (min-width: 701px) {
            .tag {
                display: flex;
                align-items: center;
                &:nth-child(odd) {
                    justify-content: right;
                }
                &:nth-child(even) {
                    justify-content: left;
                    flex-direction: row-reverse;
                }
            }
        }

        column-gap: 3em;
        .tag {
            display: flex;
            align-items: center;
            position: relative;
            > .label {
                width: fit-content;
                margin: 0;
                font-size: 13px;
                font-weight: 500;
                color: rgb(204, 204, 204);
                font-family: monospace;
                user-select: none;
                cursor: default;
                white-space: nowrap;
                text-transform: uppercase;
            }

            .line {
                height: 1px;
                background-color: rgba(255, 255, 255, 0.125);
                width: 40px;
            }

            &.validation-error {
                border: 1px solid rgb(161, 46, 46);
            }

            &.validation-warning {
                border: 1px solid rgb(225, 154, 0);
            }
        }

        .artist-input {
            /* overflow: hidden; */
        }
    }
    .error-prompt {
        padding: 0.2em;
        border-radius: 8px;
        margin: 1em;
        background-color: rgb(111, 87, 87);
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: center;

        iconify-icon {
            padding: 1em;
        }
        p {
            margin: 0;
        }
    }
</style>
