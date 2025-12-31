<script lang="ts">
    import Icon from "../ui/Icon.svelte";

    export let label: string;
    export let value: string | null;
    export let placeholder = "Select a location";

    export let onSelect: () => void;
    export let onClear: (() => void) | null = null;
</script>

<tr>
    <td>{label}</td>
    <td>
        <div class="path-setting">
            <small class:value-empty={!value}>
                {value ?? placeholder}
            </small>

            <div class="actions">
                <Icon
                    icon="material-symbols:folder"
                    onClick={onSelect}
                    title="Select folder"
                />
                {#if value && onClear}
                    <Icon
                        icon="material-symbols:close"
                        onClick={onClear}
                        title="Clear"
                    />
                {/if}
            </div>
        </div>
    </td>
</tr>

<style lang="scss">
    td {
        vertical-align: middle;

        padding-right: 10px;
        text-align: start;
        opacity: 0.7;
        width: 40%;
    }

    .path-setting {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;

        small {
            margin: 0;
            overflow: hidden;
            text-overflow: ellipsis;

            &.value-empty {
                opacity: 0.6;
                font-style: italic;
            }
        }

        .actions {
            display: flex;
            gap: 6px;
            cursor: pointer;
        }
    }
</style>
