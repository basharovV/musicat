<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";
    import { audioDir } from "@tauri-apps/api/path";
    import { liveQuery } from "dexie";
    import type {
        ArtistContentItem,
        ArtistFileItem,
        ArtistLinkItem,
        ContentItem
    } from "src/App";
    import { db } from "../../data/db";
    import { getContentFileType } from "../../utils/FileUtils";

    import {
        droppedFiles,
        fileDropHandler,
        hoveredFiles
    } from "../../data/store";
    import FileBlock from "./FileBlock.svelte";
    import LinkBlock from "./LinkBlock.svelte";
    import { onMount } from "svelte";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { flip } from "svelte/animate";
    import { quadInOut } from "svelte/easing";
    import { getLinkItemWithData } from "../../utils/URLMetadata";

    $: contentItems = liveQuery(async () => {
        return await db.scrapbook.toArray();
    });

    function addFile(filePath) {
        console.log("adding item", filePath);
        const contentFileType = getContentFileType(filePath);
        console.log("type", contentFileType);
        const filename = filePath.split("/")?.pop() ?? "";
        const toAdd: ArtistFileItem = {
            name: filename,
            tags: [],
            type: "file",
            fileType: contentFileType,
            path: filePath
        };

        db.scrapbook.add(toAdd);
    }

    async function addLink(url) {
        const item = await getLinkItemWithData(url);
        db.scrapbook.add(item);
    }

    function importContentItem() {
        openImportDialog();
    }

    export async function openImportDialog() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: await audioDir()
        });
        if (Array.isArray(selected)) {
            // user selected multiple files
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single directory
            addFile(selected);
        }
    }

    function onMouseEnter() {
        console.log("Entered scrapbook: hovered files: ", $hoveredFiles);
        if ($hoveredFiles.length > 0) {
            $fileDropHandler = "scrapbook";
        }
    }

    function onMouseLeave() {}

    async function handleFileDrop(files: string[]) {
        console.log("drop:", files);
        for (const droppedFile of files) {
            addFile(droppedFile);
        }
        $droppedFiles = [];
        $hoveredFiles = [];
    }

    let isDragging = false;
    let container: HTMLElement;
    let containerRect;

    function onDragEnter(evt) {
        isDragging = true;
        evt.dataTransfer.dropEffect = "copy";
        console.log("dragenter", evt.dataTransfer.items);
    }

    function onDragLeave(evt: DragEvent) {
        console.log(`container, top: ${containerRect.top}`);
        console.log(`container, left: ${containerRect.left}`);
        console.log(
            `container, bottom: ${containerRect.height + containerRect.top}`
        );
        console.log(`container, right: ${containerRect.right}`);

        console.log(`evt, y: ${evt.clientY}`);
        console.log(`evt, x: ${evt.clientX}`);

        if (
            evt.clientY < containerRect.top ||
            evt.clientY >= container.clientHeight + containerRect.top ||
            evt.clientX < containerRect.left ||
            evt.clientX >= container.clientWidth + containerRect.left
        ) {
            isDragging = false;
        }
        console.log("isDragging", isDragging);
    }

    async function getDropDataAsString(
        item: DataTransferItem
    ): Promise<string> {
        return new Promise((resolve, reject) => {
            try {
                item.getAsString((str) => {
                    console.log("got string", str);
                    resolve(str);
                });
            } catch (err) {
                reject(err);
            }
        });
    }

    async function onDrop(evt: DragEvent) {
        const items = evt.dataTransfer.items;
        console.log("items", evt.dataTransfer.items);
        for (let i = 0; i < items.length; i++) {
            console.error("hmm");
            try {
                if (items[i].kind === "string") {
                    const data = await getDropDataAsString(items[i]);
                } else if (items[i].kind === "file") {
                    const data = await items[i].webkitGetAsEntry();
                    console.log("data", data);
                    if (data.isFile) {
                    }
                }
            } catch (err) {
                console.error("error doing drop: ", err);
            }
        }
    }

    onMount(() => {
        containerRect = container.getBoundingClientRect();
    });

    $: {
        console.log($fileDropHandler);
    }

    $: {
        if (
            $droppedFiles &&
            $droppedFiles.length > 0 &&
            $fileDropHandler === "scrapbook"
        ) {
            handleFileDrop($droppedFiles);
        }
    }

    // Right-click menu

    let rightClickedItem: ArtistContentItem;
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
        if (rightClickedItem?.id) {
            // Delete the item
            db.scrapbook.delete(rightClickedItem.id);
            closeMenu();
        }
    }

    let rightClickedPos = { x: 0, y: 0 };

    function onRightClick(e, item) {
        rightClickedItem = item;
        rightClickedPos = { x: e.clientX, y: e.clientY };
    }

    function onPaste(evt: ClipboardEvent) {
        const data = evt.clipboardData.getData("text");
        if (data && data.startsWith("https://")) {
            addLink(data);
        }
    }
