<script lang="ts">
    import { liveQuery } from "dexie";
    import { db } from "../../data/db";

    import SmartQuery from "../../data/SmartQueries";

    import {
        isSmartQueryBuilderOpen,
        isSmartQueryUiOpen,
        isSmartQueryValid,
        uiView,
        selectedSmartQuery,
        smartQuery,
        smartQueryInitiator
    } from "../../data/store";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";

    $: savedSmartQueries = liveQuery(async () => {
        return db.smartQueries.toArray();
    });

    function hideSmartQuery() {
        $uiView = 'library';
    }

    function hideSmartQueryBuilder() {
        if ($smartQueryInitiator === 'genre-pill') {
            $uiView = 'library';
        }
        $isSmartQueryBuilderOpen = false;
    }

    function showSmartQueryBuilder() {
        $isSmartQueryBuilderOpen = true;
    }

    function save() {
        $smartQuery.save();
        // Close the builder UI and set the current selected query to the one we just saved
        $isSmartQueryBuilderOpen = false;
        $selectedSmartQuery = `~usq:${$smartQuery.name}`;
        $smartQuery.reset();
    }
</script>

{#if !$isSmartQueryBuilderOpen}
    <div class="query-browser">
        <div>
            <iconify-icon
                class="close-icon"
                icon="mingcute:close-circle-fill"
                on:click={hideSmartQuery}
            />
        </div>
        <div class="query-header">
            <p class="query-header-title">Showing results for</p>
            <select bind:value={$selectedSmartQuery}>
                {#each SmartQuery as query}
                    <option value={query.value}>{query.name}</option>
                {/each}
                <option value="----">----</option>

                {#if $savedSmartQueries}
                    {#each $savedSmartQueries as query}
                        <option value={`~usq:${query.name}`}
                            >{query.name}</option
                        >
                    {/each}
                {/if}
            </select>
        </div>
        <div>
            <button on:click={showSmartQueryBuilder}
                >{$isSmartQueryBuilderOpen
                    ? "Save smart query"
                    : "New smart query"}</button
            >
        </div>
    </div>
{:else}
    <div class="query-editor-info">
        <ButtonWithIcon
            icon="mingcute:close-circle-fill"
            onClick={hideSmartQueryBuilder}
            text="Hide builder"
        />

        <div class="smart-query-actions">
            <p>Name:</p>
            <input bind:value={$smartQuery.name} />
            <img src="images/arrow-down-right.svg" />
            <button
                disabled={!$isSmartQueryValid || !$smartQuery.isNameSet}
                on:click={save}>Save smart query</button
            >
        </div>
    </div>
{/if}

<style lang="scss">
    .close-icon {
        cursor: default;
    }

    .query-browser {
        align-items: center;
        justify-content: space-between;
        display: grid;
        grid-template-columns: auto auto 1fr;
        gap: 1em;
        > :nth-child(3) {
            display: flex;
            justify-content: flex-end;
        }

        .query-header {
            display: flex;
            flex-direction: row;
            padding: 0em 0.2em 0 1em;
            background-color: rgba(114, 86, 190, 0.338);
            align-items: center;
            border-radius: 4px;
            justify-content: flex-start;
            gap: 10px;
            .query-header-title {
                margin: 3px 0;
                font-size: 14px;
                font-weight: 600;
            }

            select {
                font-size: 18px;
                color: rgb(51, 51, 51);
                /* background-color: rgb(220, 208, 237); */
                outline: none;
                opacity: 0.9;
                margin-top: 4px;
            }
        }
    }

    button {
        background-color: rgb(98, 77, 212);
        border-radius: 4px;

        &:disabled {
            background-color: rgb(73, 53, 184);
            color: rgb(116, 114, 114);
        }
    }

    .query-editor-info {
        width: 100%;
        align-items: center;
        justify-content: space-between;
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 1em;

        .exit-builder-mode {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            margin-left: 1em;
            font-weight: normal;
            /* background-color: rgba(240, 248, 255, 0.088); */
            border: 1px solid rgba(255, 255, 255, 0.2);
            padding: 0.3em 0.5em;
            border-radius: 4px;
            color: rgba(255, 255, 255, 0.811);
            white-space: nowrap;
            p {
                margin: 0;
            }
            &:active {
                opacity: 0.8;
            }
            &:hover {
                background-color: rgba(240, 248, 255, 0.088);
            }
        }

        .smart-query-actions {
            display: flex;
            align-items: center;
            flex-direction: row;
            justify-content: flex-end;
        }

        input {
            background-color: transparent;
            outline: none;
            border: none;
            margin-left: 10px;
            border: 1px solid rgba(255, 255, 255, 0.36);
            border-radius: 4px;
            padding: 0 0.5em;
            font-size: 1em;
            line-height: 1.8rem;
            width: fit-content;
            min-width: 100px;
        }

        img {
            width: 40px;
            position: relative;
            opacity: 0.2;
        }

        > :nth-child(3) {
            display: flex;
            justify-content: flex-end;
        }
    }
</style>
