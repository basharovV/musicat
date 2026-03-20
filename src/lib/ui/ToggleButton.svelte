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

        border: 1px solid rgba(128, 128, 128, 0.159);
        border-radius: 4px;
        padding: 0 4px;
        margin-right: 8px;
        position: relative;
        &:hover {
            background-color: rgba(128, 128, 128, 0.191);
        }
        &:active {
            background-color: rgba(128, 128, 128, 0.391);
        }
        &.selected {
            border: 1px solid rgb(from var(--type-bw-inverse) r g b / 0.5);
            ::before {
                position: absolute;
                top: -11px;
                left: 0;
                right: 0;
                margin: 0 auto;
                height: 10px;
                width: 1.5px;
                background-color: rgb(from var(--type-bw-inverse) r g b / 0.5);
            }
        }
        p {
            color: var(--text-secondary);
            margin: 0 0 1px 0;
            line-height: normal;
        }
    }
</style>
