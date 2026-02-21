<script lang="ts">
    import Menu from "../ui/menu/Menu.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import Icon from "./Icon.svelte";
    import Input from "./Input.svelte";
    import PortalMenu from "./menu/PortalMenu.svelte";

    export let selected;
    export let onSelect = null;
    export let size = null;
    export let placeholder;

    interface InputOption {
        value: string;
        label: string;
    }

    // Only show songs with selected compression type
    export let options: InputOption[] = [];

    let inputField;

    $: filteredList = options.filter(
        (o) =>
            initialState ||
            o.label.toLowerCase().includes(inputValue.toLowerCase()),
    );

    let initialState = true;

    let showSelector = false;

    let inputValue = selected?.value ?? "";

    $: {
        if (inputValue?.length > 0) {
            initialState = false;
        }
    }
</script>

<div class="dropdown" style={size ? `font-size: ${size}px;` : ""}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        bind:this={inputField}
        on:click={() => {
            showSelector = !showSelector;
            initialState = true;
        }}
        class="current"
    >
        <Input bind:value={inputValue} fullWidth minimal />
        <Icon icon="heroicons-solid:selector" size={14} />
    </div>
    {#if showSelector}
        <PortalMenu
            items={filteredList.map((i) => ({
                text: i.label,
                source: i.value,
            }))}
            anchor={inputField}
            onClose={() => (showSelector = false)}
            onItemSelected={(s) => {
                inputValue = s;
                onSelect(s);
            }}
            {placeholder}
        ></PortalMenu>
    {/if}
</div>

<style lang="scss">
    .dropdown {
        cursor: default;
        display: flex;
        position: relative;
        width: max-content;
        align-items: center;

        .current {
            padding: 0px 5px;
            display: inline-flex;
            flex-direction: row;
            align-items: center;
            height: fit-content;
            color: var(--text-secondary);
            background: var(--input-bg);
            border: 1px solid rgba(128, 128, 128, 0.159);
            border-radius: 4px;
            padding-bottom: 1px;

            &:hover {
                background-color: rgba(128, 128, 128, 0.191);
            }

            &:active {
                background-color: rgba(128, 128, 128, 0.391);
            }
        }

        * {
            user-select: none;
        }

        p {
            margin: 0;
            line-height: initial;
        }
    }
</style>
