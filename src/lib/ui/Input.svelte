<script>
    import { onMount } from "svelte";

    export let value;
    export let onChange = (evt) => {};
    export let fullWidth = false;
    export let autoCompleteValue = null;
    export let tabBehavesAsEnter = false;
    export let onEnterPressed = null;
    export let onBackspacePressed = null;
    export let onEscPressed = null;
    export let autoFocus = false;
    export let placeholder = "";
    export let minimal = false;
    export let small = false;
    export let alt = false;

    function onKeyDown(evt) {
        if (evt.keyCode === 13) {
            if (onEnterPressed) {
                evt.preventDefault();
                onEnterPressed();
            }
        } else if (evt.keyCode === 9) {
            // Tab
            if (autoCompleteValue?.length) {
                evt.preventDefault();
                if (tabBehavesAsEnter) {
                    onEnterPressed && onEnterPressed();
                } else {
                    value = autoCompleteValue;
                }
            }
        } else if (evt.keyCode === 27) {
            if (autoCompleteValue?.length) {
                // Esc
                evt.preventDefault();
                onEscPressed && onEscPressed();
            }
        } else if (evt.keyCode === 8) {
            if (value?.length === 0) {
                // Backspace on empty
                evt.preventDefault();
                onBackspacePressed && onBackspacePressed();
            }
        }
    }

    let inputField;
    onMount(() => {
        if (autoFocus) inputField.focus();
    });
</script>

<div>
    <input
        bind:this={inputField}
        bind:value
        {placeholder}
        on:input={onChange}
        class:full-width={fullWidth}
        class:minimal
        class:small
        class:alt
        on:keydown={onKeyDown}
        spellcheck="false"
        autocomplete="off"
    />

    {#if autoCompleteValue}
        <p class="autocomplete">{autoCompleteValue}</p>
    {/if}
</div>

<style lang="scss">
    div {
        position: relative;
    }
    input {
        line-height: inherit;
        padding: 0.3em;
        background-color: color-mix(in srgb, var(--background) 66%, black);
        border: 1px solid color-mix(in srgb, var(--background) 56%, black);
        border-radius: 2px;
        font-size: 14px;
        z-index: 1;
        color: var(--text);
        &:focus {
            background-color: color-mix(in srgb, var(--background) 60%, black);
        }

        &.alt {
            border-radius: 5px;
            padding: 0.1em 0.3em;
            &:focus {
                outline-style: solid;
                outline-width: 1px;
                outline-offset: 0.5px;
                outline-color: rgb(196, 199, 200);
            }
        }

        &.full-width {
            width: 100%;
        }
        &.minimal {
            background: none;
            border: none;
            outline: none;
        }
        &.small {
            line-height: 0.9em;
        }
        &::placeholder {
            opacity: 0.5;
            color: var(--text);
        }
    }

    .autocomplete {
        position: absolute;
        pointer-events: none;
        left: 6px;
        right: 0px;
        bottom: 0px;
        top: 1px;
        max-width: 100%;
        text-overflow: ellipsis;
        white-space: nowrap;
        overflow: hidden;
        margin: auto 0;
        height: fit-content;
        vertical-align: middle;
        line-height: normal;
        text-align: left;
        user-select: none;
        cursor: default;
        opacity: 0.5;
        z-index: 0;
    }
</style>
