<script lang="ts">
    import { UserQueryPart } from "./UserQueryPart";

    import { onMount } from "svelte";
    import {
        forceRefreshLibrary,
        isSmartQueryBuilderOpen,
        isSmartQuerySaveUiOpen,
        isSmartQueryValid,
        selectedSmartQuery,
        smartQuery,
        smartQueryInitiator,
        uiView
    } from "../../data/store";
    import { autoWidth } from "../../utils/AutoWidth";
    import Menu from "../menu/Menu.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import type { QueryPartStruct } from "./QueryPart";
    import { BUILT_IN_QUERY_PARTS } from "./QueryParts";
    import SmartQueryPart from "./SmartQueryPart.svelte";
    import { fly } from "svelte/transition";
    import { cubicInOut } from "svelte/easing";

    const fields = ["artist"];

    const keys = ["artist", "album", "year", "genre", "duration", "title"];

    let matchingQueryParts: QueryPartStruct[] = [];

    let queryInput: HTMLInputElement;
    let isAutocompleteHidden = true;
    let isAutofocus = false;

    function onFocus() {
        console.log("smartQueryInitiator", $smartQueryInitiator);

        setTimeout(() => {
            if ($smartQuery.userInput.trim().length === 0) {
                // Show all options. Later can cap it if there's too many.

                console.log("empty input");
                updateQueryPartsAutocompletePos();
                matchingQueryParts = BUILT_IN_QUERY_PARTS;
                console.log("matching parts", matchingQueryParts);
            }
        }, 150);

        isAutofocus = false;
    }

    function onLostFocus() {
        matchingQueryParts = [];
    }

    onMount(() => {
        setTimeout(() => {
            isAutofocus = true;
            if ($smartQueryInitiator !== "library-cell") {
                queryInput.focus();
            }
        }, 150);
    });

    function onInput() {
        if ($smartQuery.userInput.trim().length > 0) {
            updateQueryPartsAutocompletePos();
            console.log("in here");
            matchingQueryParts = [];
            // Check each word
            $smartQuery.userInput
                .trim()
                .split(" ")
                .forEach((word) => {
                    if (word.length === 0) {
                        return;
                    }
                    // Check if matches keys
                    const matchesKeys = keys.reduce((matched, key) => {
                        if (key.toLowerCase().includes(word.toLowerCase())) {
                            matched.push(key);
                        }
                        return matched;
                    }, []);

                    const matchedQueryParts = BUILT_IN_QUERY_PARTS.reduce(
                        (matched, part) => {
                            if (
                                part.fieldKey
                                    .toLowerCase()
                                    .includes(word.toLowerCase()) ||
                                part.prompt
                                    .toLowerCase()
                                    .includes(word.toLowerCase()) ||
                                part.name
                                    .toLowerCase()
                                    .includes(word.toLowerCase())
                            ) {
                                matched.push(part);
                            }
                            console.log("matched ", matched);

                            return matched;
                        },
                        []
                    );

                    matchingQueryParts = matchedQueryParts;
                });
        } else {
            matchingQueryParts = [];
        }
    }

    const onSelectPart = (part: QueryPartStruct) => {
        console.log("adding part", part);
        matchingQueryParts = [];
        $smartQuery.userInput = "";
        $smartQuery.addPart(new UserQueryPart(part));
        $smartQuery.parts = $smartQuery.parts;
    };

    const onRemovePart = (partIdx: number) => {
        console.log("removing part " + partIdx);
        $smartQuery.removePart(partIdx);
        $smartQuery.parts = $smartQuery.parts;

        setTimeout(() => {
            queryInput.focus();
        }, 150);
    };

    function closeAutoComplete() {
        matchingQueryParts = [];
    }

    let inputX: number;
    let inputY: number;

    function updateQueryPartsAutocompletePos() {
        inputX = queryInput.offsetLeft;
        inputY = queryInput.offsetTop + 20;
        console.log("X", inputX, "Y", inputY);
    }

    let hoveredItemIdx = 0;
    let numberOfItems = 0;

    function save() {
        $smartQuery.save();
        // Close the builder UI and set the current selected query to the one we just saved
        $isSmartQueryBuilderOpen = false;
        $selectedSmartQuery = `~usq:${$smartQuery.name}`;
        $smartQuery.reset();
    }
</script>

