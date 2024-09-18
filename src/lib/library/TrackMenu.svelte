<script lang="ts">
    import { type } from "@tauri-apps/plugin-os";
    import { open } from "@tauri-apps/plugin-shell";
    import { onMount } from "svelte";
    import { db } from "../../data/db";
    import {
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        isTrackInfoPopupOpen,
        isWikiOpen,
        rightClickedTrack,
        rightClickedTracks,
        selectedPlaylistId,
        selectedTags,
        uiView,
        wikiArtist
    } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuDivider from "../menu/MenuDivider.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import { invoke } from "@tauri-apps/api/core";
    import type { Album, Song, ToImport } from "../../App";
    import MenuInput from "../menu/MenuInput.svelte";
    import { dedupe } from "../../utils/ArrayUtils";
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
    let isConfirmingDelete = false;
    let isConfirmingRemoveFromPlaylist = false;

    $: {
        if (
            $rightClickedTracks?.map((s) => s.id).includes(songId) ||
            songId !== $rightClickedTrack?.id
        ) {
            isConfirmingDelete = false;
            isConfirmingRemoveFromPlaylist = false;
        }
    }

    function closeMenu() {
        showMenu = false;
        isConfirmingDelete = false;
        isConfirmingRemoveFromPlaylist = false;
    }

    async function deleteTrack(selectedPlaylistId = null) {
        let playlist;

        console.log("delete");
        if (!isConfirmingDelete) {
            songId = $rightClickedTrack?.id;
            isConfirmingDelete = true;
            return;
        }

        if ($rightClickedTracks.length) {
            closeMenu();
            if (selectedPlaylistId) {
                playlist = await db.playlists.get(selectedPlaylistId);

                $rightClickedTracks.forEach((t) => {
                    const trackIdx = playlist.tracks.findIndex(
                        (pt) => pt === t.id
                    );
                    playlist.tracks.splice(trackIdx, 1);
                });
            }
            $rightClickedTracks.forEach((t) => {
                db.songs.delete(t.id);
            });
            $rightClickedTracks = [];
            isConfirmingDelete = false;
        } else if ($rightClickedTrack) {
            closeMenu();
            if (selectedPlaylistId) {
                playlist = await db.playlists.get(selectedPlaylistId);

                const trackIdx = playlist.tracks.findIndex(
                    (pt) => pt === $rightClickedTrack.id
                );
                playlist.tracks.splice(trackIdx, 1);
            }
            db.songs.delete($rightClickedTrack.id);
            $rightClickedTrack = null;
            isConfirmingDelete = false;
        }
    }

    async function removeFromPlaylist() {
        if (!isConfirmingRemoveFromPlaylist) {
            songId = $rightClickedTrack?.id;
            isConfirmingRemoveFromPlaylist = true;
            return;
        }
        const playlist = await db.playlists.get($selectedPlaylistId);
        if ($rightClickedTracks.length) {
            closeMenu();
            $rightClickedTracks.forEach((t) => {
                const trackIdx = playlist.tracks.findIndex((pt) => pt === t.id);
                playlist.tracks.splice(trackIdx, 1);
            });
            $rightClickedTracks = [];
            isConfirmingDelete = false;
        } else if ($rightClickedTrack) {
            closeMenu();
            const trackIdx = playlist.tracks.findIndex(
                (pt) => pt === $rightClickedTrack.id
            );
            playlist.tracks.splice(trackIdx, 1);
            $rightClickedTrack = null;
            isConfirmingDelete = false;
        }
        await db.playlists.put(playlist);
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
        $isTrackInfoPopupOpen = true;
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
        return dedupe(tags);
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
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={() => {
                if (!$selectedPlaylistId) {
                    deleteTrack();
                } else {
                    deleteTrack($selectedPlaylistId);
                }
            }}
            text={$rightClickedTrack
                ? "Remove track from library"
                : "Remove tracks from library"}
            confirmText="Click again to confirm"
        />
        <MenuOption
            onClick={reImportTracks}
            description="Will also re-import albums"
            text={$rightClickedTrack
                ? "Re-import track"
                : `Re-import ${$rightClickedTracks.length} tracks`}
            isLoading={isReimporting}
        />
        {#if $selectedPlaylistId}
            <MenuOption
                isDestructive={true}
                isConfirming={isConfirmingRemoveFromPlaylist}
                onClick={removeFromPlaylist}
                text={$rightClickedTrack
                    ? "Remove from playlist"
                    : `Remove  ${$rightClickedTracks.length} tracks from playlist`}
                confirmText="Click again to confirm"
            />
        {/if}
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
            <MenuOption onClick={lookUpChords} text="Look up chords" />
            <MenuOption onClick={lookUpLyrics} text="Look up lyrics" />
            <MenuDivider />

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
