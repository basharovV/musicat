<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { smartQuery } from "../../data/store";
    import { autoWidth } from "../../utils/AutoWidth";
    import type { UserQueryPart } from "./UserQueryPart";

    export let userQueryPart: UserQueryPart;
    let parts = userQueryPart.queryPart.prompt.split(" ");
    export let shouldFocus = true;
    export let onRemove: Function;
    export let onFocus: Function;

    let firstInputField;
    let container: HTMLDivElement;
    let currentFocusedInputIdx = 0;
    let currentFocusedInputPart = null;
    let isFocused = false;

    /**
     * In a query part like "between {startYear} and {endYear}"
     * this function returns the word index corresponding to the input index.
     *
     * eg. the prompt above has 4 words/parts, but only 2 inputs.
     * So the second input is at position 4.
     */
    function getInputIdxFromPos(pos: number) {
        let counter = 0;
        let foundIdx = 0;
        parts.forEach((p, idx) => {
            const isInput = p.startsWith("{") && p.endsWith("}");
            if (isInput) {
                counter++;
                if (counter === pos) {
                    foundIdx = idx;
                }
            }
        });
        return foundIdx;
    }

    function getInputPosFromPartIdx(partIdx: number) {
        let counter = -1;
        parts.forEach((p, idx) => {
            const isInput = p.startsWith("{") && p.endsWith("}");
            if (isInput && idx < partIdx) {
                counter++;
            }
        });
        return counter;
    }

    function onBackspace(partIdx) {
        // Handle empty
        console.log("partIdx", partIdx);
        console.log("getInputIdxFromPos(2)", getInputIdxFromPos(2));
        if (!isFocused) return;
        // If we're on input field 2 or above, focus previous
        if (
            Object.keys(userQueryPart.userInputs).length > 1 &&
            partIdx >= getInputIdxFromPos(2)
        ) {
            console.log("focusing previous");
            let counter = -1;
            parts.forEach((p, idx) => {
                const isInput = p.startsWith("{") && p.endsWith("}");
                if (isInput && idx < partIdx) {
                    counter++;
                }
            });
            const foundPrevInput =
                container.getElementsByTagName("input")[counter];
            console.log("prevInput", foundPrevInput);
            foundPrevInput?.focus();
        }
        // Otherwise delete whole entry
        else {
            console.log("removing current");
            onRemove && onRemove();
        }
    }

    function onInput() {
        $smartQuery.validate();
    }

    function onInputFocus(idx, part) {
        currentFocusedInputIdx = idx;
        currentFocusedInputPart = part;
        isFocused = true;
        console.log("onfocus", idx, part);
        onFocus && onFocus();
    }

    function onLostFocus() {
        console.log("uncofusing", currentFocusedInputPart);
        currentFocusedInputIdx = 0;
        currentFocusedInputPart = null;
        isFocused = false;
    }

    function onKeyPressed(event) {
        if (event.keyCode === 8 && currentFocusedInputPart && isFocused) {
            console.log("currently focused", currentFocusedInputPart);
            const userInputVal =
                userQueryPart.userInputs[currentFocusedInputPart].value;
            if (!userInputVal || userInputVal.toString().length === 0) {
                event.preventDefault();
                onBackspace(currentFocusedInputIdx);
            }
        }
    }

    addEventListener("keydown", onKeyPressed);

    onDestroy(() => {
        removeEventListener("keydown", onKeyPressed);
    });

    onMount(() => {
        if (shouldFocus) {
            const foundFirstInput = container.getElementsByTagName("input")[0];
            if (foundFirstInput) {
                foundFirstInput.focus();
            }
        }
    });
</script>

<div bind:this={container} class:focused={isFocused}>
    {#each parts as part, idx (part)}
        {#if part.startsWith("{") && part.endsWith("}")}
            <!-- svelte-ignore a11y-autofocus -->
            <input
                class={`is-${
                    userQueryPart.userInputs[
                        part.slice(1, part.length - 1) // Remove { } around the key
                    ].type
                }`}
                use:autoWidth
                bind:value={userQueryPart.userInputs[
                    part.slice(1, part.length - 1)
                ].value}
                on:input={onInput}
                on:focus={() =>
                    onInputFocus(idx, part.slice(1, part.length - 1))}
                on:blur={onLostFocus}
            />
        {:else}
            <p>{part}</p>
        {/if}
    {/each}
</div>

<style lang="scss">
    div {
        background-color: var(--smart-playlist-builder-block-bg);
        border-radius: 4px;
        display: flex;
        flex-direction: row;
        gap: 5px;
        align-items: center;
        padding: 0em 0.5em;
        height: 30px;
        /* transition: all 0.2s cubic-bezier(0.075, 0.82, 0.165, 1); */

        &.focused {
            background-color: var(--smart-playlist-builder-block-focused-bg);
            border: 1px solid rgba(255, 255, 255, 0.089);
            transform: scale(1.04);
        }
    }
    p {
        font-size: 1em;
        margin: 0;
        font-weight: 400;
        line-height: 14px;
        color: var(--smart-playlist-builder-block-text);
    }

    input {
        background-color: var(--smart-playlist-builder-block-input-bg);
        /* background-color: #45306A; */
        border: 1px solid #d4ccdd6c;
        /* box-shadow: 0px 0px 1px 4px #3e286420 inset; */
        border-radius: 4px;
        padding: 0.2em;
        font-size: 14px;
        margin: 0;
        outline: none;
        transition: min-width 0.2s ease-in;
        color: var(--smart-playlist-builder-block-input-text);

        &.is-number {
            min-width: 30px;
        }

        &.is-string {
            min-width: 50px;
        }

        &:hover {
        }

        &:focus {
            border: 1px solid white;
        }
    }
</style>
