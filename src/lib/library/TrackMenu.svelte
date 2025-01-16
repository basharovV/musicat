<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { type } from "@tauri-apps/plugin-os";
    import { onMount } from "svelte";
    import type { Album, Song, ToImport } from "../../App";
    import { db } from "../../data/db";
    import { deleteSongsFromPlaylist } from "../../data/M3UUtils";
    import {
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        popupOpen,
        rightClickedTrack,
        rightClickedTracks,
        selectedPlaylistFile,
        selectedTags,
        toDeletePlaylist,
        uiView,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { dedupe } from "../../utils/ArrayUtils";
    import { enrichSongCountry } from "../data/LibraryEnrichers";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuInput from "../ui/menu/MenuInput.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import Icon from "../ui/Icon.svelte";
    import { deleteFromLibrary } from "../../data/LibraryUtils";
    import { liveQuery } from "dexie";
    import { searchArtistOnWikiPanel } from "../menu/search";
    import { openInFinder } from "../menu/file";
    import { removeQueuedSongs } from "../../data/storeHelper";

    type ActionType = "country" | "delete" | "remove" | "remove_from_playlist";

    let confirmingType: ActionType = null;
    let explorerName: string;
    let loadingType: ActionType = null;
    let position = { x: 0, y: 0 };
    let showMenu = false;
    let song: Song;
    let songs: Song[];

    onMount(async () => {
        const os = await type();
        switch (os) {
            case "macos":
                explorerName = "Finder";
                break;
            case "windows":
                explorerName = "Explorer";
                break;
            case "linux":
                explorerName = "File manager";
                break;
        }
    });

    export function close() {
        showMenu = false;
    }

    export function isOpen() {
        return showMenu;
    }

    export function open(
        _songs: Song | Song[],
        _position: { x: number; y: number },
    ) {
        position = _position;

        if (Array.isArray(_songs)) {
            song = null;
            songs = _songs;
        } else {
            song = _songs;
            songs = [];
        }

        confirmingType = null;

        showMenu = true;
    }

    function compose(action, ...args) {
        return () => {
            close();

            action(...args);
        };
    }

    $: isConfirming = (type: ActionType): boolean => confirmingType === type;
    $: isDisabled = (type?: ActionType): boolean =>
        !!loadingType && loadingType !== type;
    $: isLoading = (type: ActionType): boolean => loadingType === type;

    /**
     * Playlists are M3U files, and in special cases (like the to-delete playlist, a separate table)
     * @param tracks
     */
    async function deleteTracksFromPlaylist(tracks: Song[]) {
        // Delete from playlist
        if ($selectedPlaylistFile) {
            // Delete directly from M3U file
            await deleteSongsFromPlaylist($selectedPlaylistFile, tracks);
            $selectedPlaylistFile = $selectedPlaylistFile; // Trigger re-render
        } else if ($uiView === "to-delete") {
            // Delete from internal playlist (currently only used for the "to-delete" playlist)
            const toDelete = await db.internalPlaylists.get(
                $toDeletePlaylist.id,
            );
            tracks.forEach((t) => {
                const trackIdx = toDelete.tracks.findIndex((pt) => pt === t.id);
                toDelete.tracks.splice(trackIdx, 1);
            });
            await db.internalPlaylists.put(toDelete);
        }
    }

    async function deleteTracksFromFileSystem(tracks: Song[]) {
        // Delete from file system (ie. move to trash)
        await invoke("delete_files", {
            event: {
                files: tracks.map((t) => t.path),
            },
        });
    }

    function removeSongs(
        type: ActionType,
        action: (songs: Song[]) => Promise<void>,
    ): () => Promise<void> {
        return async () => {
            if (loadingType) {
                return;
            }

            if (confirmingType !== type) {
                confirmingType = type;
                return;
            }

            loadingType = "remove";
            confirmingType = null;

            const tracksToRemove = songs.length ? songs : [song];

            await action(tracksToRemove);

            loadingType = null;

            close();
        };
    }

    /**
     * Removes track from the library - but not from disk!
     * i.e from songs DB, playlists
     */
    async function removeFromLibrary(songs: Song[]) {
        await deleteFromLibrary(songs);
        await deleteTracksFromPlaylist(songs);
        await removeQueuedSongs(songs.map(({ id }) => id));
    }

    async function removeFromPlaylist(songs: Song[]) {
        await deleteTracksFromPlaylist(songs);
        await removeQueuedSongs(songs.map(({ id }) => id));
    }

    /**
     * Moves the files(s) to the system trash / Recycle bin
     * Also performs deletion from DB
     */
    async function removeFromSystem(songs: Song[]) {
        await deleteTracksFromFileSystem(songs);
        await deleteFromLibrary(songs);
        await deleteTracksFromPlaylist(songs);
        await removeQueuedSongs(songs.map(({ id }) => id));
    }

    function openInfo() {
        if (song) {
            $rightClickedTracks = [];
            $rightClickedTrack = song;
        } else {
            $rightClickedTracks = songs;
            $rightClickedTrack = null;
        }

        close();

        $popupOpen = "track-info";
    }

    // Enrichers

    async function fetchingOriginCountry() {
        loadingType = "country";

        await enrichSongCountry(song);

        loadingType = null;
    }

    let isReimporting = false;
    async function reImportAlbum(album: Album) {
        const existingAlbum = await db.albums.get(album.id);
        if (existingAlbum) {
            existingAlbum.tracksIds = [
                ...existingAlbum.tracksIds,
                ...album.tracksIds,
            ];
            await db.albums.put(existingAlbum);
        } else {
            await db.albums.add(album);
        }
    }

    async function reImportTracks() {
        isReimporting = true;
        const response = await invoke<ToImport>("scan_paths", {
            event: {
                paths: song ? [song.path] : songs.map((t) => t.path),
                recursive: false,
                process_albums: true,
                is_async: false,
            },
        });
        console.log("response", response);
        await db.transaction("rw", db.songs, db.albums, async () => {
            await db.songs.bulkPut(response.songs);
            for (const album of response.albums) {
                await reImportAlbum(album);
            }
        });
        isReimporting = false;
    }

    let tagUserInput = "";

    let allTags = liveQuery(() => {
        return db.songs.orderBy("tags").uniqueKeys();
    }); // Used for autocomplete

    $: splitTags = dedupe($allTags?.flatMap((t) => t as string));

    $: tagAutoCompleteValue =
        tagUserInput.length &&
        splitTags?.find((v) => v.startsWith(tagUserInput));

    async function addTagToContextItem() {
        if (tagAutoCompleteValue?.length) {
            tagUserInput = tagAutoCompleteValue;
        }
        if (songs.length > 1) {
            await Promise.all(
                songs.map(async (t) => {
                    t.tags = t.tags
                        ? [...t.tags, tagUserInput.toLowerCase().trim()]
                        : [tagUserInput.toLowerCase().trim()];
                    await db.songs.put(t);
                }),
            );

            songs = songs;
        } else {
            await db.songs.update(song.id, {
                tags: song.tags
                    ? [...song.tags, tagUserInput.toLowerCase().trim()]
                    : [tagUserInput.toLowerCase().trim()],
            });

            song = await db.songs.get(song.id);
        }
        tagUserInput = "";
    }

    async function deleteTag(tag) {
        console.log("delete");
        if (songs.length > 1) {
            await Promise.all(
                songs.map(async (t) => {
                    t.tags = t.tags.filter((t) => t !== tag);
                    await db.songs.put(t);
                }),
            );

            songs = songs;
        } else {
            song.tags.splice(
                song.tags.findIndex((t) => t === tag),
                1,
            );

            db.songs.update(song.id, {
                tags: song.tags,
            });

            song = await db.songs.get(song.id);
        }
    }

    async function commonTagsBetweenTracks(tracks: Song[]) {
        const tags = tracks.map((t) => t.tags).flat();
        return dedupe(tags).filter(Boolean);
    }
</script>

{#if showMenu}
    <Menu {...position} onClickOutside={close} fixed>
        <MenuOption
            isDisabled={true}
            text={song ? song.title : songs.length + " tracks"}
        />
        <MenuOption
            onClick={reImportTracks}
            description="Will also re-import albums"
            text={song ? "Re-import track" : `Re-import ${songs.length} tracks`}
            isLoading={isReimporting}
            isDisabled={isDisabled()}
        />
        {#if song}
            <MenuDivider />

            <MenuOption isDisabled={true} text="Edit tags" />
            {#if song.tags}
                <div class="tags">
                    {#each song.tags as tag}
                        <!-- svelte-ignore a11y-no-static-element-interactions -->
                        <div
                            class="tag"
                            on:click={() => {
                                $isTagCloudOpen = true;
                                $isSmartQueryBuilderOpen = false;
                                $uiView = "library";
                                $selectedTags.add(tag);
                                $selectedTags = $selectedTags;
                                close();
                            }}
                        >
                            <p>{tag}</p>
                            <Icon
                                icon="mingcute:close-circle-fill"
                                size={13}
                                onClick={() => deleteTag(tag)}
                            />
                        </div>
                    {/each}
                </div>
            {/if}
            <MenuInput
                bind:value={tagUserInput}
                autoCompleteValue={tagAutoCompleteValue}
                onEnterPressed={addTagToContextItem}
                placeholder="Add a tag"
                onEscPressed={close}
                isDisabled={isDisabled()}
                small
            />
            {#if song.artist}
                <MenuDivider />
                <MenuOption text="⚡️ Enrich" isDisabled />
                <MenuOption
                    isDisabled={isDisabled()}
                    isLoading={isLoading("country")}
                    onClick={fetchingOriginCountry}
                    text={!song.originCountry
                        ? isLoading("country")
                            ? "Looking online..."
                            : "Origin country"
                        : "Origin country ✅"}
                    description="from Wikipedia"
                />
                <MenuDivider />
                <MenuOption
                    isDisabled={isDisabled()}
                    onClick={compose(searchArtistOnWikiPanel, song)}
                    text="Wiki panel: <i>{song.artist}</i>"
                />
            {/if}
            <!-- <MenuDivider />
            <MenuOption onClick={lookUpChords} text="Look up chords" />
            <MenuOption onClick={lookUpLyrics} text="Look up lyrics" /> -->
        {:else if songs.length}
            <MenuDivider />

            <MenuOption isDisabled={true} text="Edit tags" />
            {#await commonTagsBetweenTracks(songs) then tags}
                {#if tags}
                    <div class="tags">
                        {#each tags as tag}
                            <!-- svelte-ignore a11y-no-static-element-interactions -->
                            <div
                                class="tag"
                                on:click={() => {
                                    $isTagCloudOpen = true;
                                    $isSmartQueryBuilderOpen = false;
                                    $uiView = "library";
                                    $selectedTags.add(tag);
                                    $selectedTags = $selectedTags;
                                    close();
                                }}
                            >
                                <p>{tag}</p>
                                <Icon
                                    icon="mingcute:close-circle-fill"
                                    size={13}
                                    onClick={() => deleteTag(tag)}
                                />
                            </div>
                        {/each}
                    </div>
                {/if}
            {/await}
            <MenuInput
                bind:value={tagUserInput}
                autoCompleteValue={tagAutoCompleteValue}
                onEnterPressed={addTagToContextItem}
                autoFocus
                placeholder="Add a tag"
                onEscPressed={close}
                isDisabled={isDisabled()}
                small
            />
        {/if}
        <MenuDivider />

        {#if $selectedPlaylistFile || $uiView === "to-delete"}
            <MenuOption
                isDestructive={true}
                isConfirming={isConfirming("remove_from_playlist")}
                isDisabled={isDisabled("remove_from_playlist")}
                isLoading={isLoading("remove_from_playlist")}
                onClick={removeSongs(
                    "remove_from_playlist",
                    removeFromPlaylist,
                )}
                text={song
                    ? "Remove from playlist"
                    : `Remove  ${songs.length} tracks from playlist`}
                confirmText="Click again to confirm"
                description={isLoading("remove_from_playlist")
                    ? "Removing from playlist..."
                    : ""}
            />
        {/if}
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirming("remove")}
            isDisabled={isDisabled("remove")}
            isLoading={isLoading("remove")}
            onClick={removeSongs("remove", removeFromLibrary)}
            text={$LL.trackMenu.removeFromLibrary(
                songs.length ? songs.length : 1,
            )}
            confirmText="Click again to confirm"
            description={isLoading("remove") ? "Removing from library..." : ""}
        />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirming("delete")}
            isDisabled={isDisabled("delete")}
            isLoading={isLoading("delete")}
            onClick={removeSongs("delete", removeFromSystem)}
            text={$LL.trackMenu.deleteFile(songs.length ? songs.length : 1)}
            confirmText="Click again to confirm"
            description={isLoading("delete")
                ? "Moving to Trash / Recycle bin..."
                : $LL.trackMenu.deleteFileHint()}
        />
        <MenuDivider />
        {#if song}
            <MenuOption
                isDisabled={isDisabled()}
                onClick={compose(openInFinder, song)}
                text="Open in {explorerName}"
            />
        {/if}
        <MenuOption
            isDisabled={isDisabled()}
            onClick={openInfo}
            text="Info & metadata"
        />
    </Menu>
{/if}

<style lang="scss">
    .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 5px;
        margin: 5px;
        .tag {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0 0.4em 0 0.7em;
            border-radius: 20px;
            background-color: var(--library-clickable-cell-bg);
            color: var(--library-clickable-cell-text);

            p {
                margin: 0;
                position: relative;
                bottom: 1px;
                font-size: 13px;
                &:hover {
                    opacity: 0.6;
                    cursor: default;
                }
            }
        }
    }
</style>
