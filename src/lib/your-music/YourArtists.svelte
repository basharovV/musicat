<script lang="ts">
    import { pictureDir } from "@tauri-apps/api/path";
    import { open } from "@tauri-apps/plugin-dialog";
    import { open as openShell } from "@tauri-apps/plugin-shell";
    import tippy from "tippy.js";
    import type { ArtistProject, LookForArtResult } from "../../App";
    import {
        isScrapbookShown,
        popupOpen,
        songbookArtists,
        songbookSelectedArtist,
        userSettings,
    } from "../../data/store";

    import { convertFileSrc } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { loadArtistsFromSongbook } from "../../data/ArtistsToolkitData";
    import LL from "../../i18n/i18n-svelte";
    import { currentThemeObject } from "../../theming/store";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Divider from "../ui/Divider.svelte";
    import Dropdown from "../ui/Dropdown.svelte";
    import Icon from "../ui/Icon.svelte";
    import Input from "../ui/Input.svelte";
    import {
        copyFile,
        readDir,
        remove,
        rename,
        mkdir,
    } from "@tauri-apps/plugin-fs";
    import { getArtistProfileImage } from "../../data/LibraryUtils";
    import { createTippy, optionalTippy } from "../ui/TippyAction";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    let currentArtistProfilePic: LookForArtResult;

    onMount(async () => {
        $songbookArtists = await loadArtistsFromSongbook();
    });

    $: if ($songbookArtists) {
        if (
            !$songbookSelectedArtist ||
            !$songbookArtists.find((a) => a === $songbookSelectedArtist)
        ) {
            $songbookSelectedArtist = $songbookArtists[0];
        }
    }

    $: if ($songbookSelectedArtist) {
        loadCurrentArtist();
    }

    async function loadCurrentArtist() {
        if ($songbookSelectedArtist) {
            const artistPath = `${$userSettings.songbookLocation}/${$songbookSelectedArtist.name}`;
            // Set profile pic
            currentArtistProfilePic = await getArtistProfileImage(artistPath);
        }
    }

    async function selectArtist(artistName: string) {
        console.log("selecting artist", artistName);
        $songbookSelectedArtist = $songbookArtists?.find(
            (a) => a.name === artistName,
        );
        await loadCurrentArtist();
    }

    let newArtist = "";
    let showArtistAddUi = false;

    async function onCreateArtist() {
        await mkdir(`${$userSettings.songbookLocation}/${newArtist}`);
        $songbookArtists = await loadArtistsFromSongbook();
        $songbookSelectedArtist = $songbookArtists.find(
            (a) => a.name === newArtist,
        );
        newArtist = "";
        showArtistAddUi = false;
    }

    let showMenu = false;

    let menuPos = { x: 0, y: 0 };

    function onMenuClick(e) {
        showMenu = !showMenu;
        menuPos = { x: e.clientX, y: e.clientY };
    }

    let isConfirmingArtistDelete = false;

    async function deleteArtist() {
        if (!isConfirmingArtistDelete) {
            isConfirmingArtistDelete = true;
            return;
        } else {
            await remove(
                `${$userSettings.songbookLocation}/${$songbookSelectedArtist.name}`,
                {
                    recursive: true,
                },
            );
            $songbookArtists = await loadArtistsFromSongbook();
            $songbookSelectedArtist =
                $songbookArtists?.length > 0 ? $songbookArtists[0] : null;
            editedArtistName = null;
            isConfirmingArtistDelete = false;
            showMenu = false;
            isEditingArtist = false;
        }
    }

    async function updateArtistName(name) {
        await rename(
            `${$userSettings.songbookLocation}/${$songbookSelectedArtist.name}`,
            `${$userSettings.songbookLocation}/${name}`,
        );
        $songbookSelectedArtist.name = name;
        $songbookArtists = await loadArtistsFromSongbook();
        editedArtistName = null;
        isEditingArtist = false;
    }

    let matchingArtists: string[] = [];

    let distinctArtists;

    function onFocus() {
        console.log("onfocus");
        setTimeout(() => {
            if (newArtist.trim().length === 0) {
                updateQueryPartsAutocompletePos();
                matchingArtists = matchingArtists;
                console.log("matching parts", matchingArtists);
            }
        }, 150);
    }

    function onLostFocus() {
        matchingArtists = [];
    }

    function onSelectArtist(artist) {
        newArtist = artist;
        onCreateArtist();
        matchingArtists = [];
    }

    function closeAutoComplete() {
        matchingArtists = [];
    }

    let artistInput: HTMLInputElement;
    let inputX: number;
    let inputY: number;

    function updateQueryPartsAutocompletePos() {
        inputX = artistInput.offsetLeft;
        inputY = artistInput.offsetTop + 40;
        console.log("X", inputX, "Y", inputY);
    }

    async function showProfilePicPicker() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: await pictureDir(),
        });
        if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single file, update artist info
            if (typeof selected?.path === "string") {
                // Delete any existing images called profile.xxx in the artist folder
                const artistPath = `${$userSettings.songbookLocation}/${$songbookSelectedArtist.name}`;
                const files = await readDir(artistPath);
                for (const file of files) {
                    if (file.name.startsWith("profile.")) {
                        await remove(`${artistPath}/${file.name}`);
                    }
                }

                // Copy the file
                const extension = selected.path.split(".").pop();
                const dest = `${$userSettings.songbookLocation}/${$songbookSelectedArtist.name}/profile.${extension}`;
                await copyFile(selected.path, dest);
                await loadCurrentArtist();
            }
        }
    }

    let isEditingArtist = null;
    let editedArtistName = null;
