<script lang="ts">
    import type { Compression } from "../../App";
    import { compressionSelected } from "../../data/store";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import Icon from "./Icon.svelte";

    let COMPRESSION_OPTIONS: { value: Compression; label: string }[] = [
        {
            value: "lossy",
            label: "lossy"
        },
        {
            value: "lossless",
            label: "lossless"
        },
        {
            value: "both",
            label: "lossy + lossless"
        }
    ];
    $: selected = COMPRESSION_OPTIONS.find(
        (c) => c.value === $compressionSelected
    );

    // Only show songs with selected compression type
    export let options = [];

    let showSelector = false;
</script>

<div class="compression-selector">
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
            <Menu
                position="relative"
                onClickOutside={() => (showSelector = false)}
            >
                {#each COMPRESSION_OPTIONS as option}
                    <MenuOption
                        onClick={() => {
                            $compressionSelected = option.value;
                            showSelector = false;
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
    .compression-selector {
        cursor: default;
        display: flex;
        position: relative;
        width: max-content;

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
            z-index: 2;
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