<container>
    <div class="builder">
        <div class="query-parts">
            {#each $smartQuery.parts as queryPart, idx (queryPart.queryPart.name + idx)}
                <!-- <div>{queryPart.toString()}</div> -->
                <SmartQueryPart
                    userQueryPart={queryPart}
                    onRemove={() => onRemovePart(idx)}
                    onFocus={onLostFocus}
                />
                <p>,</p>
            {/each}

            <span>
                <input
                    type="text"
                    bind:this={queryInput}
                    bind:value={$smartQuery.userInput}
                    on:focus={onFocus}
                    on:input={onInput}
                    use:autoWidth
                    autocomplete="off"
                    spellcheck="false"
                    placeholder="add new block"
                /></span
            >
        </div>
        <div class="options">
            <div class="validation">
                {#if $isSmartQueryValid}
                    <p>query is valid</p>
                    <Icon color="green" icon="charm:tick" />
                {:else}
                    <p>query is not valid</p>
                    <Icon color="orange" icon="ant-design:warning-outlined" />
                {/if}
            </div>
            <div class="save">
                <ButtonWithIcon
                    icon="material-symbols:save-outline"
                    onClick={() => {
                        $isSmartQuerySaveUiOpen = !$isSmartQuerySaveUiOpen;
                    }}
                    text="Save"
                    theme="transparent"
                />
            </div>
            <div class="close">
                <ButtonWithIcon
                    icon="material-symbols:close"
                    onClick={() => {
                        if ($smartQueryInitiator === "library-cell") {
                            $forceRefreshLibrary = true;
                            $isSmartQueryBuilderOpen = false;
                            $uiView = "library";
                        } else {
                            $isSmartQueryBuilderOpen = false;
                            // $uiView = "smart-query";
                        }
                    }}
                    text="Close editor"
                    theme="transparent"
                />
            </div>
        </div>
        {#if matchingQueryParts.length > 0}
            <Menu
                x={inputX}
                y={inputY}
                onClickOutside={closeAutoComplete}
                items={matchingQueryParts.map((p) => ({
                    text: p.description,
                    description: "eg. " + p.example,
                    source: p
                }))}
                onItemSelected={onSelectPart}
            />
        {/if}
    </div>
    {#if $isSmartQuerySaveUiOpen}
        <!-- <ButtonWithIcon
        icon="mingcute:close-circle-fill"
        onClick={hideSmartQueryBuilder}
        text="Hide builder"
    /> -->

        <div
            class="smart-query-actions"
            transition:fly={{
                y: -10,
                duration: 200,
                easing: cubicInOut
            }}
        >
            <p>Name:</p>
            <form on:submit|preventDefault={save}>
                <input bind:value={$smartQuery.name} />
            </form>
            <img src="images/arrow-down-right.svg" />
            <button
                disabled={!$isSmartQueryValid || !$smartQuery.isNameSet}
                on:click={save}>Save smart query</button
            >
        </div>
    {/if}
</container>

<style lang="scss">
    container {
        display: flex;
        flex-direction: column;
    }
    .builder {
        padding: 0.5em 0.5em 0.5em 0;
        display: grid;
        grid-template-columns: 1fr auto;
        overflow: visible;
        position: relative;
        border-bottom: 1px solid rgba(255, 255, 255, 0.16);
    }

    .smart-query-actions {
        display: flex;
        align-items: center;
        flex-direction: row;
        justify-content: flex-end;
        padding-right: 0.5em;

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
            width: 30px;
            margin: 0 5px;
            position: relative;
            opacity: 0.2;
        }

        button {
            background-color: rgb(98, 77, 212);
            border-radius: 4px;

            &:disabled {
                background-color: rgb(73, 53, 184);
                color: rgb(116, 114, 114);
            }
        }
    }

    .query-parts {
        flex-grow: 1;
        display: flex;
        flex-direction: row;
        gap: 0.5em 0.2em;
        margin-right: 1em;
        margin-left: 1em;
        margin-top: 0.3em;
        flex-wrap: wrap;
        max-width: 90%;
        > p {
            margin-bottom: 0;
            line-height: 20px;
            vertical-align: bottom;
            opacity: 0.7;
        }
    }
    input {
        background-color: transparent;
        outline: none;
        border: none;
        border-bottom: 1px solid rgba(255, 255, 255, 0.56);
        border-top-left-radius: 2px;
        border-top-right-radius: 2px;
        font-size: 1em;
        line-height: 1.8rem;
        width: fit-content;
        min-width: 100px;
    }

    .validation {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 0.3em;
        font-weight: normal;

        p {
            font-size: 13px;
            opacity: 0.7;
            margin: 0;
        }
        .valid {
            color: green;
        }

        .invalid {
            color: orange;
        }
    }

    .autocomplete {
        position: fixed;
        top: 4em;
        padding: 1em;
        backdrop-filter: blur(10px);
        z-index: 10;
        border-radius: 4px;
        border: 1px solid rgba(255, 255, 255, 0.23);
        box-shadow: 2px 2px 5px 0px #0002;
        background-color: #20202296;

        > div {
            &:hover {
                background: #c6c8ca;
                color: rgb(79, 76, 76);
            }
        }
    }

    .options {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 0.5em;
    }
    .save {
        display: flex;
    }
</style>