</script>

<div class="header">
    {#if $songbookArtists?.length && $songbookSelectedArtist && !showArtistAddUi}
        <div class="selected-artist">
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="profile-pic"
                use:tippy={{
                    content: "Add a profile pic",
                    placement: "bottom",
                }}
                on:click={showProfilePicPicker}
            >
                {#if currentArtistProfilePic}
                    <!-- svelte-ignore a11y-missing-attribute -->
                    <img src={currentArtistProfilePic.artworkSrc} />
                {:else}
                    <Icon icon="fa-solid:cat" />
                {/if}
            </div>
            {#if isEditingArtist}
                <div class="edit-options">
                    <Input
                        bind:value={editedArtistName}
                        small
                        onEnterPressed={() => {
                            updateArtistName(editedArtistName);
                        }}
                    />
                    <ButtonWithIcon
                        icon="charm:tick"
                        disabled={editedArtistName ===
                            $songbookSelectedArtist.name}
                        onClick={() => {
                            updateArtistName(editedArtistName);
                        }}
                        text="Save"
                        size="small"
                    />
                    <Icon
                        icon="material-symbols:close"
                        onClick={() => (isEditingArtist = false)}
                        color={$currentThemeObject["icon-secondary"]}
                        size={16}
                        boxed
                    />
                    <Divider />

                    <Icon
                        icon="charm:menu-kebab"
                        color="#898989"
                        onClick={(e) => onMenuClick(e)}
                        size={16}
                        boxed
                    />
                    <!--
                    <ButtonWithIcon
                        icon="ant-design:delete-outlined"
                        disabled={editedArtistName === $songbookSelectedArtist.name}
                        onClick={() => {
                            deleteArtist();
                        }}
                        text="Delete artist"
                        size="small"
                        isDestructive
                    />
                    <Divider /> -->
                </div>
            {:else}
                <div class="artist-info">
                    {#if $songbookArtists?.length && $songbookSelectedArtist}
                        <Dropdown
                            size={18}
                            selected={{
                                value: $songbookSelectedArtist.name,
                                label: $songbookSelectedArtist.name,
                            }}
                            options={$songbookArtists.map((a) => {
                                return {
                                    value: a.name,
                                    label: a.name,
                                };
                            })}
                            onSelect={(artist) => {
                                selectArtist(artist);
                            }}
                        >
                            {#each $songbookArtists as artist (artist.name)}
                                <option
                                    value={artist.name}
                                    class="artist"
                                    on:click={() => {
                                        $songbookSelectedArtist = artist;
                                    }}
                                >
                                    <p>{artist.name}</p>
                                </option>
                            {/each}
                        </Dropdown>
                    {/if}
                    <Icon
                        icon="ic:baseline-edit"
                        size={16}
                        onClick={() => {
                            isEditingArtist = true;
                            editedArtistName = $songbookSelectedArtist.name;
                        }}
                        boxed
                        color={$currentThemeObject["icon-secondary"]}
                        tooltip={{
                            content: "Edit artist",
                            placement: "bottom",
                        }}
                    />
                    <Icon
                        icon="ph:plus-fill"
                        size={16}
                        onClick={() => {
                            showArtistAddUi = true;
                        }}
                        boxed
                        color={$currentThemeObject["icon-secondary"]}
                        tooltip={{
                            content: "Add new artist",
                            placement: "bottom",
                        }}
                    />
                </div>
            {/if}
        </div>
    {:else if $songbookArtists?.length === 0 || showArtistAddUi}
        <form class="add-artist" on:submit|preventDefault={onCreateArtist}>
            <Input bind:value={newArtist} placeholder="Add an artist" small />
        </form>
        {#if $songbookArtists?.length > 0}
            <Icon
                icon="material-symbols:close"
                onClick={() => (showArtistAddUi = false)}
                color={$currentThemeObject["icon-secondary"]}
                size={16}
                boxed
            />
        {/if}
    {/if}

    {#if showMenu}
        <Menu
            x={menuPos.x}
            y={menuPos.y}
            onClickOutside={() => {
                showMenu = false;
            }}
            position="manual"
        >
            <MenuOption
                text="Delete artist"
                confirmText="Are you sure?"
                isDestructive
                isConfirming={isConfirmingArtistDelete}
                onClick={deleteArtist}
            />
        </Menu>
    {/if}

    <div
        class="artist-options"
        class:onboarding={$songbookArtists?.length === 0}
    >
        <div
            use:tippy={{
                content: $userSettings.songbookLocation
                    ? $LL.artistsToolkit.header.songbookLocationHint({
                          path: $userSettings.songbookLocation,
                      })
                    : $LL.artistsToolkit.header.songbookLocationHintEmpty(),
                placement: "top",
            }}
        >
            <Icon
                icon="material-symbols:folder"
                onClick={() => {
                    if ($userSettings.songbookLocation) {
                        openShell($userSettings.songbookLocation);
                    } else {
                        $popupOpen = "settings";
                    }
                }}
            ></Icon>
        </div>
        <Divider />
        <Icon
            icon="ant-design:bulb-outlined"
            tooltip={{
                content: $isScrapbookShown
                    ? $LL.artistsToolkit.header.hideScrapbook()
                    : $LL.artistsToolkit.header.showScrapbook(),
                placement: "bottom",
            }}
            onClick={() => {
                $isScrapbookShown = !$isScrapbookShown;
            }}
        />
    </div>

    {#if matchingArtists.length > 0}
        <Menu
            x={inputX}
            y={inputY}
            onClickOutside={closeAutoComplete}
            items={matchingArtists.map((p) => ({
                text: p,
                description: "",
                source: p,
            }))}
            position="manual"
            onItemSelected={onSelectArtist}
        />
    {/if}
</div>

<style lang="scss">
    .header {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        align-items: center;
        gap: 4px;
        width: 100%;
        height: 44px;
        padding: 4px;
        background-color: var(--panel-background);
        border-radius: 5px;
        border: 0.7px solid var(--panel-primary-border-accent2);
    }
    h3 {
        font-family: Snake;
        font-size: 3em;
        margin: 0;
        margin-left: 1em;
    }

    .profile-pic {
        height: 32px;
        object-fit: cover;
        width: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid rgba(255, 255, 255, 0.093);
        border-radius: 8px;
        margin-right: 0.6em;

        img {
            width: 100%;
            height: 100%;
            object-fit: cover;
            border-radius: 4px;
            box-shadow: 2px 2px 50px 40px
                color-mix(in srgb, var(--accent-secondary) 8%, transparent);
        }
        &:hover {
            background-color: #5150523a;
            img {
                opacity: 0.6;
            }
        }
        &:active {
            background-color: #51505216;
            opacity: 0.4;
        }
    }

    .tabs {
        flex-grow: 2;
        display: flex;
        margin-left: 2px;
        flex-direction: row;
        border-radius: 5px;
        text-shadow: 0px 2px 20px rgba(0, 0, 0, 0.2);
        color: rgba(255, 255, 255, 0.303);
        cursor: default;
        margin-top: 8px;
        gap: 10px;

        .active {
            color: white;
        }

        p {
            border: 1px solid transparent;
            font-size: 1.1em;
            vertical-align: middle;
            /* border-radius: 8px; */
            user-select: none;
            margin: 0;
            border-radius: 5px;

            &:hover:not(.active) {
                color: rgba(255, 255, 255, 0.503);
            }
        }
    }

    .arrow {
        margin: 0 1em;
    }

    .selected-artist {
        display: flex;
        flex-grow: 1;
        height: 100%;
    }

    .edit-options {
        display: flex;
        flex-direction: row;
        align-items: center;
        height: 100%;
        gap: 5px;
    }

    .artist-info {
        flex-grow: 2;
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .artist-options {
        // In onboarding mode, the artist info isn't shown, so this should take up the whole space

        flex: 1;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: flex-end;
        padding: 0 1em;
        gap: 10px;
        height: 100%;

        form {
            flex-grow: 1;
        }
    }

    select {
        font-size: 40px;
        padding: 0.2em 1em 0.3em 1em;
    }
    .add-artist {
        padding: 0 0.2em;
    }
</style>
