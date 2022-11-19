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
        line-height: 0.9em;
        padding: 0.3em;
        background-color: rgba(53, 48, 48, 0.349);
        border: 1px solid rgb(81, 76, 76);
        border-radius: 2px;
        font-size: 14px;
        z-index: 1;
        &:focus {
            background-color: rgba(102, 92, 92, 0.104);
        }
        &.full-width {
            width: 100%;
        }
        &.minimal {
            background: none;
            border: none;
            outline: none;
        }
        &::placeholder {
            opacity: 0.5;
        }
    }

    .autocomplete {
        position: absolute;
        pointer-events: none;
        left: 6px;
        right: 0px;
        bottom: 0px;
        top: 2px;
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
