<script lang="ts">
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";

    // Only show songs with selected compression type
    export let options = [];
    export let compressionSelected;

    let showSelector = false;
</script>

<div class="compression-selector">
    <div
        on:click={() => {
            showSelector = !showSelector;
        }}
        class="current"
    >
        <p>{compressionSelected.label}</p>
        <iconify-icon icon="heroicons-solid:selector" />
    </div>
    {#if showSelector}
        <div class="menu">
            <Menu
                position="relative"
                onClickOutside={() => (showSelector = false)}
            >
                {#each options as option}
                    <MenuOption
                        onClick={() => {
                            compressionSelected = option;
                            showSelector = false;
                        }}
                        singleSelection
                        text={option.label}
                        checked={option.value === compressionSelected.value}
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

        .current {
            padding: 0px 5px;
            display: inline-flex;
            flex-direction: row;
            align-items: center;
            height: fit-content;
            color: rgb(188, 188, 188);
            border: 1px solid rgba(128, 128, 128, 0.159);
            border-radius: 4px;

            &:hover {
                background-color: rgba(128, 128, 128, 0.191);
            }

            iconify-icon {
                color: grey;
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
