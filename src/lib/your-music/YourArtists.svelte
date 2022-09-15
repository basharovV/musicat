<script lang="ts">
    import { liveQuery } from "dexie";
    import type { ArtistContentItem, ArtistItemType } from "src/App";
    import { db } from "../../data/db";

    import { autoWidth } from "../../utils/AutoWidth";
    import Menu from "../menu/Menu.svelte";

    $: artists = liveQuery(async () => {
        let results = await db.artistProjects.toArray();
        if (selectedArtist === undefined && results.length > 0) {
            selectedArtist = results[0];
        }
        return results;
    });

    export let selectedArtist;

    let newArtist = "";

    async function onCreateArtist() {
        const id = await db.artistProjects.put({
            name: newArtist
        });
        selectedArtist = await db.artistProjects.get(id);
        newArtist = "";
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
    {#if $artists?.length && selectedArtist}
        <select bind:value={selectedArtist.name}>
            {#each $artists as artist (artist.name)}
                <option
                    value={artist.name}
                    class="artist"
                    on:click={() => {
                        selectedArtist = artist;
                    }}
                >
                    <p>{artist.name}</p>
                </option>
            {/each}
        </select>
    {/if}
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
            position='manual'
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
        gap: 20px;
        margin-left: -10px;
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
