<script lang="ts">
    import isDarkColor from "is-dark-color";
    import { currentThemeObject } from "../../../theming/store";

    export let isDisabled = false;
    export let isDestructive = false;
    export let value = "";
    export let description = "";
    export let confirmText = "";
    export let isConfirming = false;
    export let isHighlighted = false;
    export let onClick = null;
    export let color: string | null = null;
    export let onEnterPressed;
    export let onEscPressed;
    export let autoFocus = false;
    export let placeholder = "";
    export let autoCompleteValue = "";
    export let small = false;
    import { createEventDispatcher } from "svelte";
    import Input from "../Input.svelte";
    const dispatch = createEventDispatcher();
</script>

<container
    class:disabled={isDisabled}
    class:destructive={isDestructive}
    class:confirming={isConfirming}
    class:highlighted={isHighlighted}
    class:with-bg={color !== null && color !== undefined}
    style={isHighlighted
        ? color
            ? `border: 1px solid ${
                  isDarkColor(color) ? "white" : color
              }e7;background-color: transparent;`
            : `background-color: ${$currentThemeObject["menu-item-highlight-bg"]};`
        : ""}
    on:click|stopPropagation={() => {
        onClick && onClick();
    }}
>
    <div class="bg" style={color ? `background-color: ${color}` : ""} />
    {#if isConfirming}
        {confirmText}
    {:else if value !== null}
        <span>
            <Input
                bind:value
                disabled={isDisabled}
                {onEnterPressed}
                {onEscPressed}
                fullWidth
                {autoFocus}
                {placeholder}
                {autoCompleteValue}
                {small}
            />
            {#if description}
                <small>
                    {description}
                </small>
            {/if}
        </span>
    {:else}
        <slot />
    {/if}
</container>

<style lang="scss">
    .bg {
        position: absolute;
        top: 2px;
        right: 2px;
        width: 5px;
        bottom: 2px;
        border-radius: 2px;
    }
    container {
        box-sizing: border-box;
        border: 1px solid transparent;
        position: relative;
        padding: 0px 5px;
        cursor: default;
        font-size: 13.5px;
        display: flex;
        align-items: center;
        grid-gap: 5px;
        color: var(--menu-item-text);
        border-radius: 2.5px;

        &:hover.with-bg,
        &.highlighted.with-bg {
            background: transparent;
            transform: scale(1.1);
            border: 1px solid var(--menu-item-text);
            color: var(--menu-item-text);
        }
        &:not(.with-bg) {
            transform: scale(1);
        }

        &.disabled {
            color: var(--menu-item-disabled);
            &:hover {
                background: transparent;
            }
        }
        &.confirming {
            background: var(--menu-item-confirm-bg);
            &:hover {
                background: var(--menu-item-confirm-bg) !important;
            }
        }
        &.destructive {
            &:hover {
                background: var(--menu-item-destructive-hover-bg);
                color: var(--menu-item-destructive-hover-text);
            }
        }

        span {
            display: block;
            padding: 0.2em 0 0.3em 0;
            z-index: 1;
            width: 100%;

            input {
                padding: 0.2em 0;
                font-size: 14px;
                outline: none;
                background: none;
                border: none;
                white-space: nowrap;
                margin: 0;
                line-height: 20px;
                overflow: hidden;
                width: 100%;
                text-overflow: ellipsis;
            }

            small {
                margin: 0;
                line-height: 14px;
                display: block;
                opacity: 0.7;
            }
        }
    }
</style>
