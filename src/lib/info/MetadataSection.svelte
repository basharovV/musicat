<svelte:options accessors={true} />

<script lang="ts">
    import { fly } from "svelte/transition";
    import Icon from "../ui/Icon.svelte";
    import LL from "../../i18n/i18n-svelte";
    import {
        current,
        isPlaying,
        lastWrittenSongs,
        playerTime,
        rightClickedTrack,
        rightClickedTracks,
    } from "../../data/store";
    import type {
        Album,
        MetadataEntry,
        Song,
        TagType,
        ToImport,
    } from "src/App";
    import { invoke } from "@tauri-apps/api/core";
    import { readMappedMetadataFromSong } from "../../data/LibraryUtils";
    import toast from "svelte-french-toast";
    import { db } from "../../data/db";
    import { get } from "svelte/store";
    import audioPlayer from "../player/AudioPlayer";
    import { optionalTippy } from "../ui/TippyAction";
    import Input from "../ui/Input.svelte";
    import {
        ENCODINGS,
        decodeLegacy,
        encodeUtf8,
    } from "../../utils/EncodingUtils";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import { cloneDeep, isEqual, uniq, uniqBy } from "lodash-es";
    import { getMapForTagType } from "../../data/LabelMap";
    import { onMount } from "svelte";

    type ValidationErrors = "err:invalid-chars" | "err:null-chars";
    type ValidationWarnings = "warn:custom-tag" | "warn:cyrillic-encoding";

    type Validation = {
        [key: string]: {
            errors: ValidationErrors[];
            warnings: ValidationWarnings[];
        };
    };

    const ALBUM_FIELDS = [
        "album",
        "albumArtist",
        "artist",
        "year",
        "genre",
        "compilation",
        "trackTotal",
        "discNumber",
        "discTotal",
    ];

    const VALIDATION_STRINGS = {
        "err:invalid-chars":
            "Invalid characters in metadata tag (hidden/unicode characters?)",
        "err:null-chars": "Hidden null characer",
        "warn:custom-tag":
            "Custom tags can't be parsed. If a custom tag can be a standard tag, use that instead.",
    };

    export let completeEvent: (event) => {};
    export let hasChanges = false;
    export let data: { mappedMetadata: MetadataEntry[]; tagType: TagType } = {
        mappedMetadata: [],
        tagType: null,
    };
    export let hasArtworkToSet: () => boolean;
    export let reset: () => void;
    export let rollbackArtwork: () => void;

    /**
     * Some tags have a \u0000 character dangling at the end.
     * This prevents the metadata from being read properly, so we can
     * show a prompt to fix this with the user's permission.
     */
    let containsError: ValidationErrors = null;
    let distinctArtists;
    let firstMatch: string = null;
    let isUnsupportedFormat: boolean;
    let metadataFromFile: MetadataEntry[] = [];
    // Encodings
    let selectedEncoding = "placeholder";

    $: artistInput =
        data?.mappedMetadata?.find((m) => m.genericId === "artist")?.value ??
        "";

    /**
     * A map of tag id <-> errors that apply to this tag
     */
    $: errors =
        data?.mappedMetadata?.reduce((errors: Validation, currentTag) => {
            containsError = null;
            if (!errors[currentTag.id]) {
                errors[currentTag.id] = {
                    errors: [],
                    warnings: [],
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
            else if (!currentTag.id.match(/^[a-zA-Z0-9©_:\-\.]+$/)) {
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

    $: firstMatchRemainder = firstMatch?.startsWith(artistInput)
        ? firstMatch.substring(artistInput.length, firstMatch.length)
        : "";
    $: {
        console.log(firstMatchRemainder);
    }

    $: hasChanges = !isEqual(data?.mappedMetadata, metadataFromFile);

    function addDefaults(entry: MetadataEntry[], format: TagType) {
        // Map eg. 'TITLE' (FLAC/Vorbis) -> title (Musicat generic identifier)
        const map = getMapForTagType(format, false);
        // Add empty default fields
        const defaults: MetadataEntry[] = [];
        const others: MetadataEntry[] = [];
        console.log("map", map);
        if (!map) return { defaults, others };
        for (const [id, genericId] of Object.entries(map)) {
            if (Array.isArray(genericId)) {
                for (const genId of genericId) {
                    // Check if this default field already exists in the file
                    const existingField = entry?.find(
                        (m) => m.genericId === genId,
                    );

                    defaults.push({
                        id,
                        genericId: genId,
                        value: existingField ? existingField.value : null,
                    });
                }
            } else {
                // Check if this default field already exists in the file
                const existingField = entry?.find(
                    (m) => m.genericId === genericId,
                );

                defaults.push({
                    id,
                    genericId,
                    value: existingField ? existingField.value : null,
                });
            }
        }

        for (const field of entry) {
            const inDefaults = defaults.find(
                (t) => t.genericId === field.genericId,
            );
            if (!inDefaults) {
                others.push({
                    id: field.id,
                    genericId: field.genericId,
                    value: field.value,
                });
            }
        }

        return { defaults, others };
    }

    async function fixEncoding() {
        for (let item of data?.mappedMetadata) {
            if (!item?.value) continue;
            const decoded = decodeLegacy(item.value, selectedEncoding);
            const encoded = new TextDecoder().decode(encodeUtf8(decoded));
            console.log("transformed: ", encoded);
            item.value = encoded;
            data = data;
        }
    }

    function hasError(errorType: ValidationErrors): boolean {
        return (
            errors &&
            Object.values(errors).filter((err) =>
                err.errors.includes(errorType),
            ).length > 0
        );
    }

    function mergeDefault(
        metadata: MetadataEntry[],
        format: TagType,
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
        const result = uniqBy(
            [...defaults, ...others.sort((a, b) => a.id.localeCompare(b.id))],
            "genericId",
        );
        return result;
    }

    function onArtistAutocompleteSelected() {
        if (firstMatch?.length) {
            console.log("firstMatch", firstMatch);
            console.log("artistInput", artistInput);
            const artistField = data?.mappedMetadata?.find(
                (m) => m.genericId === "artist",
            );
            if (artistField) {
                artistField.value = firstMatch;
                artistInput = firstMatch;
            }
            data = data;
            firstMatch = null;
        }
    }

    async function onArtistUpdated(value) {
        console.log("artist updated");
        artistInput = value;
        const artistField = data?.mappedMetadata?.find(
            (m) => m.genericId === "artist",
        );
        if (artistField) {
            artistField.value = artistInput;
        }
        data = data;
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

    function onTagLabelClick(tag: MetadataEntry) {
        // Double-clicking on title populates the title from the filename
        if (tag.genericId === "title") {
            // Strip file extension from filename if any
            const filename = $rightClickedTrack.file;
            const extension = filename.split(".").pop();
            const filenameWithoutExtension = filename.replace(
                "." + extension,
                "",
            );
            tag.value = filenameWithoutExtension;
            data = data;
        }
    }

    async function reImportAlbum(album: Album) {
        const existingAlbum = await db.albums.get(album.id);
        if (existingAlbum) {
            existingAlbum.artist = album.artist;
            existingAlbum.artwork = album.artwork;
            existingAlbum.displayTitle = album.displayTitle;
            existingAlbum.genre = album.genre;
            existingAlbum.title = album.title;
            existingAlbum.year = album.year;
            existingAlbum.tracksIds = uniq([
                ...existingAlbum.tracksIds,
                ...album.tracksIds,
            ]);

            await db.albums.put(existingAlbum);
        }
    }

    async function reImportTracks(songs: Song[]) {
        await db.songs.bulkPut(songs);
    }

    export async function resetMetadata() {
        containsError = null;
        const { mappedMetadata, tagType } = await readMappedMetadataFromSong(
            $rightClickedTrack || $rightClickedTracks[0],
        );
        let metadata = mergeDefault(mappedMetadata, tagType);

        if ($rightClickedTracks.length) {
            const isCompilation =
                metadata.find((m) => m.genericId === "compilation")?.value ===
                "1";
            const hasAlbumArtist = metadata.find(
                (m) => m.genericId === "albumArtist",
            )?.value?.length;
            let hideArtist = false;

            if (!isCompilation) {
                const artist = metadata.find(
                    (m) => m.genericId === "artist",
                )?.value;

                if (artist?.length) {
                    for (let i = 1; i < $rightClickedTracks.length; i += 1) {
                        const { mappedMetadata } =
                            await readMappedMetadataFromSong(
                                $rightClickedTracks[i],
                            );
                        const newArtist = mappedMetadata.find(
                            (m) => m.genericId === "artist",
                        )?.value;

                        if (artist !== newArtist) {
                            hideArtist = true;
                            break;
                        }
                    }
                }
            }

            metadata = metadata.filter(({ genericId }) => {
                if (!ALBUM_FIELDS.includes(genericId)) {
                    return false;
                }
                if (genericId === "artist") {
                    return (
                        !isCompilation &&
                        ((hasAlbumArtist && !hideArtist) || !hasAlbumArtist)
                    );
                }
                if (genericId === "albumArtist") {
                    return !isCompilation;
                }

                return true;
            });
        }

        data = {
            mappedMetadata: metadata,
            tagType,
        };
        metadataFromFile = cloneDeep(data.mappedMetadata);
        hasChanges = !isEqual(data?.mappedMetadata, metadataFromFile);
        console.log("metadata", data);
    }

    function stripNonAsciiChars() {
        data.mappedMetadata = data?.mappedMetadata.map((entry) => ({
            ...entry,
            id: entry.id?.replace(/(\u0000)/g, "") ?? entry.id,
        }));
        console.log("stripped ascii: ", data);
        writeMetadata();
    }

    /**
     * Send an event to the backend to write the new metadata, overwriting any existing tags.
     */
    export async function writeMetadata() {
        const toWrite = data.mappedMetadata.map(({ genericId, value }) => ({
            id: genericId,
            value: value?.length ? value : null,
        }));
        console.log("Writing: ", toWrite);

        let toImport: ToImport;
        if ($rightClickedTrack) {
            const event = {
                tracks: [
                    completeEvent({
                        song_id: $rightClickedTrack.id,
                        metadata: toWrite,
                        tag_type: data.tagType,
                        file_path: $rightClickedTrack.path,
                    }),
                ],
            };

            toImport = await invoke<ToImport>("write_metadatas", { event });
        } else if ($rightClickedTracks?.length) {
            console.log("Writing album");
            toImport = await invoke<ToImport>("write_metadatas", {
                event: {
                    tracks: $rightClickedTracks.map((track) =>
                        completeEvent({
                            song_id: track.id,
                            metadata: toWrite,
                            tag_type: data.tagType,
                            file_path: track.path,
                        }),
                    ),
                },
            });
        }
        console.log($rightClickedTrack || $rightClickedTracks[0]);
        console.log(toImport);
        if (toImport) {
            if (toImport.error) {
                // Show error
                toast.error(toImport.error);
                // Roll back to current artwork
                rollbackArtwork();
            } else {
                await reImportTracks(toImport.songs);
            }
        }

        if (toImport.albums.length) {
            for (const album of toImport.albums) {
                await reImportAlbum(album);
            }
        }

        toast.success("Successfully written metadata!", {
            position: "top-right",
        });

        const writtenTracks = $rightClickedTrack
            ? [$rightClickedTrack]
            : $rightClickedTracks;

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
    }

    onMount(async () => {
        resetMetadata();
    });
</script>

<section class="metadata-section boxed">
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
                {#each data?.mappedMetadata as tag}
                    {#if tag.genericId === "artist"}
                        <div class="tag">
                            <p class="label">
                                {$LL.trackInfo.artist()}
                            </p>
                            <div class="line" />
                            <div
                                class="artist-input"
                                use:optionalTippy={{
                                    content: $LL.input.enterHintTooltip(),
                                    placement: "bottom",
                                    show:
                                        firstMatch !== null &&
                                        firstMatch.toLowerCase() !==
                                            artistInput?.toLowerCase() &&
                                        artistInput?.length > 0,
                                    showOnCreate: true,
                                    trigger: "manual",
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
                                            (e) => VALIDATION_STRINGS[e],
                                        ),
                                    )
                                    .join(","),
                                placement: "bottom",
                                theme: "error",
                                show:
                                    errors[tag.id]?.errors.length > 0 ||
                                    errors[tag.id]?.warnings.length > 0,
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
                                class:unmapped={tag.genericId === undefined}
                                data-tag-id={tag.genericId || tag.id}
                                on:click={() => onTagLabelClick(tag)}
                                use:optionalTippy={{
                                    content:
                                        $LL.trackInfo.setTitleFromFileNameHint(),
                                    show: tag.genericId === "title",
                                    placement: "bottom",
                                }}
                            >
                                {tag.genericId ? tag.genericId : tag.id}
                            </p>
                            <div class="line" />
                            <Input bind:value={tag.value} fullWidth small />
                        </div>
                    {/if}
                {/each}
            </form>
            {#if data?.mappedMetadata?.length}
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
        {/if}
    {:else}
        <p>{$LL.trackInfo.noMetadata()}</p>
    {/if}
</section>

<style lang="scss">
    .metadata-section {
        margin-top: 2em;
        border-radius: 5px;
        position: relative;
        padding: 2em 1em;
        color: var(--text);
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
            color: var(--popup-track-metadata-title);
            border: 1px solid
                rgb(from var(--popup-track-metadata-title) r g b / 0.08);
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
                border: 1px solid var(--popup-track-metadata-validation-error);
            }

            &.validation-warning {
                border: 1px solid var(--popup-track-metadata-validation-warning);
            }
        }

        .artist-input {
            /* overflow: hidden; */
        }
    }

    .tools {
        border: 1px solid rgb(from var(--inverse) r g b / 0.08);
        padding: 2em;
        margin: 2em;
        position: relative;

        .section-title {
            color: rgb(from var(--popup-track-section-title-text) r g b / 0.85);
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
        background-color: var(--popup-track-metadata-prompt-error);
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: center;
        p {
            margin: 0;
        }
    }
</style>
