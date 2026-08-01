<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { type } from "@tauri-apps/plugin-os";
    import { onMount } from "svelte";
    import toast from "svelte-french-toast";
    import { get } from "svelte/store";
    import type { Song, Stem, ToImport } from "../../App";
    import { deleteFromLibrary, reImport } from "../../data/LibraryUtils";
    import { deleteSongsFromPlaylist } from "../../data/M3UUtils";
    import {
        canShowInfoPopup,
        isSmartQueryBuilderOpen,
        isTagCloudOpen,
        libraryUpdater,
        popupOpen,
        refreshToDeletePlaylist,
        rightClickedAlbum,
        rightClickedTracks,
        selectedPlaylistFile,
        selectedTags,
        songToSeparate,
        toDeletePlaylist,
        uiView,
        userSettings,
    } from "../../data/store";
    import { removeQueuedSongs } from "../../data/storeHelper";
    import LL from "../../i18n/i18n-svelte";
    import { dedupe } from "../../utils/ArrayUtils";
    import { enrichSongCountry } from "../../data/LibraryEnrichers";
    import { openInFinder } from "../menu/file";
    import { searchArtistOnWikiPanel } from "../menu/search";
    import audioPlayer from "../player/AudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuInput from "../ui/menu/MenuInput.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import { closeCurrentMenu } from "../ui/ContextMenu";

    type ActionType = "country" | "delete" | "remove" | "remove_from_playlist";

    export let onUnselect: () => void;

    let confirmingType: ActionType = null;
    let explorerName: string;
    let loadingType: ActionType = null;

    export let songs: Song[] = [];
    let song: Song;

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

        if (songs.length === 1) {
            song = songs[0];
            getStems(song);
        } else {
            song = null;
            stems = [];
        }
    });

    export function close() {
        closeCurrentMenu();
    }

    let stems: Stem[] = [];

    async function getStems(song: Song) {
        stems = await invoke("get_stems", {
            event: {
                song_id: song.id,
            },
        });
        console.log("Got stems", stems);
    }

    async function playStem(stem: Stem) {
        audioPlayer.playSong({
            ...song,
            title: stem.name,
            path: stem.path,
        });
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
            const remainingTrackIds = $toDeletePlaylist.tracks.filter(
                (id) => !tracks.some((t) => t.id === id),
            );
            await invoke("set_playlist_tracks", {
                playlistId: $toDeletePlaylist.id,
                trackIds: remainingTrackIds,
            });
            await refreshToDeletePlaylist();
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

            onUnselect && onUnselect();

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
            $rightClickedTracks = [song];
        } else {
            $rightClickedTracks = songs;
        }
        $rightClickedAlbum = null;

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

    async function reImportTracks() {
        isReimporting = true;

        const response = await invoke<ToImport>("scan_paths", {
            event: {
                paths: song ? [song.path] : songs.map((t) => t.path),
                recursive: false,
                process_albums: true,
                process_m3u: false,
                is_async: false,
                is_cover_fullcheck: $userSettings.isCoverFullCheckEnabled,
            },
        });
        console.log("response", response);

        if (response) {
            if (response.error) {
                // Show error
                toast.error(response.error);
            } else {
                reImport(response);

                toast.success(
                    `Successfully re-imported track${song ? "" : "s"}!`,
                    {
                        position: "top-right",
                    },
                );
            }
        }

        isReimporting = false;
    }

    let tagUserInput = "";

    let splitTags: string[] = [];
    invoke<string[]>("get_distinct_track_values", { field: "tags" }).then(
        (tags) => (splitTags = tags),
    ); // Used for autocomplete, already distinct

    $: tagAutoCompleteValue =
        tagUserInput.length &&
        splitTags?.find((v) => v.startsWith(tagUserInput));

    async function addTagToContextItem() {
        if (tagAutoCompleteValue?.length) {
            tagUserInput = tagAutoCompleteValue;
        }
        const newTag = tagUserInput.toLowerCase().trim();
        if (songs.length > 1) {
            await Promise.all(
                songs.map(async (t) => {
                    t.tags = t.tags ? [...t.tags, newTag] : [newTag];
                    await invoke("update_track_tags", { trackId: t.id, tags: t.tags });
                }),
            );

            songs = songs;
        } else {
            const tags = song.tags ? [...song.tags, newTag] : [newTag];
            await invoke("update_track_tags", { trackId: song.id, tags });
            song = await invoke<Song>("get_track_by_id", { id: song.id });
        }
        libraryUpdater.update((n) => n + 1);
        tagUserInput = "";
    }

    async function deleteTag(tag) {
        console.log("delete");
        if (songs.length > 1) {
            await Promise.all(
                songs.map(async (t) => {
                    t.tags = t.tags.filter((t) => t !== tag);
                    await invoke("update_track_tags", { trackId: t.id, tags: t.tags });
                }),
            );

            songs = songs;
        } else {
            song.tags.splice(
                song.tags.findIndex((t) => t === tag),
                1,
            );

            await invoke("update_track_tags", { trackId: song.id, tags: song.tags });

            song = await invoke<Song>("get_track_by_id", { id: song.id });
        }
        libraryUpdater.update((n) => n + 1);
    }

    async function commonTagsBetweenTracks(tracks: Song[]) {
        const tags = tracks.map((t) => t.tags).flat();
        return dedupe(tags).filter(Boolean);
    }
    async function separateStems() {
        try {
            await invoke("separate_stems", {
                event: {
                    path: song.path,
                },
            });
            $songToSeparate = song;
        } catch (error) {
            toast.error(error);
            $songToSeparate = null;
        }
        close();
    }