</script>

<container
    bind:this={container}
    on:mouseenter={onMouseEnter}
    on:mouseleave={onMouseLeave}
    on:dragenter={onMouseEnter}
    on:paste|preventDefault={onPaste}
    class:file-drop={$hoveredFiles.length}
>
    <div class="legend">
        <div class="lyrics">
            <iconify-icon icon="bi:file-earmark-text" />
            <p>Lyrics</p>
        </div>
        <div class="audio">
            <iconify-icon icon="bi:file-earmark-play" />
            <p>Audio</p>
        </div>
        <div class="video">
            <iconify-icon icon="dashicons:editor-video" />
            <p>Video</p>
        </div>
        <div class="link">
            <iconify-icon icon="akar-icons:link-chain" />
            <p>Link</p>
        </div>
    </div>
    <div class="scrapbook">
        {#if $contentItems}
            <div>
                {#each $contentItems as item (item.id)}
                    <div
                        animate:flip={{ duration: 180, easing: quadInOut }}
                        class="item"
                        on:contextmenu|preventDefault={(e) => {
                            onRightClick(e, item);
                        }}
                    >
                        {#if item.type === "file"}
                            <FileBlock {item} />
                        {:else if item.type === "link"}
                            <LinkBlock {item} />
                        {/if}
                    </div>
                {/each}
                {#if $hoveredFiles.length}
                    <div class="item placeholder">
                        <p>Drop some media in here!</p>
                    </div>
                {/if}
            </div>
        {/if}
        <button on:click={importContentItem}>Import something</button>
        <p>Audio / video / image / lyrics .txt file</p>
    </div>
</container>

{#if rightClickedItem}
    <Menu
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
        position: relative;
        height: 100%;
        display: block;

        &.dragging {
            * {
                pointer-events: none !important;
            }
        }
    }
    .legend {
        display: inline-flex;
        gap: 10px;
        margin-bottom: 1em;
        top: 0;
        position: sticky;
        > div {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            p {
                font-size: 13px;
                margin: 0;
                color: rgb(171, 171, 171);
            }
            &.lyrics > iconify-icon {
                color: rgb(212, 212, 66);
            }
            &.audio > iconify-icon {
                color: rgb(199, 69, 199);
            }
            &.video > iconify-icon {
                color: rgb(224, 72, 72);
            }
            &.link > iconify-icon {
                color: rgb(70, 227, 227);
            }
        }
    }
    .scrapbook {
        display: flex;
        flex-direction: column;
        > div {
            display: flex;
            flex-direction: row;
            gap: 0.6em 0.6em;
            flex-wrap: wrap;
        }

        button {
            margin-top: 2em;
        }
        p {
            opacity: 0.5;
        }
    }
    .item {
        height: auto;
        width: auto;

        &.placeholder {
            border: 1px dashed white;
            border-radius: 4px;
            padding: 1em;
        }
    }
</style>
