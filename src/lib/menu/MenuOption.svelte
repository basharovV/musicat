<script lang="ts">
    import isDarkColor from "is-dark-color";
    import Icon from "../ui/Icon.svelte";
    import { currentThemeObject } from "../../theming/store";

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
    export let singleSelection = false;

    $: {
        console.log("highlighted", isHighlighted);
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<container
    class:disabled={isDisabled}
    class:destructive={isDestructive}
    class:confirming={isConfirming}
    class:highlighted={isHighlighted}
    class:border-highlight={borderHighlight}
    class:with-bg={color !== null && color !== undefined}
    class:single-selection={singleSelection}
    class:loading={isLoading}
    style={isHighlighted
        ? color
            ? `border: 1px solid ${
                  isDarkColor(color) ? "white" : color
              }e7;background-color: transparent;`
            : `background-color: ${$currentThemeObject["menu-item-highlight-bg"]};`
        : ""}
    on:click|stopPropagation={(e) => {
        !isDisabled && onClick && onClick();
    }}
>
    <div class="bg" style={color ? `background-color: ${color}` : ""} />
    {#if text}
        <span>
            <p>{@html isConfirming ? confirmText : text}</p>
            {#if description}
                <small>
                    {description}
                </small>
            {/if}
        </span>

        {#if isLoading}
            <div class:loading={isLoading}>
                <Icon icon="line-md:loading-loop" />
            </div>
        {/if}

        {#if checked !== null}
            <Icon
                icon="charm:tick"
                size={12}
                color={checked
                    ? $currentThemeObject["menu-checkbox-on"]
                    : singleSelection
                      ? "transparent"
                      : $currentThemeObject["menu-checkbox-off"]}
            />
        {/if}

        {#if onDelete}
            <Icon icon="mingcute:close-circle-fill" onClick={onDelete} />
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
        color: var(--menu-item-text);
        border-radius: 2.5px;
        user-select: none;

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

        &:hover,
        &.highlighted {
            color: var(--menu-item-highlight-text);
            background-color: var(--menu-item-highlight-bg);
            border: 1px solid var(--menu-item-highlight-border);
            &.border-highlight {
                color: var(--menu-item-highlight-secondary-text);
                background-color: var(--menu-item-highlight-secondary-bg);
                border: 1px solid var(--menu-item-highlight-secondary-border);
            }
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
            &:hover:not(.disabled) {
                background: var(--menu-item-destructive-hover-bg);
                color: var(--menu-item-destructive-hover-text);
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

            :global(u) {
                text-underline-offset: 3px;
            }

            :global(i) {
                font-size: 12.5px;
            }
        }
    }

    @keyframes loading {
        from {
            background-color: var(--menu-item-loading-from);
        }
        to {
            background-color: var(--menu-item-loading-to);
        }
    }
</style>