</script>

<Menu onClickOutside={close}>
    <MenuOption
        isDisabled={true}
        text={song ? song.title : songs.length + " tracks"}
    />
    <MenuOption
        onClick={reImportTracks}
        description={$LL.trackMenu.reImportTrackHint() ||
            "Will also re-import albums"}
        text={song
            ? $LL.trackMenu.reImportTrack() || "Re-Import Track"
            : $LL.trackMenu.reImportTracks({ n: songs.length }) ||
              `Re-import ${songs.length} tracks`}
        isLoading={isReimporting}
        isDisabled={isDisabled()}
    />
    {#if song}
        <MenuDivider />
        <MenuOption
            isDisabled={true}
            text={$LL.trackMenu.separateTitle() || "Stems (click to play)"}
        />
        {#if stems.length === 0}
            <MenuOption
                onClick={separateStems}
                text={$LL.trackMenu.separateStems() || "Separate stems"}
            />
        {:else}
            {#each stems as stem}
                <MenuOption
                    isDisabled={false}
                    text={stem.name}
                    onClick={() => {
                        playStem(stem);
                    }}
                />
            {/each}
        {/if}

        <MenuDivider />
        <MenuOption
            isDisabled={true}
            text={$LL.trackMenu.editTag() || "Edit tags"}
        />
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
            <MenuOption
                text={$LL.trackMenu.enrich() || "⚡️ Enrich"}
                isDisabled
            />
            <MenuOption
                isDisabled={isDisabled()}
                isLoading={isLoading("country")}
                onClick={fetchingOriginCountry}
                text={!song.originCountry
                    ? isLoading("country")
                        ? $LL.trackMenu.lookingOnline() || "Looking online..."
                        : $LL.trackMenu.originCountry() || "Origin country"
                    : $LL.trackMenu.originCountryC() || "Origin country ✅"}
                description={$LL.trackMenu.originCountryHint() ||
                    "from Wikipedia"}
            />
            <MenuDivider />
            <MenuOption
                isDisabled={isDisabled()}
                onClick={compose(searchArtistOnWikiPanel, song)}
                text="{$LL.trackMenu.wikiPanel() || 'Wiki panel:'}
                <i>{song.artist}</i>"
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
            onClick={removeSongs("remove_from_playlist", removeFromPlaylist)}
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
        text={$LL.trackMenu.removeFromLibrary(songs.length ? songs.length : 1)}
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
            text={$LL.trackMenu.openInFileManager({ explorerName }) ||
                `Open in ${explorerName}`}
        />
    {/if}
    {#if get(canShowInfoPopup)}
        <MenuOption
            isDisabled={isDisabled()}
            onClick={openInfo}
            text={$LL.trackMenu.infoMetadata() || "Info & metadata"}
        />
    {/if}
</Menu>

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
