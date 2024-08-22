<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog";
    import { pictureDir } from "@tauri-apps/api/path";
    import { liveQuery } from "dexie";
    import tippy from "tippy.js";
    import type { ArtistProject } from "../../App";
    import { db } from "../../data/db";
    import { isScrapbookShown, selectedArtistId } from "../../data/store";

    import { autoWidth } from "../../utils/AutoWidth";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Icon from "../ui/Icon.svelte";

    // const tabs = ["Music", "Media", "Gigs", "Info", "Analytics"]; EVENTUALLY
    const tabs = ["Music", "Info"];

    export let selectedTab = tabs[0];
    export let selectedArtist: ArtistProject;

    $: artists = liveQuery(async () => {
        let results = await db.artistProjects.toArray();
        if ($selectedArtistId === null && results.length > 0) {
            $selectedArtistId = results[0].name;
        }
        return results;
    });

    let newArtist = "";
    let showArtistAddUi = false;

    async function onCreateArtist() {
        const id = await db.artistProjects.put({
            name: newArtist,
            members: []
        });
        $selectedArtistId = await (await db.artistProjects.get(id)).name;
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
            isConfirmingArtistDelete = false;
            showMenu = false;
            const filteredArtists = $artists.filter(
                (a) => a.name !== $selectedArtistId
            );
            console.log("filteredArtists", filteredArtists);
            if (filteredArtists?.length) {
                $selectedArtistId = filteredArtists[0].name;
            }
        }
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

    async function onInput() {
        let matched = [];
        if (newArtist.trim().length > 0) {
            updateQueryPartsAutocompletePos();
            matchingArtists = [];
            if (distinctArtists === undefined) {
                distinctArtists = await db.songs.orderBy("artist").uniqueKeys();
            }
            distinctArtists.forEach((a) => {
                if (a.toLowerCase().includes(newArtist.toLowerCase())) {
                    matched.push(a);
                }
            });

            matchingArtists = matched;
        } else {
            matchingArtists = [];
        }
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
            if (typeof selected === "string") {
                selectedArtist.profilePhoto = "asset://localhost/" + selected;
            }
            db.artistProjects.update(selectedArtist.name, selectedArtist);
            // addFolder(selected);
        }
    }
</script>

<!-- <h3>Your artists and projects</h3> -->
<div class="header">
    {#if $artists?.length && $selectedArtistId}
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
                <img src={selectedArtist.profilePhoto} />
            {:else}
                <Icon icon="fa-solid:cat" />
            {/if}
        </div>
        <div class="artist-info">
            {#if $artists?.length && $selectedArtistId}
                <select bind:value={$selectedArtistId}>
                    {#each $artists as artist (artist.name)}
                        <option
                            value={artist.name}
                            class="artist"
                            on:click={() => {
                                $selectedArtistId = artist.name;
                            }}
                        >
                            <p>{artist.name}</p>
                        </option>
                    {/each}
                </select>
            {/if}

            {#if $artists?.length && $selectedArtistId}
                <div class="tabs">
                    {#each tabs as tab}
                        <p
                            class:active={tab === selectedTab}
                            on:click={() => {
                                selectedTab = tab;
                            }}
                        >
                            {tab}
                        </p>
                    {/each}
                </div>
            {/if}
        </div>
    {:else}
        <h3>Welcome to your Artist's Toolkit</h3>
        <p class="arrow">→</p>
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
        <Icon
            icon="charm:menu-kebab"
            color="#898989"
            onClick={(e) => onMenuClick(e)}
        />
        {#if $artists?.length === 0 || showArtistAddUi}
            <form on:submit|preventDefault={onCreateArtist}>
                <input
                    use:autoWidth
                    bind:this={artistInput}
                    bind:value={newArtist}
                    on:input={onInput}
                    on:focus={onFocus}
                    on:blur={onLostFocus}
                    type="text"
                    class="artist add{$artists?.length ? ' alt' : ''}"
                    placeholder="Add an artist"
                />
            </form>
        {/if}
        {#if !showArtistAddUi && $artists?.length > 0}
            <Icon
                icon="material-symbols:add"
                color="#898989"
                onClick={(e) => {
                    showArtistAddUi = true;
                }}
            />
        {/if}
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
    h3 {
        /* color: rgb(137, 130, 130); */
        font-family: Snake;
        font-size: 3em;
        margin: 0;
        margin-left: 1em;
    }

    .profile-pic {
        width: 80px;
        height: 80px;
        padding: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 2px solid rgba(255, 255, 255, 0.093);
        border-radius: 8px;
        margin-right: 0.6em;

        img {
            width: 100%;
            height: 100%;
            object-fit: cover;
            border-radius: 4px;
            box-shadow: 2px 2px 50px 40px rgba(72, 16, 128, 0.181);
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
    .header {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        align-items: center;
        gap: 4px;
        padding: 8px;
        width: 100%;
        min-height: 80px;
    }

    .artist-info {
        flex-grow: 2;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
    }

    .artist-options {
        // In onboarding mode, the artist info isn't shown, so this should take up the whole space
        &.onboarding {
            flex-grow: 2;
        }
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: flex-start;

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
