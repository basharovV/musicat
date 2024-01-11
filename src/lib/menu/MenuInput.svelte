<script lang="ts">
    import isDarkColor from "is-dark-color";

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
    import { createEventDispatcher } from "svelte";
    import Input from "../ui/Input.svelte";
    const dispatch = createEventDispatcher();

    $: {
        console.log("highlighted", isHighlighted);
    }
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
            : "background-color: #c6c8ca;"
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
                {onEnterPressed}
                {onEscPressed}
                fullWidth
                {autoFocus}
                {placeholder}
                {autoCompleteValue}
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
        padding: 0px 10px;
        cursor: default;
        font-size: 13.5px;
        display: flex;
        align-items: center;
        grid-gap: 5px;
        color: rgb(255, 255, 255);
        border-radius: 2.5px;

        &:hover.with-bg,
        &.highlighted.with-bg {
            background: transparent;
            transform: scale(1.1);
            border: 1px solid white;
            color: white;
        }
        &:not(.with-bg) {
            transform: scale(1);
        }

        &:hover,
        &.highlighted {
            /* color: rgb(37, 36, 36); */
        }
        &.disabled {
            color: rgba(174, 174, 174, 0.4);
            &:hover {
                background: transparent;
            }
        }
        &.confirming {
            background: #d2630e;
            &:hover {
                background: #d2630e !important;
            }
        }
        &.destructive {
            &:hover {
                background: #d20e32;
                color: white;
            }
        }

        span {
            display: block;
            padding: 0.2em 0 0.3em 0;
            z-index: 1;

            input {
                padding: 0.2em 0;
                font-size: 14px;
                outline: none;
                background: none;
                border: none;
                white-space: nowrap;
                margin: 0;
                line-height: 20px;
                max-width: 250px;
                overflow: hidden;
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
