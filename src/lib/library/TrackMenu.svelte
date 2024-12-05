<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { type } from "@tauri-apps/plugin-os";
    import { open } from "@tauri-apps/plugin-shell";
    import { onMount } from "svelte";
    import type { Album, Song, ToImport } from "../../App";
    import { db } from "../../data/db";
    import { deleteSongsFromPlaylist } from "../../data/M3UUtils";
    import {
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isWikiOpen,
        popupOpen,
        rightClickedTrack,
        rightClickedTracks,
        selectedPlaylistFile,
        selectedTags,
        toDeletePlaylist,
        uiView,
        wikiArtist
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { dedupe } from "../../utils/ArrayUtils";
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import Menu from "../menu/Menu.svelte";
    import MenuDivider from "../menu/MenuDivider.svelte";
    import MenuInput from "../menu/MenuInput.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Icon from "../ui/Icon.svelte";

    export let pos = { x: 0, y: 0 };
    export let showMenu = false;
    let explorerName: string;

    onMount(async () => {
        const os = await type();
        switch (os) {
            case "macos":
                explorerName = "Finder";
                break;
            case "windows":
                explorerName = "Explorer";
                break;
            case "Linux":
                explorerName = "File manager";
                break;
        }
        getAllTags();
    });

    let songId;
    let isDestructiveConfirmType: "remove" | "remove_from_playlist" | "delete" =
        null;
    let isConfirmingRemoveFromPlaylist = false;

    $: {
        if (
            $rightClickedTracks?.map((s) => s.id).includes(songId) ||
            songId !== $rightClickedTrack?.id
        ) {
            isDestructiveConfirmType = null;
            isConfirmingRemoveFromPlaylist = false;
        }
    }

    function closeMenu() {
        showMenu = false;
        isDestructiveConfirmType = null;
        isConfirmingRemoveFromPlaylist = false;
    }

    /**
     * Playlists are M3U files, and in special cases (like the to-delete playlist, a separate table)
     * @param tracks
     */
    async function deleteTracksFromPlaylists(tracks: Song[]) {
        // Delete from playlist
        if ($selectedPlaylistFile) {
            // Delete directly from M3U file
            deleteSongsFromPlaylist($selectedPlaylistFile, tracks);
        } else if ($uiView === "to-delete") {
            // Delete from internal playlist (currently only used for the "to-delete" playlist)
            const toDelete = await db.internalPlaylists.get($toDeletePlaylist.id);
            tracks.forEach((t) => {
                const trackIdx = toDelete.tracks.findIndex((pt) => pt === t.id);
                toDelete.tracks.splice(trackIdx, 1);
            });
            await db.internalPlaylists.put(toDelete);
        }
    }

    async function deleteTracksFromDB(tracks: Song[]) {
        tracks.forEach((t) => {
            db.songs.delete(t.id);
        });
    }

    async function deleteTracksFromFileSystem(tracks: Song[]) {
        // Delete from file system (ie. move to trash)
        await invoke("delete_files", {
            event: {
                files: tracks.map((t) => t.path)
            }
        });
    }

    /**
     * Removes track from the library - but not from disk!
     * i.e from songs DB, playlists
     */
    async function removeTrackFromLibrary() {
        console.log("[Track menu] Remove from library");
        if (isDestructiveConfirmType !== "remove") {
            songId = $rightClickedTrack?.id;
            isDestructiveConfirmType = "remove";
            return;
        }

        let tracksToRemove = [];
        if ($rightClickedTracks.length) {
            tracksToRemove = $rightClickedTracks;
        } else if ($rightClickedTrack) {
            tracksToRemove.push($rightClickedTrack);
        }

        closeMenu();

        // Delete
        await deleteTracksFromPlaylists(tracksToRemove);
        await deleteTracksFromDB(tracksToRemove);

        // Reset
        if ($rightClickedTracks.length) {
            $rightClickedTracks = [];
        } else if ($rightClickedTrack) {
            $rightClickedTrack = null;
        }
        isDestructiveConfirmType = null;
    }

    async function removeFromPlaylist() {
        if (isDestructiveConfirmType !== "remove_from_playlist") {
            isDestructiveConfirmType = "remove_from_playlist";
            songId = $rightClickedTrack?.id;
            isConfirmingRemoveFromPlaylist = true;
            return;
        }

        let tracksToRemove = [];
        if ($rightClickedTracks.length) {
            tracksToRemove = $rightClickedTracks;
        } else if ($rightClickedTrack) {
            tracksToRemove.push($rightClickedTrack);
        }

        closeMenu();
        await deleteTracksFromPlaylists(tracksToRemove);

        // Reset
        if ($rightClickedTracks.length) {
            $rightClickedTracks = [];
        } else if ($rightClickedTrack) {
            $rightClickedTrack = null;
        }
        isDestructiveConfirmType = null;
    }

    /**
     * Moves the files(s) to the system trash / Recycle bin
     * Also performs deletion from DB
     */
    async function deleteFile() {
        console.log("delete");
        if (isDestructiveConfirmType !== "delete") {
            songId = $rightClickedTrack?.id;
            isDestructiveConfirmType = "delete";
            return;
        }

        let tracksToRemove = [];
        if ($rightClickedTracks.length) {
            tracksToRemove = $rightClickedTracks;
        } else if ($rightClickedTrack) {
            tracksToRemove.push($rightClickedTrack);
        }

        closeMenu();
        await deleteTracksFromFileSystem(tracksToRemove);
        await deleteTracksFromPlaylists(tracksToRemove);
        await deleteTracksFromDB(tracksToRemove);

        // Reset
        if ($rightClickedTracks.length) {
            $rightClickedTracks = [];
        } else if ($rightClickedTrack) {
            $rightClickedTrack = null;
        }
        isDestructiveConfirmType = null;
    }

    function searchArtistOnYouTube() {
        closeMenu();
        const query = encodeURIComponent($rightClickedTrack.artist);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchSongOnYouTube() {
        closeMenu();
        const query = encodeURIComponent($rightClickedTrack.title);
        open(`https://www.youtube.com/results?search_query=${query}`);
    }
    function searchArtistOnWikipedia() {
        $wikiArtist = $rightClickedTrack.artist;
        $isWikiOpen = !$isWikiOpen;
        closeMenu();
    }
    function openInFinder() {
        closeMenu();
        const query = $rightClickedTrack.path.replace(
            $rightClickedTrack.file,
            ""
        );
        open(query);
    }
    function lookUpChords() {
        closeMenu();
        const query = encodeURIComponent(
            $rightClickedTrack.artist +
                " " +
                $rightClickedTrack.title +
                " chords"
        );
        open(`https://duckduckgo.com/?q=${query}`);
    }
    function lookUpLyrics() {
        closeMenu();
        const query = encodeURIComponent(
            $rightClickedTrack.artist +
                " " +
                $rightClickedTrack.title +
                " lyrics"
        );
        open(`https://duckduckgo.com/?q=${query}`);
    }
    function openInfo() {
        closeMenu();
        $popupOpen = "track-info";
    }
    // Enrichers

    let isFetchingOriginCountry = false;

    async function enrichArtistCountry() {
        isFetchingOriginCountry = true;
        const country = await findCountryByArtist($rightClickedTrack.artist);
        console.log("country", country);
        if (country) {
            $rightClickedTrack.originCountry = country;

            // Find all songs with this artist
            const artistSongs = await db.songs
                .where("artist")
                .equals($rightClickedTrack.artist)
                .toArray();
            artistSongs.forEach((s) => {
                s.originCountry = country;
                db.songs.update(s.id, s);
            });
        }
        isFetchingOriginCountry = false;
    }

    let isReimporting = false;
    async function reImportAlbum(album: Album) {
        const existingAlbum = await db.albums.get(album.id);
        if (existingAlbum) {
            existingAlbum.tracksIds = [
                ...existingAlbum.tracksIds,
                ...album.tracksIds
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
                paths: $rightClickedTracks.map((t) => t.path),
                recursive: false,
                process_albums: true,
                is_async: false
            }
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

    let allTags = []; // Used for autocomplete

    async function getAllTags() {
        allTags = await db.songs.orderBy("tags").uniqueKeys();
    }

    $: splitTags = dedupe(allTags?.flatMap((t) => t));

    $: tagAutoCompleteValue =
        tagUserInput.length &&
        splitTags?.find((v) => v.startsWith(tagUserInput));

    async function addTagToContextItem() {
        if (tagAutoCompleteValue?.length) {
            tagUserInput = tagAutoCompleteValue;
        }
        if ($rightClickedTracks.length > 1) {
            await Promise.all(
                $rightClickedTracks.map(async (t) => {
                    t.tags = t.tags
                        ? [...t.tags, tagUserInput.toLowerCase().trim()]
                        : [tagUserInput.toLowerCase().trim()];
                    await db.songs.put(t);
                })
            );
            $rightClickedTracks = $rightClickedTracks;
        } else {
            await db.songs.update($rightClickedTrack.id, {
                tags: $rightClickedTrack.tags
                    ? [
                          ...$rightClickedTrack.tags,
                          tagUserInput.toLowerCase().trim()
                      ]
                    : [tagUserInput.toLowerCase().trim()]
            });
            $rightClickedTrack = await db.songs.get($rightClickedTrack.id);
        }
        tagUserInput = "";
    }

    async function deleteTag(tag) {
        console.log("delete");
        if ($rightClickedTracks.length > 1) {
            await Promise.all(
                $rightClickedTracks.map(async (t) => {
                    t.tags = t.tags.filter((t) => t !== tag);
                    await db.songs.put(t);
                })
            );
            $rightClickedTracks = $rightClickedTracks;
        } else {
            $rightClickedTrack.tags.splice(
                $rightClickedTrack.tags.findIndex((t) => t === tag),
                1
            );
            $rightClickedTrack = $rightClickedTrack;
            db.songs.update($rightClickedTrack.id, {
                tags: $rightClickedTrack.tags
            });
            $rightClickedTrack = await db.songs.get($rightClickedTrack.id);
        }
    }

    async function commonTagsBetweenTracks(tracks: Song[]) {
        const tags = tracks.map((t) => t.tags).flat();
        return dedupe(tags).filter(Boolean);
    }
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu} fixed>
        <MenuOption
            isDisabled={true}
            text={$rightClickedTrack
                ? $rightClickedTrack.title
                : $rightClickedTracks.length + " tracks"}
        />
        <MenuOption
            onClick={reImportTracks}
            description="Will also re-import albums"
            text={$rightClickedTrack
                ? "Re-import track"
                : `Re-import ${$rightClickedTracks.length} tracks`}
            isLoading={isReimporting}
        />
        {#if $rightClickedTrack}
            <MenuDivider />

            <MenuOption isDisabled={true} text="Edit tags" />
            {#if $rightClickedTrack.tags}
                <div class="tags">
                    {#each $rightClickedTrack.tags as tag}
                        <!-- svelte-ignore a11y-no-static-element-interactions -->
                        <div
                            class="tag"
                            on:click={() => {
                                $isTagCloudOpen = true;
                                $isSmartQueryBuilderOpen = false;
                                $uiView = "library";
                                $selectedTags.add(tag);
                                $selectedTags = $selectedTags;
                                closeMenu();
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
                onEscPressed={closeMenu}
                small
            />

            <MenuDivider />
            <MenuOption text="Enrich" isDisabled />
            <MenuOption
                onClick={enrichArtistCountry}
                text={!$rightClickedTrack.originCountry
                    ? isFetchingOriginCountry
                        ? "Looking online..."
                        : "Origin country"
                    : "Origin country ✅"}
                description="from Wikipedia"
            />
            <MenuDivider />
            <MenuOption
                onClick={searchArtistOnWikipedia}
                text="Open wiki panel for {$rightClickedTrack.artist}"
            />
            <MenuDivider />
            <!-- <MenuOption onClick={lookUpChords} text="Look up chords" />
            <MenuOption onClick={lookUpLyrics} text="Look up lyrics" /> -->

            <MenuOption onClick={openInFinder} text="Open in {explorerName}" />
        {:else if $rightClickedTracks.length}
            <MenuDivider />

            <MenuOption isDisabled={true} text="Edit tags" />
            {#await commonTagsBetweenTracks($rightClickedTracks) then tags}
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
                                    closeMenu();
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
                onEscPressed={closeMenu}
                small
            />
        {/if}
        <MenuDivider />

        {#if $selectedPlaylistFile || $uiView === "to-delete"}
            <MenuOption
                isDestructive={true}
                isConfirming={isDestructiveConfirmType ===
                    "remove_from_playlist"}
                onClick={removeFromPlaylist}
                text={$rightClickedTrack
                    ? "Remove from playlist"
                    : `Remove  ${$rightClickedTracks.length} tracks from playlist`}
                confirmText="Click again to confirm"
            />
        {/if}
        <MenuOption
            isDestructive={true}
            isConfirming={isDestructiveConfirmType === "remove"}
            onClick={() => {
                removeTrackFromLibrary();
            }}
            text={$LL.trackMenu.removeFromLibrary(
                $rightClickedTracks.length ? $rightClickedTracks.length : 1
            )}
            confirmText="Click again to confirm"
        />
        <MenuOption
            isDestructive={true}
            isConfirming={isDestructiveConfirmType === "delete"}
            onClick={() => {
                deleteFile();
            }}
            description={$LL.trackMenu.deleteFileHint()}
            text={$LL.trackMenu.deleteFile(
                $rightClickedTracks.length ? $rightClickedTracks.length : 1
            )}
            confirmText="Click again to confirm"
        />
        <MenuDivider />

        <MenuOption onClick={openInfo} text="Info & metadata" />
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
