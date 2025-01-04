<script lang="ts">
    import { liveQuery } from "dexie";
    import SmartQuery from "../../data/SmartQueries";
    import { db } from "../../data/db";
    import Icon from "../ui/Icon.svelte";

    import {
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        selectedSmartQuery,
        smartQueryInitiator,
        uiView,
    } from "../../data/store";

    $: savedSmartQueries = liveQuery(async () => {
        return db.smartQueries.toArray();
    });

    function hideSmartQuery() {
        $uiView = "library";
    }

    function hideSmartQueryBuilder() {
        if ($smartQueryInitiator === "library-cell") {
            $uiView = "library";
        }
        $isSmartQueryBuilderOpen = false;
    }

    function showSmartQueryBuilder() {
        $isSmartQueryBuilderOpen = true;
        $isSmartQuerySaveUiOpen = false;
    }
</script>

<div class="query-browser">
    <Icon
        icon="mingcute:close-circle-fill"
        size={14}
        onClick={hideSmartQuery}
    />
    <div class="query-header">
        <p class="query-header-title">Smart playlist:</p>
        <select bind:value={$selectedSmartQuery}>
            {#each Object.values(SmartQuery) as query}
                <option value={query.value}>{query.name}</option>
            {/each}
            <option value="----">----</option>

            {#if $savedSmartQueries}
                {#each $savedSmartQueries as query}
                    <option value={`~usq:${query.name}`}>{query.name}</option>
                {/each}
            {/if}
        </select>
    </div>
</div>

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
        padding: 0.5em;
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
                color: rgb(233, 226, 226);
                /* background-color: rgb(220, 208, 237); */
                outline: none;
                opacity: 0.9;
                margin-top: 4px;
            }
        }
    }

    select > option {
        color: white;
    }

    .query-editor-info {
        width: 100%;
        align-items: center;
        justify-content: space-between;
        display: grid;
        grid-template-columns: 1fr;
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
