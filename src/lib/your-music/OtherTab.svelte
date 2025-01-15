<script lang="ts">
    import type { ArtistContentItem, ContentItem } from "src/App";
    import { songDetailsUpdater } from "../../data/store";
    import { onMount } from "svelte";
    import { flip } from "svelte/animate";
    import { quadInOut } from "svelte/easing";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuOption from "../ui/menu/MenuOption.svelte";
    import FileBlock from "./FileBlock.svelte";
    import LinkBlock from "./LinkBlock.svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { audioDir } from "@tauri-apps/api/path";

    export let items: ArtistContentItem[];
    export let enabled = false;
    export let showDropPlaceholder = false;
    export let addContentItem;
    export let deleteContentItem;

    $: {
        console.log("ITEMS:", items);
    }

    // Right-click menu

    let rightClickedItem: ArtistContentItem;
    let rightClickedItemIdx: 0;
    let isConfirmingDelete = false;

    function closeMenu() {
        rightClickedItem = null;
        isConfirmingDelete = false;
    }

    function deleteItem() {
        if (!isConfirmingDelete) {
            isConfirmingDelete = true;
            return;
        }
        deleteContentItem(rightClickedItemIdx);
        closeMenu();
    }

    let rightClickedPos = { x: 0, y: 0 };

    function onRightClick(e, item, idx) {
        rightClickedItem = item;
        rightClickedItemIdx = idx;
        rightClickedPos = { x: e.clientX, y: e.clientY };
    }

    function importContentItem() {
        openImportDialog();
    }

    export async function openImportDialog() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: await audioDir(),
        });
        if (Array.isArray(selected)) {
            // user selected multiple files
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single directory
            addContentItem([selected]);
        }
    }
</script>

<container>
    {#if enabled}
        {#if items}
            <div class="items">
                {#each items as item, idx (item.name)}
                    <div
                        animate:flip={{ duration: 180, easing: quadInOut }}
                        class="item"
                        on:contextmenu|preventDefault={(e) => {
                            onRightClick(e, item, idx);
                        }}
                    >
                        {#if item.type === "file"}
                            <FileBlock {item} style="outline" />
                        {:else if item.type === "link"}
                            <LinkBlock {item} style="outline" />
                        {/if}
                    </div>
                {/each}

                {#if showDropPlaceholder}
                    <div class="placeholder">
                        <p>Drop some media in here!</p>
                    </div>
                {/if}
            </div>
            {#if items.length === 0}
                <button on:click={importContentItem}>Add some files</button>
                <p class="prompt">Or drag something from the scrapbook.</p>
                <small
                    >Note: this just creates a link to the existing files on
                    your system.</small
                >
            {/if}
        {/if}
    {:else}
        <br />
        <p>Create a project to add additional content</p>
    {/if}
</container>

{#if rightClickedItem}
    <Menu
        fixed={true}
        x={rightClickedPos.x}
        y={rightClickedPos.y}
        onClickOutside={closeMenu}
    >
        <MenuOption isDisabled={true} text={rightClickedItem.name} />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={deleteItem}
            text={"Delete item"}
            confirmText="Click again to delete"
        />
    </Menu>
{/if}

<style lang="scss">
    container {
        display: flex;
        flex-direction: column;
        padding: 2em;
    }

    .items {
        display: flex;
        flex-direction: row;
        gap: 0.6em 0.6em;
        flex-wrap: wrap;
    }

    button {
        margin-top: 2em;
        width: fit-content;
    }
    .prompt {
        opacity: 0.6;
        margin-top: 1em;
        margin-bottom: 0;
    }
    small {
        opacity: 0.6;
        margin: 0;
        font-size: 0.9em;
    }

    .placeholder {
        border: 1px dashed white;
        border-radius: 4px;
        padding: 1em;
        height: 60px;
    }
</style>
