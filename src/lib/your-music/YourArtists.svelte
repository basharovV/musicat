<script lang="ts">
    import { liveQuery } from "dexie";
    import { text } from "svelte/internal";
    import { db } from "../../data/db";
    import { isScrapbookShown, selectedArtistId } from "../../data/store";

    import { autoWidth } from "../../utils/AutoWidth";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Icon from "../ui/Icon.svelte";

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
</script>

<!-- <h3>Your artists and projects</h3> -->
<div class="artists">
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

            <Icon
                icon="charm:menu-kebab"
                color="#898989"
                onClick={(e) => onMenuClick(e)}
            />
        {/if}
    </div>
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
    <div class="artist-options">
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
        <button
            on:click={() => {
                $isScrapbookShown = !$isScrapbookShown;
            }}>{$isScrapbookShown ? "Hide" : "Show"} scrapbook</button
        >
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
        color: rgb(137, 130, 130);
    }
    .artists {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        align-items: center;
        gap: 4px;
        margin-left: -10px;
        width: 100%;
    }

    .artist-info {
        flex-grow: 1;
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .artist-options {
        flex-grow: 2;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: flex-end;
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
        border-radius: 4px;
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
