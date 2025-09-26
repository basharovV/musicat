<svelte:options accessors={true} />

<script lang="ts">
    import { fly } from "svelte/transition";
    import Icon from "../ui/Icon.svelte";
    import LL from "../../i18n/i18n-svelte";
    import {
        current,
        lastWrittenSongs,
        playerTime,
        rightClickedTrack,
        rightClickedTracks,
    } from "../../data/store";
    import type { MetadataEntry, TagType, ToImport } from "src/App";
    import { invoke } from "@tauri-apps/api/core";
    import {
        getAlbumId,
        readMappedMetadataFromSong,
        reImport,
    } from "../../data/LibraryUtils";
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
    import { cloneDeep, isEqual, uniqBy } from "lodash-es";
    import { onMount } from "svelte";
    import tippy from "svelte-tippy";
    import { type IndexableTypeArray } from "dexie";
    import hotkeys from "hotkeys-js";

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
    let isUnsupportedFormat: boolean;
    let metadataFromFile: MetadataEntry[] = [];
    // Encodings
    let selectedEncoding = "placeholder";

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

    $: hasChanges = !isEqual(data?.mappedMetadata, metadataFromFile);

    const defaultTags = [
        "TrackTitle",
        "TrackArtist",
        "AlbumArtist",
        "AlbumTitle",
        "Composer",
        "Genre",
        "Year",
        "TrackNumber",
        "TrackTotal",
        "DiscNumber",
        "DiscTotal",
        "Copyright",
        "Publisher",
        "Isrc",
        "Bpm",
        "Compilation",
    ];

    function addDefaults(entry: MetadataEntry[], format: TagType) {
        // Map eg. 'TITLE' (FLAC/Vorbis) -> title (Musicat generic identifier)

        // Add empty default fields
        const defaults: MetadataEntry[] = [];
        const others: MetadataEntry[] = [];
        for (const defaultTag of defaultTags) {
            // Check if this default field already exists in the file
            const existingField = entry?.find((m) => m.id === defaultTag);

            defaults.push({
                id: defaultTag,
                value: existingField ? existingField.value : null,
                values: existingField ? existingField.values : null,
                originalValue: existingField ? existingField.value : null,
            });
        }

        for (const field of entry) {
            const inDefaults = defaults.find((t) => t.id === field.id);
            if (!inDefaults) {
                others.push({
                    id: field.id,
                    value: field.value,
                    values: field.values,
                    originalValue: field.value,
                });
            }
        }

        console.log("default tags", defaults, others);

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
        // if (
        //     ($rightClickedTrack || $rightClickedTracks[0]).fileInfo.codec ===
        //     "WAV"
        // ) {
        //     isUnsupportedFormat = true;
        //     return []; // UNSUPPORTED FORMAT, for now...
        // }

        console.log("adding defaults", metadata, format);
        if (!format) return [];

        isUnsupportedFormat = false;
        const cloned = cloneDeep(metadata);
        const { defaults, others } = addDefaults(cloned, format);
        const result = uniqBy(
            [...defaults, ...others.sort((a, b) => a.id.localeCompare(b.id))],
            "id",
        );
        return result;
    }

    /**
     * Get the representation of metadata for mutliple tracks.
     * If a tag's value is common across tracks, it will be in @see {MetadataEntry.value}
     * If a tag's value is different across tracks, they will be in @see {MetadataEntry.values}
     * @param metadata the combined metadata
     */
    function combineMultipleTrackMetadata(metadata: MetadataEntry[][]) {
        const result: MetadataEntry[] = [];
        for (const field of metadata[0]) {
            // eg. tracks in an album will all have the same "album" field
            const allValues = metadata
                .map((m) => m.find((f) => f.id === field.id))
                .filter((f) => f);
            const allValuesUnique = Array.from(
                new Set(allValues.map((f) => f.value)),
            );
            if (allValuesUnique.length === 1) {
                result.push({
                    id: field.id,
                    value: allValuesUnique[0],
                    originalValue: allValuesUnique[0],
                });
            } else {
                result.push({
                    id: field.id,
                    value: null,
                    originalValue: null,
                    values: allValuesUnique,
                });
            }
        }
        return result;
    }

    // Auto complete
    const autoCompleteConfig = Object.fromEntries(
        [
            {
                id: "TrackArtist",
                songField: "artist",
            },
            {
                id: "AlbumArtist",
                songField: "artist",
            },
            {
                id: "AlbumTitle",
                songField: "album",
            },
        ].map((f) => [f.id, f]),
    );

    let autoCompleteFieldId: string = null;
    $: autoCompleteFieldValue = data.mappedMetadata?.find(
        (m) => m.id === autoCompleteFieldId,
    )?.value;

    let autoCompleteDistinctMatches: IndexableTypeArray | null = null;
    let firstMatch: string = null;

    $: firstMatchRemainder = firstMatch?.startsWith(autoCompleteFieldValue)
        ? firstMatch.substring(autoCompleteFieldValue.length, firstMatch.length)
        : "";
    $: {
        console.log(firstMatchRemainder);
    }

    function onEnterPressedInInputField(evt: KeyboardEvent) {
        console.log("Enter pressed: ", evt);
        if (firstMatch?.length && autoCompleteFieldId) {
            console.log("firstMatch", firstMatch);
            console.log("artistInput", autoCompleteFieldValue);

            // Set the field
            const field = data?.mappedMetadata?.find(
                (m) => m.id === autoCompleteFieldId,
            );
            if (field) {
                field.value = firstMatch;
                autoCompleteFieldValue = firstMatch;
            }
            data = data;
            resetAutoComplete();
        } else {
            // Do nothing
        }
    }

    function resetAutoComplete() {
        autoCompleteFieldId = null;
        autoCompleteDistinctMatches = null;
        firstMatch = null;
    }

    async function onFieldUpdated(field: MetadataEntry, value: string) {
        // First update the value
        field.value = value;

        let autoCompleteConfigForField = autoCompleteConfig[field.id];
        // Handle autocomplete
        if (autoCompleteConfigForField) {
            console.log("autocomplete enabled for", field.id);
            // Set up for auto complete if enabled
            autoCompleteFieldId = field.id;

            data = data;
            let matched = [];
            if (value && value.trim().length > 0) {
                if (autoCompleteDistinctMatches === null) {
                    autoCompleteDistinctMatches = await db.songs
                        .orderBy(autoCompleteConfigForField.songField)
                        .uniqueKeys();
                }
                console.log(
                    "found distinct matches",
                    autoCompleteDistinctMatches,
                );
                autoCompleteDistinctMatches.forEach((a) => {
                    if (
                        a
                            .toString()
                            .toLowerCase()
                            .startsWith(value.toLowerCase())
                    ) {
                        matched.push(a);
                    }
                });

                firstMatch = matched.length && matched[0] ? matched[0] : null;
            } else {
                firstMatch = null;
            }
        } else {
            autoCompleteFieldId = null;
            return;
        }
    }

    function onFieldBlur(field: MetadataEntry) {
        if (multipleValuesEditEnabledFor === field.id && !field.value) {
            multipleValuesEditEnabledFor = null;
        }
    }

    function onTagLabelClick(tag: MetadataEntry) {
        // Double-clicking on title populates the title from the filename
        if (tag.id === "title") {
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

    let multipleValuesEditEnabledFor = null;
    function onMultipleValuesClick(tag: MetadataEntry) {
        if (multipleValuesEditEnabledFor === tag.id) {
            multipleValuesEditEnabledFor = null;
        } else {
            multipleValuesEditEnabledFor = tag.id;
            const input = document.querySelector(`[data-name=` + tag.id + `]`);
            if (input instanceof HTMLInputElement) {
                input.focus();
            }
        }
    }

    export async function resetMetadata() {
        containsError = null;

        // The fields to display
        let displayMetadata: MetadataEntry[] = [];
        // The tag type to write back to file
        let tagType: TagType = null;

        if ($rightClickedTracks.length) {
            const mappedMetadata = await Promise.all(
                $rightClickedTracks.map((t) => readMappedMetadataFromSong(t)),
            );
            // Check if every track has the same tag type.
            const tagTypes = mappedMetadata.map((s) => s.tagType);
            if (tagTypes.every((t) => t === tagTypes[0])) {
                tagType = tagTypes[0];

                displayMetadata = mergeDefault(
                    combineMultipleTrackMetadata(
                        mappedMetadata.map((s) => s.mappedMetadata),
                    ),
                    tagType,
                );
            }
        } else {
            const { mappedMetadata, tagType: tag } =
                await readMappedMetadataFromSong($rightClickedTrack);
            tagType = tag;
            displayMetadata = mergeDefault(mappedMetadata, tagType);
        }

        data = {
            mappedMetadata: displayMetadata,
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
        const toWrite = data.mappedMetadata
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

        const writtenTracks = $rightClickedTrack
            ? [$rightClickedTrack]
            : $rightClickedTracks;
        const tracks = [];
        const songIdToOldAlbumId = {};

        for (const song of writtenTracks) {
            tracks.push(
                completeEvent({
                    song_id: song.id,
                    metadata: toWrite,
                    tag_type: data.tagType,
                    file_path: song.path,
                }),
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
                            class:unmapped={tag.id === undefined}
                            data-tag-id={tag.id || tag.id}
                            on:click={() => onTagLabelClick(tag)}
                            use:optionalTippy={{
                                content:
                                    $LL.trackInfo.setTitleFromFileNameHint(),
                                show: tag.id === "title",
                                placement: "bottom",
                            }}
                        >
                            {tag.id.replace(/([a-z])([A-Z])/g, "$1 $2")}
                        </p>
                        <div class="line" />
                        <div class="input">
                            <Input
                                name={tag.id}
                                value={tag.value}
                                fullWidth
                                small
                                autoCompleteValue={autoCompleteFieldId ===
                                    tag.id && firstMatch}
                                onChange={(val) => onFieldUpdated(tag, val)}
                                onEnterPressed={onEnterPressedInInputField}
                                tabBehavesAsEnter
                                onBlur={() => onFieldBlur(tag)}
                            />
                            {#if tag.values && multipleValuesEditEnabledFor !== tag.id}
                                <p
                                    class="multiple-values-label"
                                    on:click|stopPropagation={() =>
                                        onMultipleValuesClick(tag)}
                                >
                                    <Icon icon="fe:music" size={16} />Multiple
                                    Values
                                </p>
                            {/if}
                        </div>
                    </div>
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
                                use:tippy={{
                                    allowHTML: true,
                                    content:
                                        $LL.trackInfo.fixLegacyEncodings.body(),
                                    placement: "top",
                                }}><Icon icon="ic:round-info" /></small
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
                            theme="translucent"
                            size="small"
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
        border: 1px solid var(--popup-track-section-border);

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

            .input {
                position: relative;
                width: 175px;
                > .multiple-values-label {
                    width: fit-content;
                    margin: 0;
                    width: 100%;
                    height: 100%;
                    background-color: color-mix(
                        in srgb,
                        var(--inverse) 60%,
                        transparent
                    );
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex: 1;
                    gap: 5px;
                    font-weight: 500;
                    color: var(--text-secondary);
                    cursor: default;
                    &:hover {
                        background-color: color-mix(
                            in srgb,
                            var(--inverse) 40%,
                            transparent
                        );
                    }
                }
            }

            .line {
                height: 1px;
                background-color: color-mix(
                    in srgb,
                    var(--input-bg) 80%,
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
                display: flex;
                flex-direction: row;
                align-items: center;
                gap: 5px;
                flex: 1;
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
