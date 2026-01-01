<script lang="ts">
    import LL from "../../i18n/i18n-svelte";
    import Icon from "./Icon.svelte";

    export let icon = null;
    export let iconSize = 20;
    export let onClick: (value?: string) => void; // Updated type to handle option values
    export let text = "";
    export let theme: "active" | "solid" | "translucent" | "transparent" =
        "solid";
    export let size: "small" | "medium" = "medium";
    export let isLoading = false;
    export let disabled = false;
    export let confirmText = null;
    export let isDestructive = false;
    export let noOutline = false;

    /**
     * If options exist, this becomes a segmented button
     */
    export let options: { value: string; icon?: string; label?: string }[] = [];
    export let value: string = ""; // Track current selected value for segmented mode

    let isConfirmingAction = false;

    function handleButtonClick(optionValue?: string) {
        if (disabled) return;

        if (isDestructive && !isConfirmingAction) {
            isConfirmingAction = true;
        } else {
            onClick(optionValue);
            isConfirmingAction = false;
        }
    }
</script>

<div class="button-container" class:segmented={options.length > 0}>
    {#if options.length > 0}
        {#each options as option}
            <div
                on:click={() => handleButtonClick(option.value)}
                class="button theme-{theme} {size}"
                class:active={value === option.value}
                class:disabled
                role="button"
                tabindex="0"
            >
                {#if option.icon}
                    <Icon icon={option.icon} size={iconSize} />
                {/if}
                {#if option.label}
                    <p>{option.label}</p>
                {/if}
            </div>
        {/each}
    {:else}
        <div
            on:click={() => handleButtonClick()}
            class="button theme-{theme} {size}"
            class:disabled
            class:no-outline={noOutline}
            role="button"
            class:icon-only={!text.length}
            tabindex="0"
        >
            {#if icon}
                <Icon
                    icon={isLoading ? "line-md:loading-loop" : icon}
                    size={iconSize}
                />
            {/if}
            {#if isDestructive && isConfirmingAction}
                <p>{confirmText || $LL.button.areYouSure()}</p>
            {:else if text.length}
                <p>{text}</p>
            {/if}
        </div>
    {/if}
</div>

<style lang="scss">
    .button-container {
        display: flex;
        width: fit-content;

        &.segmented {
            // Match the border-color of the theme if necessary

            .button {
                border-radius: 0;
                &:first-child {
                    border-top-left-radius: 6px;
                    border-bottom-left-radius: 6px;
                }
                &:last-child {
                    border-top-right-radius: 6px;
                    border-bottom-right-radius: 6px;
                }

                &.active {
                    background-color: var(--button-active-bg) !important;
                }
            }
        }
    }

    .button {
        width: fit-content;
        display: flex;
        flex-direction: row;
        height: fit-content;
        align-items: center;
        gap: 5px;
        font-weight: normal;
        border-radius: 6px;
        white-space: nowrap;
        cursor: pointer;
        transition: all 0.1s ease;
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
        &:not(:no-outline) {
            &:focus,
            &:focus-visible {
                outline: 4px auto -webkit-focus-ring-color;
            }
        }

        // Themes
        &.theme-active {
            font-size: 1em;
            font-weight: 500;
            font-family: inherit;
            background-color: var(--button-active-bg);
            color: var(--button-text);
            white-space: nowrap;
            border-color: transparent;

            &:hover:not(.disabled) {
                background-color: color-mix(
                    in srgb,
                    var(--button-active-bg) 80%,
                    transparent
                );
            }

            &:disabled {
                background-color: var(--button-active-disabled-bg);
                color: var(--button-active-disabled-text);

                &:hover {
                    background-color: var(--button-active-disabled-bg);
                }
            }
        }

        &.theme-solid {
            border: 1px solid transparent;
            font-size: 1em;
            font-weight: 500;
            font-family: inherit;
            background-color: var(--button-solid-bg);
            color: var(--button-text);
            white-space: nowrap;

            &:hover:not(.disabled) {
                border-color: var(--accent);
                background-color: color-mix(
                    in srgb,
                    var(--button-solid-bg) 80%,
                    transparent
                );
            }
        }

        &.theme-translucent {
            background-color: color-mix(
                in srgb,
                var(--inverse) 30%,
                transparent
            );
            border: 1px solid
                color-mix(in srgb, var(--inverse) 10%, transparent);
            color: var(--button-text);

            &:hover:not(.disabled) {
                border-color: var(--accent);
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 50%,
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
        }

        // Sizes
        &.small {
            padding: 0.3em 0.75em;
            &.icon-only {
                padding: 0.3em 0.5em;
            }
        }

        &.medium {
            padding: 0.6em 1.2em;
            &.icon-only {
                padding: 0.6em 0.8em;
            }
        }
    }
</style>
