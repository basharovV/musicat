<script lang="ts">
    import type { ColumnViewModel, LibraryColumn } from "../../App";
    import type { PersistentWritable } from "../../data/storeUtils";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import { getAllColumns } from "./LibraryColumns";

    export let columnIndex = 0;
    export let displayedColumns: PersistentWritable<LibraryColumn[]>;
    let allColumns = getAllColumns();
    export let onResetOrder;

    interface ColumnOption extends ColumnViewModel {
        show: boolean;
    }

    $: columns = allColumns.map((f) => {
        const option: ColumnOption = {
            ...f,
            show:
                $displayedColumns.find((c) => c.fieldName === f.value) !==
                undefined,
        };
        return option;
    });

    function closeMenu() {}

    function toggleField(field: ColumnOption) {
        allColumns = [...allColumns];

        field.show = !field.show;
        const index = $displayedColumns.findIndex(
            (c) => c.fieldName === field.value,
        );

        if (index === -1) {
            if (field.show) {
                $displayedColumns = [
                    ...$displayedColumns.slice(0, columnIndex + 1),
                    {
                        fieldName: field.value,
                    },
                    ...$displayedColumns.slice(columnIndex + 1),
                ];
                columnIndex += 1;
            }
        } else {
            if (!field.show) {
                $displayedColumns.splice(index, 1);
                $displayedColumns = [...$displayedColumns];

                if (index === columnIndex) {
                    columnIndex -= 1;
                    if (columnIndex < -1) {
                        columnIndex = 0;
                    }
                }
            }
        }
    }
    // Enrichers
</script>

<Menu onClickOutside={closeMenu}>
    <MenuOption text="Columns" isDisabled />
    {#each columns as field}
        <MenuOption
            text={field.name}
            checked={field.show}
            onClick={() => toggleField(field)}
            borderHighlight
        />
    {/each}
    <MenuDivider />
    <MenuOption
        text="Reset to default"
        description="and auto-size columns"
        onClick={onResetOrder}
    />
</Menu>
