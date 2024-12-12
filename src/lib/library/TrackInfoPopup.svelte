<script lang="ts">
    import { pictureDir } from "@tauri-apps/api/path";
    import { open } from "@tauri-apps/plugin-dialog";
    import { open as fileOpen } from "@tauri-apps/plugin-shell";

    import hotkeys from "hotkeys-js";
    import { cloneDeep, isEqual, uniqBy } from "lodash-es";
    import type {
        Album,
        MetadataEntry,
        Song,
        TagType,
        ToImport
    } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import toast from "svelte-french-toast";
    import tippy from "svelte-tippy";
    import { fade, fly } from "svelte/transition";
    import { getMapForTagType } from "../../data/LabelMap";
    import { readMappedMetadataFromSong } from "../../data/LibraryImporter";
    import { db } from "../../data/db";
    import {
        os,
        popupOpen,
        rightClickedTrack,
        rightClickedTracks
    } from "../../data/store";
    import { focusTrap } from "../../utils/FocusTrap";
    import "../tippy.css";
    import Input from "../ui/Input.svelte";
    import { optionalTippy } from "../ui/TippyAction";

    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import {
        ENCODINGS,
        decodeLegacy,
        encodeUtf8
    } from "../../utils/EncodingUtils";
    import {
        fetchAlbumArt,
        findCountryByArtist
    } from "../data/LibraryEnrichers";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";

    import { Image } from "@tauri-apps/api/image";
    import { Buffer } from "buffer";
    import LL from "../../i18n/i18n-svelte";
    // optional

    const ALBUM_FIELDS = ["album", "artist", "date", "genre"];

    function onClose() {
        $popupOpen = null;
    }

    function addDefaults(metadata: MetadataEntry[], format: TagType) {
        // Map eg. 'TITLE' (FLAC/Vorbis) -> title (Musicat generic identifier)
        const map = getMapForTagType(format, false);
        // Add empty default fields
        const defaults: MetadataEntry[] = [];
        const others: MetadataEntry[] = [];
        console.log("map", map);
        if (!map) return { defaults, others };
        for (const field of Object.entries(map)) {
            // Check if this default field already exists in the file
            const existingField = metadata?.find(
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
        if (
            ($rightClickedTrack || $rightClickedTracks[0]).fileInfo.codec ===
            "WAV"
        ) {
            isUnsupportedFormat = true;
            return []; // UNSUPPORTED FORMAT, for now...
        }

        console.log("adding defaults", metadata, format);
        if (!format) return [];

        isUnsupportedFormat = false;
        const cloned = cloneDeep(metadata);
        const { defaults, others } = addDefaults(cloned, format);
        return uniqBy(
            [...defaults, ...others.sort((a, b) => a.id.localeCompare(b.id))],
            "id"
        );
    }

    // console.log("track", ($rightClickedTrack || $rightClickedTracks[0]));
    let metadata: { mappedMetadata: MetadataEntry[]; tagType: TagType } = {
        mappedMetadata: [],
        tagType: null
    };
    let metadataFromFile: MetadataEntry[] = [];
    // let metadata: MetadataEntry[] = cloneDeep(($rightClickedTrack || $rightClickedTracks[0]).metadata);

    function getDurationText(durationInSeconds: number) {
        return `${(~~(
            ($rightClickedTrack || $rightClickedTracks[0]).fileInfo.duration /
            60
        ))
            .toString()
            .padStart(2, "0")}:${(~~(
            ($rightClickedTrack || $rightClickedTracks[0]).fileInfo.duration %
            60
        ))
            .toString()
            .padStart(2, "0")}`;
    }

    // The artwork for this track(s)
    let artworkFormat;
    let artworkBuffer: Buffer;
    let artworkSrc;

    // When scrolling through tracks, re-use artworks
    let previousArtworkFormat;
    let previousArtworkSrc;

    // The preliminary artwork to be set (from an image on disk)
    let artworkToSetSrc = null;
    let artworkToSetFormat = null;
    let artworkToSetData: Uint8Array = null;
    let isArtworkSet = false;
    let artworkFileToSet = null;

    let foundArtwork;

    $: hasChanges =
        isArtworkSet || !isEqual(metadata?.mappedMetadata, metadataFromFile);

    let artworkFocused = false;

    async function getArtwork() {
        if ($rightClickedTrack === null && !$rightClickedTracks.length) {
            return;
        }
        console.log("getting artwork", $rightClickedTrack, $rightClickedTracks);

        let path = ($rightClickedTrack || $rightClickedTracks[0]).path;
        if (path) {
            const songWithArtwork = await invoke<Song>("get_song_metadata", {
                event: { path, isImport: false, includeFolderArtwork: true }
            });

            if (!songWithArtwork) {
                toast.error(
                    `Error reading file ${path}. Check permissions, or if the file is used by another program.`,
                    { className: "app-toast" }
                );
            }

            if (songWithArtwork?.artwork) {
                console.log("artwork");
                artworkFormat = songWithArtwork.artwork.format;
                if (songWithArtwork.artwork.data.length) {
                    artworkBuffer = Buffer.from(songWithArtwork.artwork.data);
                    artworkSrc = `data:${artworkFormat};base64, ${artworkBuffer.toString(
                        "base64"
                    )}`;
                } else if (songWithArtwork.artwork.src) {
                    artworkSrc = convertFileSrc(songWithArtwork.artwork.src);
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

    /**
     * Send an event to the backend to write the new metadata, overwriting any existing tags.
     */
    async function writeMetadata() {
        let toImport: ToImport;
        if ($rightClickedTrack) {
            const toWrite = metadata?.mappedMetadata
                .filter((m) => m.value !== null)
                .map((t) => ({ id: t.id, value: t.value }));
            console.log("Writing: ", toWrite);

            toImport = await invoke<ToImport>("write_metadatas", {
                event: {
                    tracks: [
                        {
                            "song_id": $rightClickedTrack.id,
                            metadata: toWrite,
                            "tag_type": metadata.tagType,
                            "file_path": $rightClickedTrack.path,
                            "artwork_file": artworkFileToSet
                                ? artworkFileToSet
                                : "",
                            "artwork_data": artworkToSetData ?? ""
                        }
                    ]
                }
            });
        } else if ($rightClickedTracks?.length) {
            console.log("Writing album");
            toImport = await invoke<ToImport>("write_metadatas", {
                event: {
                    tracks: await Promise.all(
                        $rightClickedTracks.map(async (track) => {
                            const fileMetadata =
                                await readMappedMetadataFromSong(track);
                            return {
                                "song_id": track.id,
                                metadata: [
                                    ...fileMetadata?.mappedMetadata,
                                    ...metadata?.mappedMetadata
                                        .filter(
                                            (m) =>
                                                m.value !== null &&
                                                ALBUM_FIELDS.includes(
                                                    m.genericId
                                                )
                                        )
                                        .map((t) => ({
                                            id: t.id,
                                            value: t.value
                                        }))
                                ],
                                "tag_type": fileMetadata.tagType,
                                "file_path": track.path,
                                "artwork_file": artworkFileToSet
                                    ? artworkFileToSet
                                    : "",
                                "artwork_data": artworkToSetData ?? ""
                            };
                        })
                    )
                }
            });
        }
        console.log($rightClickedTrack || $rightClickedTracks[0]);
        console.log(toImport);
        if (toImport) {
            if (toImport.error) {
                // Show error
                toast.error(toImport.error);
                // Roll back to current artwork
                artworkToSetFormat = null;
                artworkToSetSrc = null;
                artworkToSetData = null;
            } else {
                await reImportTracks(toImport.songs);
            }
        }

        artworkFileToSet = null;
        isArtworkSet = false;

        if (toImport.albums.length) {
            for (const album of toImport.albums) {
                await reImportAlbum(album);
            }
        }

        toast.success("Successfully written metadata!", {
            position: "top-right"
        });

        await reset();
    }

    async function reImportAlbum(album: Album) {
        const existingAlbum = await db.albums.get(album.id);
        if (existingAlbum) {
            existingAlbum.tracksIds = [
                ...existingAlbum.tracksIds,
                ...album.tracksIds
            ];
            await db.albums.put(existingAlbum);
        }
    }

    async function reImportTracks(songs: Song[]) {
        await db.songs.bulkPut(songs);
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
        console.log("clicked", artworkFocused);
        if (!artworkFocused) {
            artworkFocused = true;
            return;
        }
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
            const src = "asset://localhost/" + selected;
            const response = await fetch(src);
            if (response.status === 200) {
                const type = response.headers.get("Content-Type");
                artworkFormat = type;
                isArtworkSet = true;
                artworkFileToSet = selected;
                artworkToSetData = null;
                artworkToSetSrc = src;
                artworkToSetFormat = type;
            }
        }
    }

    /**
     * Grabs the existing tags from the file, and adds in the defaults for that file format
     */
    async function getMetadataFromFile(song: Song): Promise<MetadataEntry[]> {
        // Get metadata from file
        const { mappedMetadata, tagType } =
            await readMappedMetadataFromSong(song);

        const result = mergeDefault(mappedMetadata, tagType);
        metadataFromFile = cloneDeep(result);
        return result;
    }

    let previousAlbum = ($rightClickedTrack || $rightClickedTracks[0]).album;

    async function updateArtwork() {
        artworkFormat = null;
        artworkSrc = null;
        artworkToSetFormat = null;
        artworkToSetSrc = null;
        artworkToSetData = null;
        foundArtwork = null;
        isArtworkSet = false;
        artworkFileToSet = null;

        await getArtwork();
    }

    async function reset() {
        updateArtwork();
        containsError = null;
        previousAlbum = ($rightClickedTrack || $rightClickedTracks[0]).album;
        const { mappedMetadata, tagType } = await readMappedMetadataFromSong(
            $rightClickedTrack || $rightClickedTracks[0]
        );
        metadata = {
            mappedMetadata: mergeDefault(mappedMetadata, tagType),
            tagType
        };
        metadataFromFile = cloneDeep(metadata.mappedMetadata);
        originCountry =
            ($rightClickedTrack || $rightClickedTracks[0]).originCountry || "";
        originCountryEdited = originCountry;
        hasChanges = !isEqual(metadata?.mappedMetadata, metadataFromFile);
        console.log("metadata", metadata);
    }

    // Shortcuts
    let modifier = $os === "macos" ? "cmd" : "ctrl";
    hotkeys(`${modifier}+enter`, "track-info", function (event, handler) {
        if (hasChanges) {
            writeMetadata();
        }
    });
    hotkeys("esc", "track-info", () => {
        onClose();
    });

    onDestroy(() => {
        if (unlisten) {
            unlisten();
        }
        hotkeys.unbind(`${modifier}+enter`, "track-info");
        hotkeys.unbind("esc", "track-info");
        hotkeys.deleteScope("track-info");
        $popupOpen = null;
    });

    $: {
        if ($rightClickedTrack || $rightClickedTracks[0]) {
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
        metadata?.mappedMetadata?.find((m) => m.genericId === "artist")
            ?.value ?? "";

    let artistInputField: HTMLInputElement;

    function onArtistAutocompleteSelected() {
        if (firstMatch?.length) {
            console.log("firstMatch", firstMatch);
            console.log("artistInput", artistInput);
            const artistField = metadata?.mappedMetadata?.find(
                (m) => m.genericId === "artist"
            );
            if (artistField) {
                artistField.value = firstMatch;
                artistInput = firstMatch;
            }
            metadata = metadata;
            firstMatch = null;
        }
    }

    async function onArtistUpdated(value) {
        console.log("artist updated");
        artistInput = value;
        const artistField = metadata?.mappedMetadata?.find(
            (m) => m.genericId === "artist"
        );
        if (artistField) {
            artistField.value = artistInput;
        }
        metadata = metadata;
        let matched = [];
        const artist = value;
        if (artist && artist.trim().length > 0) {
            if (distinctArtists === undefined) {
                distinctArtists = await db.songs.orderBy("artist").uniqueKeys();
            }
            distinctArtists.forEach((a) => {
                if (a.toLowerCase().startsWith(artist.toLowerCase())) {
                    matched.push(a);
                }
            });

            firstMatch = matched.length && matched[0] ? matched[0] : null;
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
    type ValidationWarnings = "warn:custom-tag" | "warn:cyrillic-encoding";

    type Validation = {
        [key: string]: {
            errors: ValidationErrors[];
            warnings: ValidationWarnings[];
        };
    };

    /**
     * Some tags have a \u0000 character dangling at the end.
     * This prevents the metadata from being read properly, so we can
     * show a prompt to fix this with the user's permission.
     */
    let containsError: ValidationErrors = null;

    function stripNonAsciiChars() {
        metadata.mappedMetadata = metadata?.mappedMetadata.map((entry) => ({
            ...entry,
            id: entry.id?.replace(/(\u0000)/g, "") ?? entry.id
        }));
        console.log("stripped ascii: ", metadata);
        writeMetadata();
    }

    // Encodings
    let selectedEncoding = "placeholder";

    async function fixEncoding() {
        for (let item of metadata?.mappedMetadata) {
            if (!item?.value) continue;
            const decoded = decodeLegacy(item.value, selectedEncoding);
            const encoded = new TextDecoder().decode(encodeUtf8(decoded));
            console.log("transformed: ", encoded);
            item.value = encoded;
            metadata = metadata;
        }
    }

    /**
     * A map of tag id <-> errors that apply to this tag
     */
    $: errors =
        metadata?.mappedMetadata?.reduce((errors: Validation, currentTag) => {
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

            // TODO Detect encodings
            // Tried jschardet but it didn't give good results.
            // Maybe scrap this entirely?

            // const windows_encodings = ["windows-1251"];
            // const encoding = jschardet.detect(currentTag.value || "", {
            //     detectEncodings: ENCODINGS,
            // });
            // console.log('encoding detect', encoding);
            // { encoding: "UTF-8", confidence: 0.9690625 }

            // if (
            //     windows_encodings.includes(encoding.encoding) &&
            //     encoding.confidence > 0.5
            // ) {
            //     errors[currentTag.id].warnings.push("warn:cyrillic-encoding");
            // }

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

    let originCountry =
        ($rightClickedTrack || $rightClickedTracks[0]).originCountry || "";
    let originCountryEdited = originCountry;
    let isFetchingOriginCountry = false;
    function onOriginCountryUpdated(event) {
        const country = event.target.value;
        originCountryEdited = country;
    }

    async function saveTrack() {
        ($rightClickedTrack || $rightClickedTracks[0]).originCountry =
            originCountryEdited;

        // Find all songs with this artist
        const artistSongs = await db.songs
            .where("artist")
            .equals(($rightClickedTrack || $rightClickedTracks[0]).artist)
            .toArray();
        artistSongs.forEach((s) => {
            db.songs.update(s.id, { originCountry: originCountryEdited });
        });
    }

    async function fetchFromWikipedia() {
        originCountryEdited = null;
        isFetchingOriginCountry = true;
        const country = await findCountryByArtist(
            ($rightClickedTrack || $rightClickedTracks[0]).artist
        );
        console.log("country", country);
        if (country) {
            originCountryEdited = country;
        }
        isFetchingOriginCountry = false;
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
            isArtworkSet = true;
        }
        // const src = "asset://localhost/" + folder + artworkFilename;
    }

    onMount(async () => {
        hotkeys.setScope("track-info");
        document.addEventListener("paste", onPaste);

        onTableResize();
    });

    onDestroy(() => {
        document.removeEventListener("paste", onPaste);
    });

    let isFetchingArtwork = false;
    let artworkResult: { success?: string; error?: string };
    async function fetchArtwork() {
        isFetchingArtwork = true;
        artworkResult = await fetchAlbumArt(
            null,
            $rightClickedTrack || $rightClickedTracks[0]
        );
        if (artworkResult.success) {
            toast.success("Found album art and written to album!", {
                position: "top-right"
            });
            updateArtwork();
        } else if (artworkResult.error) {
            toast.error("Couldn't find album art", {
                position: "top-right"
            });
        }
        isFetchingArtwork = false;
    }

    function onTagLabelClick(tag: MetadataEntry) {
        // Double-clicking on title populates the title from the filename
        if (tag.genericId === "title") {
            // Strip file extension from filename if any
            const filename = $rightClickedTrack.file;
            const extension = filename.split(".").pop();
            const filenameWithoutExtension = filename.replace(
                "." + extension,
                ""
            );
            tag.value = filenameWithoutExtension;

            metadata = metadata;
        }
    }
</script>

<container use:focusTrap>
    <header>
        <div class="close">
            <Icon icon="mingcute:close-circle-fill" onClick={onClose} />
            <small>ESC</small>
        </div>
        <div class="button-container">
            <ButtonWithIcon
                disabled={!hasChanges}
                onClick={writeMetadata}
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
        <section class="info-section" bind:this={tableOuterContainer}>
            <div class="file-outer" bind:this={tableOuterContainer}>
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
                                        <p
                                            on:click={() =>
                                                fileOpen(
                                                    track.path.replace(
                                                        track.file,
                                                        ""
                                                    )
                                                )}
                                        >
                                            <span
                                                ><Icon
                                                    icon="bi:file-earmark-play"
                                                    size={12}
                                                /></span
                                            >
                                            {track.file}
                                        </p></td
                                    >
                                    <td>
                                        <p>
                                            {track.fileInfo.codec}
                                        </p></td
                                    >
                                    <td>
                                        <p>
                                            {track.fileInfo.tagType}
                                        </p></td
                                    >
                                    <td>
                                        <p>
                                            {getDurationText(
                                                track.fileInfo.duration
                                            )}
                                        </p>
                                    </td>
                                    <td>
                                        <p>
                                            {track.fileInfo.sampleRate} hz
                                        </p></td
                                    >
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
            </div>

            <div class="enrichment">
                <h5 class="section-title">
                    <Icon
                        icon="iconoir:atom"
                        size={34}
                    />{$LL.trackInfo.enrichmentCenter()}
                </h5>
                <div class="label">
                    <h4>{$LL.trackInfo.countryOfOrigin()}</h4>
                    <div
                        use:tippy={{
                            content: $LL.trackInfo.countryOfOriginTooltip(),
                            placement: "right"
                        }}
                    >
                        <Icon icon="mdi:information" />
                    </div>
                </div>
                <div class="country">
                    <Input
                        fullWidth
                        value={originCountryEdited}
                        placeholder={isFetchingOriginCountry
                            ? $LL.trackInfo.fetchingOriginCountry()
                            : ""}
                        onChange={onOriginCountryUpdated}
                    />
                    <ButtonWithIcon
                        onClick={saveTrack}
                        text={$LL.trackInfo.save()}
                        icon="material-symbols:save-outline"
                        theme="translucent"
                        disabled={originCountry === originCountryEdited}
                    />
                    <ButtonWithIcon
                        onClick={fetchFromWikipedia}
                        isLoading={isFetchingOriginCountry}
                        text={$LL.trackInfo.fetchFromWikipedia()}
                        icon="tabler:world-download"
                        theme="transparent"
                    />
                </div>
            </div>
        </section>

        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <section class="user-section">
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                contenteditable
                class="artwork-container"
                class:focused={artworkFocused}
                on:mousedown|preventDefault={(e) => {
                    onImageClick(e);
                    //@ts-ignore
                    e.currentTarget.focus();
                }}
                on:focus={() => (artworkFocused = true)}
                on:blur={() => (artworkFocused = false)}
                use:tippy={{
                    content: $LL.trackInfo.artworkTooltip(),
                    placement: "bottom",
                    trigger: "focusin"
                }}
            >
                <div class="artwork-frame">
                    {#if (artworkToSetSrc && artworkToSetFormat) || (previousArtworkSrc && previousArtworkFormat) || (artworkSrc && artworkFormat)}
                        <img
                            alt="Artwork"
                            class="artwork"
                            src={artworkToSetSrc ||
                                previousArtworkSrc ||
                                artworkSrc}
                        />
                    {:else}
                        <div class="artwork-placeholder">
                            <Icon icon="mdi:music-clef-treble" />
                            <!-- <small>No art</small> -->
                        </div>
                    {/if}
                </div>
            </div>
            {#if isArtworkSet}
                <small>{$LL.trackInfo.artworkReadyToSave()}</small>
            {:else if foundArtwork}
                <small
                    >{$LL.trackInfo.artworkFound()}
                    {foundArtwork.artworkFilenameMatch}</small
                >
            {:else if artworkSrc}
                <small>{$LL.trackInfo.encodedInFile()}</small>
            {:else}
                <small style="color: grey">{$LL.trackInfo.noArtwork()}</small>
            {/if}
            <span
                use:tippy={{
                    allowHTML: true,
                    content: $LL.trackInfo.artworkTooltipBody(),
                    placement: "left"
                }}
                ><Icon icon="ic:round-info" /><small
                    >{$LL.trackInfo.aboutArtwork()}</small
                ></span
            >
            <div class="find-art-btn">
                <ButtonWithIcon
                    icon="ic:twotone-downloading"
                    text="Fetch art"
                    theme="transparent"
                    onClick={fetchArtwork}
                />
            </div>
        </section>
    </div>
    <div class="bottom">
        <section class="metadata-section">
            <h5 class="section-title">
                <Icon icon="fe:music" size={30} />{$LL.trackInfo.metadata()}
            </h5>
            {#if $rightClickedTrack || $rightClickedTracks[0]}
                {#if isUnsupportedFormat}
                    <p>
                        {$LL.trackInfo.unsupportedFormat()}
                    </p>
                {:else}
                    {#if containsError === "err:null-chars"}
                        <div
                            transition:fly={{ y: -20, duration: 100 }}
                            class="error-prompt"
                        >
                            <Icon icon="ant-design:warning-outlined" />
                            <p>
                                {$LL.trackInfo.errors.nullChars()}
                            </p>
                            <button on:click={stripNonAsciiChars}
                                >{$LL.trackInfo.fix()}</button
                            >
                        </div>
                    {/if}
                    <form>
                        {#each metadata?.mappedMetadata?.filter((m) => $rightClickedTrack || ($rightClickedTracks.length && ALBUM_FIELDS.includes(m.genericId))) as tag, idx}
                            {#if tag.genericId === "artist"}
                                <div class="tag">
                                    <p class="label">
                                        {$LL.trackInfo.artist()}
                                    </p>
                                    <div class="line" />
                                    <div
                                        class="artist-input"
                                        use:optionalTippy={{
                                            content:
                                                $LL.input.enterHintTooltip(),
                                            placement: "bottom",
                                            show:
                                                firstMatch !== null &&
                                                firstMatch.toLowerCase() !==
                                                    artistInput?.toLowerCase() &&
                                                artistInput?.length > 0,
                                            showOnCreate: true,
                                            trigger: "manual"
                                        }}
                                    >
                                        <Input
                                            small
                                            value={artistInput}
                                            autoCompleteValue={firstMatch}
                                            onChange={onArtistUpdated}
                                            onEnterPressed={onArtistAutocompleteSelected}
                                            tabBehavesAsEnter
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
                                        show:
                                            errors[tag.id]?.errors.length > 0 ||
                                            errors[tag.id]?.warnings.length > 0
                                    }}
                                    class="tag
                                        {errors[tag.id]?.warnings.length > 0
                                        ? 'validation-warning'
                                        : ''}
                                         {errors[tag.id]?.errors.length > 0
                                        ? 'validation-error'
                                        : ''}"
                                >
                                    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                                    <p
                                        class="label"
                                        class:unmapped={tag.genericId ===
                                            undefined}
                                        data-tag-id={tag.genericId || tag.id}
                                        on:click={() => onTagLabelClick(tag)}
                                        use:optionalTippy={{
                                            content:
                                                $LL.trackInfo.setTitleFromFileNameHint(),
                                            show: tag.genericId === "title",
                                            placement: "bottom"
                                        }}
                                    >
                                        {tag.genericId ? tag.genericId : tag.id}
                                    </p>
                                    <div class="line" />
                                    <Input
                                        bind:value={tag.value}
                                        fullWidth
                                        small
                                    />
                                </div>
                            {/if}
                        {/each}
                    </form>
                    <div class="tools">
                        <h5 class="section-title">
                            <Icon icon="ri:tools-fill" />{$LL.trackInfo.tools()}
                        </h5>
                        <div class="tool">
                            <div class="description">
                                <p>
                                    {$LL.trackInfo.fixLegacyEncodings.title()}
                                </p>
                                <small
                                    >{$LL.trackInfo.fixLegacyEncodings.body()}</small
                                >
                            </div>
                            <select bind:value={selectedEncoding}>
                                <option value="placeholder"
                                    >{$LL.trackInfo.fixLegacyEncodings.hint()}</option
                                >
                                {#each ENCODINGS as encoding}
                                    <option
                                        value={encoding}
                                        class="encoding"
                                        on:click={() => {
                                            selectedEncoding = encoding;
                                        }}
                                    >
                                        <p>{encoding}</p>
                                    </option>
                                {/each}
                            </select>
                            <ButtonWithIcon
                                text={$LL.trackInfo.fix()}
                                theme="transparent"
                                onClick={fixEncoding}
                                disabled={selectedEncoding === "placeholder"}
                            />
                        </div>
                    </div>
                {/if}
            {:else}
                <p>{$LL.trackInfo.noMetadata()}</p>
            {/if}
        </section>
    </div>
</container>

<style lang="scss">
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
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(53, 51, 51);
        background-color: var(--overlay-bg);
        box-shadow: 0px 5px 40px rgba(0, 0, 0, 0.259);
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
        background-color: color-mix(in srgb, var(--background) 1%, transparent);
        border-bottom: 1px solid
            color-mix(in srgb, var(--background) 60%, var(--inverse));
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
            border: 1px solid rgba(255, 255, 255, 0.517);
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

    .info-section {
    }
    .file-outer {
        border: 1px solid
            color-mix(in srgb, var(--background) 70%, var(--inverse));
        border-radius: 5px;
        padding: 4px;
        min-width: 575px;
        position: relative;
        background-color: color-mix(
            in srgb,
            var(--overlay-bg) 80%,
            var(--inverse)
        );

        .section-title {
            top: -10px;
            margin: 0;
            color: white;
            border: 1px solid rgba(128, 128, 128, 0.16);
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
                        background-color: rgba(0, 0, 0, 0.118);
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

                    p {
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

    .user-section {
        padding: 0 0 0 1em;
        > small {
            color: rgb(68, 161, 94);
            white-space: nowrap;
            text-overflow: ellipsis;
            overflow: hidden;
            display: block;
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
        .find-art-btn {
            margin-top: 1em;
        }
    }

    .section-title {
        position: absolute;
        border-radius: 4px;
        max-height: 2em;
        display: flex;
        flex-direction: row;
        gap: 5px;
        align-items: center;
        background-color: color-mix(
            in srgb,
            var(--background) 50%,
            var(--inverse)
        );
        z-index: 11;
        border: 1px solid rgba(28, 163, 201, 0.29);
        top: -15px;
        padding: 0 10px;
        letter-spacing: 1px;
        font-weight: 400;
        left: 3em;
        right: 0;
        width: fit-content;
        margin: 0.5em 0;
        text-align: start;
        color: #7dffee;
        text-transform: uppercase;
    }

    .enrichment {
        margin-top: 1.5em;
        border: 1px solid
            color-mix(in srgb, var(--background) 70%, var(--inverse));
        border-radius: 5px;
        padding: 2em 1em 1em 1em;
        grid-column: 1 / 3;
        background-color: color-mix(
            in srgb,
            var(--overlay-bg) 80%,
            var(--inverse)
        );
        position: relative;

        .section-title {
            border: 1px solid rgba(28, 163, 201, 0.19);

            color: #7dffee;
        }

        .label {
            display: flex;
            flex-direction: row;
            gap: 5px;
            align-items: center;
            margin: 0 0 5px 0;

            h4 {
                margin: 0;
                color: var(--text);
                text-align: left;
                text-transform: uppercase;
                letter-spacing: 0.5px;
                font-size: 0.9em;
            }

            p {
                margin: 0;
                color: rgba(255, 255, 255, 0.768);
            }
        }
        .country {
            display: flex;
            justify-content: space-between;
            align-items: center;
            width: fit-content;
            gap: 5px;
            p {
                margin-right: 1em;
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
        border: 1px solid
            color-mix(in srgb, var(--background) 70%, var(--inverse));

        background-color: color-mix(
            in srgb,
            var(--overlay-bg) 80%,
            var(--inverse)
        );
        font-family:
            system-ui,
            -apple-system,
            Avenir,
            Helvetica,
            Arial,
            sans-serif;

        .section-title {
            color: #ffe6ac;
            border: 1px solid rgba(255, 203, 158, 0.145);
        }
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
        column-gap: 3em;

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

        .tag {
            display: flex;
            align-items: center;
            position: relative;
            margin: 1px 0;
            > .label {
                width: fit-content;
                margin: 0;
                font-size: 13px;
                font-weight: 500;
                color: var(--text);
                font-family: monospace;
                user-select: none;
                cursor: default;
                white-space: nowrap;
                text-transform: uppercase;
                border: 1px solid transparent;

                &.unmapped {
                    opacity: 0.5;
                }
                &[data-tag-id="title"] {
                    &:hover {
                        border: 1px solid
                            color-mix(
                                in srgb,
                                var(--background) 60%,
                                var(--inverse)
                            );
                        cursor: pointer;
                    }
                }
            }

            .line {
                height: 1px;
                background-color: color-mix(
                    in srgb,
                    var(--background) 60%,
                    var(--inverse)
                );
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

    .tools {
        border: 1px solid rgba(255, 255, 255, 0.099);
        padding: 2em;
        margin: 2em;
        position: relative;

        .section-title {
            color: rgb(170, 170, 170);
            border: transparent;
        }

        .tool {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            color: var(--text);
            * {
                color: var(--text);
            }

            .description {
                text-align: left;
                p {
                    margin: 0;
                }
                small {
                    opacity: 0.7;
                    line-height: 0.5em;
                }
            }

            select {
                font-size: 20px;
                padding: 0.2em 1em 0.3em 1em;
            }
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
        p {
            margin: 0;
        }
    }
</style>
