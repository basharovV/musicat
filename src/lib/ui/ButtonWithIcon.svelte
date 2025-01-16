<script lang="ts">
    import LL from "../../i18n/i18n-svelte";
    import Icon from "./Icon.svelte";

    export let icon = null;
    export let onClick;
    export let text = "";
    export let theme: "active" | "solid" | "translucent" | "transparent" =
        "solid";
    export let size: "small" | "medium" = "medium";
    export let isLoading = false;
    export let disabled = false;
    export let fill = false;
    export let confirmText = null;
    export let isDestructive = false;

    let isConfirmingAction = false;
</script>

<div
    on:click={() => {
        if (disabled) {
            return;
        } else if (isDestructive && !isConfirmingAction) {
            isConfirmingAction = true;
        } else {
            onClick();
            isConfirmingAction = false;
        }
    }}
    class="theme-{theme} {size}"
    class:disabled
    class:fill
    class:no-text={text.length === 0}
    role="button"
    tabindex="0"
>
    {#if icon}
        <Icon icon={isLoading ? "line-md:loading-loop" : icon} />
    {/if}
    {#if isDestructive && isConfirmingAction}
        <p>{confirmText || $LL.button.areYouSure()}</p>
    {:else if text.length}
        <p>{text}</p>
    {/if}
</div>

<style lang="scss">
    div {
        width: fit-content;
        display: flex;
        flex-direction: row;
        height: fit-content;
        align-items: center;
        gap: 5px;
        font-weight: normal;
        border-radius: 6px;
        color: var(--button-text);
        white-space: nowrap;
        cursor: pointer;
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
        &:hover:not(.disabled) {
            background-color: rgba(240, 248, 255, 0.088);
        }
        &:focus,
        &:focus-visible {
            outline: 4px auto -webkit-focus-ring-color;
        }

        // Themes
        &.theme-active {
            font-size: 1em;
            font-weight: 500;
            font-family: inherit;
            background-color: var(--smart-playlist-button-bg);
            white-space: nowrap;

            &:hover {
                background-color: var(--smart-playlist-button-disabled-bg);
            }

            &:disabled {
                background-color: var(--smart-playlist-button-disabled-bg);
                color: var(--smart-playlist-button-disabled);
            }
        }

        &.theme-solid {
            border: 1px solid transparent;
            font-size: 1em;
            font-weight: 500;
            font-family: inherit;
            background-color: var(--button-bg);
            white-space: nowrap;
            &:hover:not(.disabled) {
                border-color: var(--accent);
                background-color: color-mix(
                    in srgb,
                    var(--button-bg) 80%,
                    transparent
                );
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
            &:hover:not(.disabled) {
                border-color: var(--accent);
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 90%,
                    transparent
                );
            }
        }

        &.theme-transparent {
            border: 1px solid
                color-mix(in srgb, var(--inverse) 10%, transparent);
            color: var(--text);

            &:hover:not(.disabled) {
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 15%,
                    transparent
                );
            }

            &.fill {
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 40%,
                    transparent
                );
            }
        }

        // Sizes
        &.small {
            padding: 0.3em 0.75em;
        }

        &.medium {
            padding: 0.6em 1.2em;
        }

        &.no-text {
            &.small {
                padding: 0.3em;
            }

            &.medium {
                padding: 0.6em;
            }
        }
    }
</style>
