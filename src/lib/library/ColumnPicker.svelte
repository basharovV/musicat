<script lang="ts">
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";

    export let columnOrder: String[];
    export let fields;
    export let isOrderChanged = false;
    export let onResetOrder;
    export let pos = { x: 0, y: 0 };
    export let showMenu = false;

    $: columns = fields.map((f) => {
        f.show = columnOrder.includes(f.value);
        return f;
    });

    function closeMenu() {
        showMenu = false;
    }

    function toggleField(field) {
        const found = fields.find((f) => f.value === field.value);
        found.show = !found.show;
        fields = [...fields];

        const index = columnOrder.indexOf(field.value);

        if (index === -1) {
            if (found.show) {
                columnOrder = [...columnOrder, field.value];
            }
        } else {
            if (!found.show) {
                columnOrder.splice(index, 1);
                columnOrder = [...columnOrder];
            }
        }
    }
    // Enrichers
</script>

{#if showMenu}
    <Menu {...pos} onClickOutside={closeMenu} fixed>
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
            text="Reset order"
            onClick={onResetOrder}
            isDisabled={!isOrderChanged}
        />
    </Menu>
{/if}
