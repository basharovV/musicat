<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog";
    import { open as openShell } from "@tauri-apps/plugin-shell";
    import { pictureDir } from "@tauri-apps/api/path";
    import { liveQuery } from "dexie";
    import tippy from "tippy.js";
    import type { ArtistProject } from "../../App";
    import { db } from "../../data/db";
    import {
        isScrapbookShown,
        isSettingsOpen,
        selectedArtistId,
        userSettings
    } from "../../data/store";

    import { autoWidth } from "../../utils/AutoWidth";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Icon from "../ui/Icon.svelte";
    import Dropdown from "../ui/Dropdown.svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { currentThemeObject } from "../../theming/store";
    import Input from "../ui/Input.svelte";
    import Divider from "../ui/Divider.svelte";
    import LL from "../../i18n/i18n-svelte";

    export let selectedArtist: ArtistProject;

    $: artists = liveQuery(async () => {
        let results = await db.artistProjects.toArray();
        if ($selectedArtistId === null && results?.length > 0) {
            $selectedArtistId = results[0]?.id;
        } else if (results.length === 0) {
            $selectedArtistId = null;
        }
        return results || [];
    });

    let newArtist = "";
    let showArtistAddUi = false;

    async function onCreateArtist() {
        const id = await db.artistProjects.put({
            name: newArtist,
            members: []
        });
        console.log("id created", id);
        $selectedArtistId = await (await db.artistProjects.get(id)).id;
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
            await db.artistProjects.delete($selectedArtistId);
            $selectedArtistId = null;
            editedArtistName = null;
            isConfirmingArtistDelete = false;
            showMenu = false;
            const filteredArtists = $artists.filter(
                (a) => a.id !== $selectedArtistId
            );
            console.log("filteredArtists", filteredArtists);
            if (filteredArtists?.length) {
                $selectedArtistId = filteredArtists[0]?.id;
            }
            isEditingArtist = false;
        }
    }

    async function updateArtistName(name) {
        await db.artistProjects.update($selectedArtistId, { name });
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
            defaultPath: await pictureDir()
        });
        if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single file, update artist info
            if (typeof selected?.path === "string") {
                selectedArtist.profilePhoto = selected.path;
            }
            await db.artistProjects.put(selectedArtist);
            selectedArtist = await db.artistProjects.get($selectedArtistId);
            // addFolder(selected);
        }
    }

    let isEditingArtist = null;
    let editedArtistName = null;
</script>

<div class="header">
    {#if $artists?.length && selectedArtist && !showArtistAddUi}
        <div class="selected-artist">
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="profile-pic"
                use:tippy={{
                    theme: "slim",
                    content: "Add a profile pic",
                    placement: "bottom"
                }}
                on:click={showProfilePicPicker}
            >
                {#if selectedArtist?.profilePhoto}
                    <!-- svelte-ignore a11y-missing-attribute -->
                    <img src={convertFileSrc(selectedArtist.profilePhoto)} />
                {:else}
                    <Icon icon="fa-solid:cat" />
                {/if}
            </div>
            {#if isEditingArtist}
                <Input
                    bind:value={editedArtistName}
                    small
                    onEnterPressed={() => {
                        updateArtistName(editedArtistName);
                    }}
                />
                <Icon
                    icon="material-symbols:close"
                    onClick={() => (isEditingArtist = false)}
                    color={$currentThemeObject["icon-secondary"]}
                    size={16}
                    boxed
                />
                <Icon
                    icon="charm:menu-kebab"
                    color="#898989"
                    onClick={(e) => onMenuClick(e)}
                    size={16}
                    boxed
                />
                <Divider />
            {:else}
                <div class="artist-info">
                    {#if $artists?.length && selectedArtist}
                        <Dropdown
                            size={18}
                            selected={{
                                value: selectedArtist.id,
                                label: selectedArtist.name
                            }}
                            options={$artists.map((a) => {
                                return {
                                    value: a.id,
                                    label: a.name
                                };
                            })}
                            onSelect={(artist) => {
                                $selectedArtistId = artist;
                            }}
                        >
                            {#each $artists as artist (artist.name)}
                                <option
                                    value={artist.name}
                                    class="artist"
                                    on:click={() => {
                                        $selectedArtistId = artist.id;
                                    }}
                                >
                                    <p>{artist.name}</p>
                                </option>
                            {/each}
                        </Dropdown>
                    {/if}
                    <Icon
                        icon="ic:baseline-edit"
                        size={14}
                        onClick={() => {
                            isEditingArtist = true;
                            editedArtistName = selectedArtist.name;
                        }}
                        boxed
                        color={$currentThemeObject["icon-secondary"]}
                    />
                    <Icon
                        icon="ph:plus-fill"
                        size={14}
                        onClick={() => {
                            showArtistAddUi = true;
                        }}
                        boxed
                        color={$currentThemeObject["icon-secondary"]}
                    />
                </div>
            {/if}
        </div>
    {:else if $artists?.length === 0 || showArtistAddUi}
        <form on:submit|preventDefault={onCreateArtist}>
            <Input bind:value={newArtist} placeholder="Add an artist" small />
        </form>
        {#if $artists?.length > 0}
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

    <div class="artist-options" class:onboarding={$artists?.length === 0}>
        <div
            use:tippy={{
                content: $userSettings.songbookLocation
                    ? $LL.artistsToolkit.header.songbookLocationHint({
                          path: $userSettings.songbookLocation
                      })
                    : $LL.artistsToolkit.header.songbookLocationHintEmpty(),
                placement: "top"
            }}
        >
            <Icon
                icon="material-symbols:folder"
                onClick={() => {
                    $isSettingsOpen = true;
                }}
            ></Icon>
        </div>
        <Icon
            icon="ant-design:bulb-outlined"
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
                source: p
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
        border: 0.7px solid
            color-mix(in srgb, var(--type-bw-inverse) 15%, transparent);
    }
    h3 {
        /* color: rgb(137, 130, 130); */
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
        /* background-color: #7c7b8023; */
        border-radius: 5px;
        text-shadow: 0px 2px 20px rgba(0, 0, 0, 0.2);
        color: rgba(255, 255, 255, 0.303);
        cursor: default;
        margin-top: 8px;
        gap: 10px;

        .active {
            color: white;
            /* border: 1px solid #5123dd; */
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

        form {
            flex-grow: 1;
        }
    }

    select {
        font-size: 40px;
        padding: 0.2em 1em 0.3em 1em;
    }

    .add {
        align-items: center;
        cursor: default;
        padding: 0.2em 0.5em;
        border: 1px solid transparent;
        transition: border 0.1s ease-in-out;
        /* border-radius: 4px; */
        border-left: 1px solid rgba(255, 255, 255, 0.146);
        min-width: 180px;
        background: none;
        font-size: 16px;
        height: 28px;
        font-weight: normal;
        color: rgb(105, 105, 105);
        outline: none;

        &:focus {
            border-left: 1px solid rgba(255, 255, 255, 0.546);
        }
        &::placeholder {
            color: rgb(76, 76, 76);
        }

        &.alt {
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
        }
    }
</style>
