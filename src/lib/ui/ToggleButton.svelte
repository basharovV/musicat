<script lang="ts">
    import Icon from "../ui/Icon.svelte";

    export let text = "";
    export let icon = null;
    export let iconSize = 14;

    export let isSelected = false;
    export let onClick = null;
</script>

<div
    class="toggle-button waveform"
    class:selected={isSelected}
    on:click={(ev) => (onClick ? onClick(ev) : null)}
>
    <Icon class={isSelected ? "active" : "inactive"} {icon} {iconSize} />
    <p>{text}</p>
</div>

<style lang="scss">
    .toggle-button {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 5px;
        cursor: default;

        border: 1px solid var(--border);
        border-radius: 4px;
        padding: 0 4px;
        margin-right: 8px;
        position: relative;
        &:hover {
            background-color: color-mix(
                in srgb,
                var(--secondary) 20%,
                transparent
            );
        }
        &:active {
            background-color: rgba(128, 128, 128, 0.391);
        }
        &.selected {
            border: 1px solid var(--primary);
        }
        p {
            color: var(--primary);
            margin: 0 0 1px 0;
            line-height: normal;
        }
    }
</style>
