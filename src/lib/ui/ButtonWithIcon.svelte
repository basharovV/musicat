<script lang="ts">
    import Icon from "./Icon.svelte";

    export let icon = null;
    export let onClick;
    export let text;
    export let theme: "solid" | "transparent" = "solid";
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
        color: rgba(255, 255, 255, 0.811);
        white-space: nowrap;
        cursor: default;
        transition: border-color 0.1s;
        user-select: none;
        p {
            margin: 0;
            line-height: normal;
        }

        &.disabled {
            background-color: #5d5d5d !important;
            color: rgb(181, 171, 171);
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
            background-color: #1a1a1a;
            white-space: nowrap;
            &:hover {
                border-color: #646cff;
            }
        }
        &.theme-transparent {
            border: 1px solid rgba(255, 255, 255, 0.2);
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
