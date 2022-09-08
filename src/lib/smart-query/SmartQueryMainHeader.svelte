<script lang="ts">
    import SmartQuery from "../../data/SmartQueries";

    import {
        isSmartQueryBuilderOpen,
        isSmartQueryUiOpen,
        isSmartQueryValid,
        selectedSmartQuery
    } from "../../data/store";

    function hideSmartQuery() {
        $isSmartQueryUiOpen = false;
    }

    function showSmartQueryBuilder() {
        $isSmartQueryBuilderOpen = true;
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
        <div class="validation">
            {#if $isSmartQueryValid}
                <p>query is valid</p>
                <iconify-icon class="valid" icon="charm:tick" />
            {:else}
                <p>query is not valid</p>
                <iconify-icon
                    class="invalid"
                    icon="ant-design:warning-outlined"
                />
            {/if}
        </div>

        <img src="images/arrow-down-right.svg" />
        <div>
            <button>Save smart query</button>
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
    }

    .query-editor-info {
        width: 100%;
        align-items: center;
        justify-content: space-between;
        display: grid;
        grid-template-columns: 1fr auto auto;
        gap: 1em;

        .validation {
            margin-left: 2em;
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 0.3em;
            font-weight: normal;

            p {
                font-size: 13px;
                opacity: 0.7;
            }
            .valid {
                color: green;
            }

            .invalid {
                color: orange;
            }
        }

        img {
            height: 50px;
            position: relative;
            top: -20px;
            opacity: 0.2;
        }

        > :nth-child(3) {
            display: flex;
            justify-content: flex-end;
        }
    }
</style>
