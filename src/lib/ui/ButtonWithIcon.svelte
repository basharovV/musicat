<script lang="ts">
    import Icon from "./Icon.svelte";

    export let icon = null;
    export let onClick;
    export let text;
    export let theme: "solid" | "translucent" | "transparent" = "solid";
    export let size: "small" | "medium" = "medium";
    export let isLoading = false;
    export let disabled = false;
</script>

<div
    on:click={onClick}
    class="theme-{theme} {size}"
    class:disabled
    role="button"
    tabindex="0"
>
    {#if icon}
        <Icon icon={isLoading ? "line-md:loading-loop" : icon} />
    {/if}
    <p>{text}</p>
</div>

<style lang="scss">
    div {
        width: fit-content;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 5px;
        /* margin-left: 1em; */
        font-weight: normal;
        /* background-color: rgba(240, 248, 255, 0.088); */
        border-radius: 6px;
        color: var(--button-text);
        white-space: nowrap;
        cursor: default;
        transition: border-color 0.1s;
        user-select: none;
        p {
            margin: 0;
            line-height: normal;
        }

        &.disabled {
            opacity: 0.5;
        }
        &:active {
            opacity: 0.8;
        }
        &:hover {
            background-color: rgba(240, 248, 255, 0.088);
        }

        &:focus,
        &:focus-visible {
            outline: 4px auto -webkit-focus-ring-color;
        }

        // Themes
        &.theme-solid {
            border: 1px solid transparent;
            font-size: 1em;
            font-weight: 500;
            font-family: inherit;
            background-color: var(--button-bg);
            white-space: nowrap;
            &:hover {
                border-color: var(--accent);
            }
        }

        &.theme-translucent {
            background-color: color-mix(
                in srgb,
                var(--inverse) 40%,
                transparent
            );
            border: 1px solid
                color-mix(in srgb, var(--inverse) 80%, transparent);
            &:hover {
                border-color: var(--accent);
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 90%,
                    transparent
                );
            }
        }
        &.theme-transparent {
            border: 1px solid rgba(255, 255, 255, 0.2);
            color: var(--text);
        }

        // Sizes
        &.small {
            padding: 0.3em 0.75em;
        }

        &.medium {
            padding: 0.6em 1.2em;
        }
    }
</style>
