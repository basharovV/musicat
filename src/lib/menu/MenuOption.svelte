<script lang="ts">
    import isDarkColor from "is-dark-color";

    export let isDisabled = false;
    export let isDestructive = false;
    export let text = "";
    export let description = "";
    export let confirmText = "";
    export let isConfirming = false;
    export let isHighlighted = false;
    export let onClick = null;
    export let onDelete = null;
    export let color: string | null = null;
    export let borderHighlight = false;
    export let checked = null;
    export let isLoading = false;

    import { createEventDispatcher } from "svelte";
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
    class:border-highlight={borderHighlight}
    class:with-bg={color !== null && color !== undefined}
    class:loading={isLoading}
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
        <span><p>{confirmText}</p></span>
    {:else if text}
        <span>
            <p>{text}</p>
            {#if description}
                <small>
                    {description}
                </small>
            {/if}
        </span>

        {#if isLoading}
            <iconify-icon
                icon="line-md:loading-loop"
                class:loading={isLoading}
            />
        {/if}

        {#if checked !== null}
            <iconify-icon icon="charm:tick" class:checked />
        {/if}

        {#if onDelete}
            <iconify-icon
                icon="mingcute:close-circle-fill"
                on:click={onDelete}
            />
        {/if}
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
            color: rgb(37, 36, 36);
            background-color: rgba(255, 255, 255, 0.796);
            &.border-highlight {
                color: white;
                background-color: rgba(255, 255, 255, 0.196);

                border: 1px solid rgba(255, 255, 255, 0.1);
            }
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

        &.loading {
            animation: loading 2s ease-in-out infinite alternate-reverse;
        }

        span {
            display: block;
            padding: 0.2em 0 0.3em 0;
            z-index: 1;
            width: 100%;
            p {
                white-space: nowrap;
                margin: 0;
                text-align: left;
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
                text-align: left;
            }
        }

        iconify-icon[icon="charm:tick"] {
            color: grey;

            &.checked {
                color: white;
            }
        }
    }

    @keyframes loading {
        from {
            background-color: inherit;
        }
        to {
            background-color: #dba4fac1;
        }
    }
</style>
