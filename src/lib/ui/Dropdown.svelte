<script lang="ts">
    import Menu from "../ui/menu/Menu.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import Icon from "./Icon.svelte";

    export let selected;
    export let onSelect = null;
    export let size = null;

    // Only show songs with selected compression type
    export let options = [];

    let showSelector = false;
</script>

<div class="dropdown" style={size ? `font-size: ${size}px;` : ""}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        on:click={() => {
            showSelector = !showSelector;
        }}
        class="current"
    >
        <p>{selected.label}</p>
        <Icon icon="heroicons-solid:selector" size={14} />
    </div>
    {#if showSelector}
        <div class="menu">
            <Menu position="auto" onClickOutside={() => (showSelector = false)}>
                {#each options as option}
                    <MenuOption
                        onClick={() => {
                            selected = option;
                            showSelector = false;
                            onSelect && onSelect(option.value);
                        }}
                        singleSelection
                        text={option.label}
                        checked={option.value === selected.value}
                    />
                {/each}
            </Menu>
        </div>
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

        .menu {
            position: absolute;
            bottom: 0.5em;
            left: -0.5em;
            z-index: 30;
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
