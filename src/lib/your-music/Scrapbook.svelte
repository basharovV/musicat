<script lang="ts">
    import md5 from "md5";
    import { open } from "@tauri-apps/plugin-dialog";
    import { open as openShell } from "@tauri-apps/plugin-shell";
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
        hoveredFiles,
        isSettingsOpen,
        userSettings
    } from "../../data/store";
    import FileBlock from "./FileBlock.svelte";
    import LinkBlock from "./LinkBlock.svelte";
    import { onMount } from "svelte";
    import Menu from "../menu/Menu.svelte";
    import MenuOption from "../menu/MenuOption.svelte";
    import { flip } from "svelte/animate";
    import { quadInOut } from "svelte/easing";
    import { getLinkItemWithData } from "../../utils/URLMetadata";
    import TagCloud from "./TagCloud.svelte";
    import MenuInput from "../menu/MenuInput.svelte";
    import hotkeys from "hotkeys-js";
    import Icon from "../ui/Icon.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import { copyFile, readDir } from "@tauri-apps/plugin-fs";
    import {
        addScrapbookFile,
        scanScrapbook
    } from "../../data/ArtistsToolkitData";
    import LL from "../../i18n/i18n-svelte";
    import tippy from "tippy.js";

    let contentTypes = [
        {
            icon: "bi:file-earmark-text",
            name: "lyrics"
        },
        {
            icon: "bi:file-earmark-play",
            name: "audio"
        },
        {
            icon: "dashicons:editor-video",
            name: "video"
        },
        {
            icon: "akar-icons:link-chain",
            name: "link"
        }
    ];

    let selectedContentTypes = [];

    function toggleContentType(contentType: string) {
        if (selectedContentTypes.includes(contentType)) {
            selectedContentTypes.splice(
                selectedContentTypes.findIndex((t) => t === contentType),
                1
            );
        } else {
            selectedContentTypes.push(contentType);
        }
        selectedContentTypes = selectedContentTypes;
    }

    $: allContentItems = liveQuery(async () => {
        return await db.scrapbook.toArray();
    });

    $: filteredContentItems =
        $allContentItems?.filter((item) => {
            let matchesContentType = selectedContentTypes.length
                ? selectedContentTypes.includes(item?.fileType?.type)
                : true;
            let matchesTags = selectedTags.length ? false : true;
            if (selectedTags.length) {
                const matchingTags = item.tags.filter((i) =>
                    selectedTags.includes(i)
                );
                matchesTags = matchingTags.length > 0;
            }
            return matchesTags && matchesContentType;
        }) ?? [];

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
            addScrapbookFile(selected.path);
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
            const filename = droppedFile.split("/")?.pop() ?? "";
            copyFile(
                droppedFile,
                `${$userSettings.scrapbookLocation}/${filename}`
            );
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

    let scrapbookDirNotFound = false;
    let isMounted = false;

    $: if (isMounted && $userSettings.scrapbookLocation) {
        loadScrapbookDir();
    }

    async function loadScrapbookDir() {
        try {
            await scanScrapbook();
            scrapbookDirNotFound = false;
        } catch (err) {
            if (err.message === "Scrapbook location not found") {
                scrapbookDirNotFound = true;
            }
        }
    }

    onMount(async () => {
        isMounted = true;
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

    let contextItem: ArtistContentItem;
    let isConfirmingDelete = false;

    function closeMenu() {
        contextItem = null;
        tagEditorVisible = false;
        isConfirmingDelete = false;
    }

    function deleteItem() {
        if (!isConfirmingDelete) {
            isConfirmingDelete = true;
            return;
        }
        if (contextItem?.id) {
            // Delete the item
            db.scrapbook.delete(contextItem.id);
            closeMenu();
        }
    }

    let rightClickedPos = { x: 0, y: 0 };

    function onRightClick(e, item) {
        contextItem = item;
        rightClickedPos = { x: e.clientX, y: e.clientY };
    }

    function onPaste(evt: ClipboardEvent) {
        console.log("paste", evt);
        const data = evt.clipboardData.getData("text");
        if (data && data.startsWith("https://")) {
            addLink(data);
        }
    }

    $: tags =
        $allContentItems?.reduce((tags, item) => {
            item.tags.forEach((t) => {
                if (!tags.includes(t)) {
                    tags.push(t);
                }
            });
            return tags;
        }, []) ?? [];

    let selectedTags = [];

    let tagEditorVisible = false;
    let tagUserInput = "";
    function showTagEditor(e: MouseEvent, item: ArtistContentItem) {
        contextItem = item;
        tagEditorVisible = true;
        rightClickedPos = { x: e.clientX, y: e.clientY };
    }

    function deleteTag(tag) {
        contextItem.tags.splice(
            contextItem.tags.findIndex((t) => t === tag),
            1
        );
        contextItem = contextItem;
        db.scrapbook.update(contextItem.id, {
            tags: contextItem.tags
        });
    }
    function addTagToContextItem() {
        if (tagAutoCompleteValue?.length) {
            tagUserInput = tagAutoCompleteValue;
        }
        db.scrapbook.update(contextItem.id, {
            tags: [...contextItem.tags, tagUserInput.toLowerCase().trim()]
        });
        tagUserInput = "";
        contextItem = null;
        tagEditorVisible = false;
    }
    $: tagAutoCompleteValue =
        tagUserInput.length && tags?.find((v) => v.startsWith(tagUserInput));

    hotkeys("esc", function (event, handler) {
        closeMenu();
    });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<container
    bind:this={container}
    on:mouseenter={onMouseEnter}
    on:mouseleave={onMouseLeave}
    on:dragenter={onMouseEnter}
    on:paste|preventDefault={onPaste}
    class:file-drop={$hoveredFiles.length}
>
    <div class="scrapbook-header">
        <h2>Scrapbook</h2>
        <div
            use:tippy={{
                content: $userSettings.scrapbookLocation,
                placement: "top"
            }}
        >
            <Icon
                icon="material-symbols:folder"
                onClick={() => {
                    openShell($userSettings.scrapbookLocation);
                }}
            ></Icon>
        </div>
    </div>
    {#if $userSettings.scrapbookLocation === null}
        <p>{$LL.artistsToolkit.scrapbook.setupHint()}</p>
        <ButtonWithIcon
            onClick={() => ($isSettingsOpen = true)}
            text={$LL.artistsToolkit.scrapbook.openSettings()}
        />
    {:else if scrapbookDirNotFound}
        <div class="callout-error">
            <Icon icon="ant-design:warning-outlined" />
            <p>{$LL.artistsToolkit.scrapbook.notFoundError()}</p>
            <ButtonWithIcon
                onClick={() => ($isSettingsOpen = true)}
                text={$LL.artistsToolkit.scrapbook.openSettings()}
            />
        </div>
    {:else}
        <div class="subheader">
            <div class="legend">
                {#each contentTypes as contentType}
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <div
                        class:selected={selectedContentTypes.includes(
                            contentType.name
                        )}
                        class={contentType.name}
                        on:click={() => toggleContentType(contentType.name)}
                    >
                        <Icon icon={contentType.icon} />
                        <p>{contentType.name}</p>
                    </div>
                {/each}
            </div>
            <div class="tag-cloud">
                <TagCloud {tags} bind:selectedTags />
            </div>
        </div>
        <div class="scrapbook">
            {#if filteredContentItems}
                <div>
                    {#each filteredContentItems as item (item?.id)}
                        <!-- svelte-ignore a11y-no-static-element-interactions -->
                        <div
                            animate:flip={{
                                duration: 180,
                                easing: quadInOut
                            }}
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

                            <div class="tags">
                                {#each item.tags as tag}
                                    <p
                                        class:selected={selectedTags.includes(
                                            tag
                                        )}
                                    >
                                        {tag}
                                    </p>
                                {/each}
                                <!-- svelte-ignore a11y-no-static-element-interactions -->
                                <div
                                    class="add-tag"
                                    on:click={(e) => showTagEditor(e, item)}
                                >
                                    <p>edit tags</p>
                                </div>
                            </div>
                        </div>
                    {/each}
                    {#if $hoveredFiles.length}
                        <div class="item placeholder">
                            <p>Drop some media in here!</p>
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    {/if}
</container>

{#if contextItem && !tagEditorVisible}
    <Menu
        x={rightClickedPos.x}
        y={rightClickedPos.y}
        onClickOutside={closeMenu}
        fixed
    >
        <MenuOption isDisabled={true} text={contextItem.name} />
        <MenuOption
            isDestructive={true}
            isConfirming={isConfirmingDelete}
            onClick={deleteItem}
            text={"Delete item"}
            confirmText="Click again to delete"
        />
    </Menu>
{/if}

{#if contextItem && tagEditorVisible}
    <Menu
        x={rightClickedPos.x}
        y={rightClickedPos.y}
        onClickOutside={closeMenu}
        fixed
    >
        <MenuOption isDisabled={true} text="Edit tags" />
        {#each contextItem.tags as tag}
            <MenuOption text={tag} onDelete={() => deleteTag(tag)} />
        {/each}
        <MenuInput
            bind:value={tagUserInput}
            autoCompleteValue={tagAutoCompleteValue}
            onEnterPressed={addTagToContextItem}
            autoFocus
            placeholder="Add a tag"
            onEscPressed={closeMenu}
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

    .scrapbook-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.2em 2em 0;
    }

    h2 {
        margin: 0;
        font-family: "Snake";
        font-size: 3em;
        color: var(--text);
        line-height: initial;
    }

    .callout-error {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;
        p {
            margin: 0;
            flex: 1;
            white-space: nowrap;
        }
    }

    .subheader {
        display: flex;
        flex-direction: column;
        position: sticky;
        top: 0;
        width: 100%;
        /* background-color: color-mix(in srgb, var(--background) 90%, transparent); */
        backdrop-filter: blur(10px);
        z-index: 0;
        padding: 1em 2em;
        border-bottom: 0.7px solid
            color-mix(in srgb, var(--inverse) 50%, transparent);
    }
    .legend {
        display: inline-flex;
        gap: 10px;
        width: 100%;
        margin-bottom: 1em;
        top: 0;
        position: sticky;
        justify-content: space-between;
        > div {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 5px;
            &:hover {
                p {
                    color: var(--text);
                    border-bottom: 1px solid rgb(154, 149, 149);
                }
            }
            &.selected {
                p {
                    color: var(--text);
                    border-bottom: 1px solid white;
                }
            }

            p {
                box-sizing: border-box;
                border-bottom: 1px solid transparent;
                font-size: 13px;
                margin: 0;
                color: var(--text);
                cursor: default;
                user-select: none;
                text-transform: capitalize;
            }
            &.lyrics {
                color: #d4d442;
            }
            &.audio {
                color: rgb(199, 69, 199);
            }
            &.video {
                color: rgb(224, 72, 72);
            }
            &.link {
                color: rgb(70, 227, 227);
            }
        }
    }

    .tag-cloud {
    }
    .scrapbook {
        display: flex;
        flex-direction: column;
        overflow: auto;
        padding: 1em 2em;
        > div {
            display: flex;
            flex-direction: row;
            gap: 0.6em 0.6em;
            flex-wrap: wrap;
        }

        button {
            margin-top: 2em;
        }
        > p {
            opacity: 0.5;
        }
    }
    .item {
        height: auto;
        width: auto;
        max-width: 165px;

        &.placeholder {
            border: 1px dashed white;
            border-radius: 4px;
            padding: 1em;
        }

        .tags {
            display: flex;
            flex-direction: row;
            flex-wrap: wrap;
            column-gap: 5px;
            justify-content: right;
            align-items: center;
            min-height: 23px;
            > p {
                cursor: default;
                font-size: 12px;
                line-height: 18px;
                margin: 0;
                width: fit-content;
                border-radius: 3px;
                color: rgb(168, 151, 145);
                border: 1px solid transparent;
                padding: 0 0.3em;

                &.selected {
                    color: #d5cdf0;
                    text-shadow: 0px 2px 10px #c0bbd0;
                }
            }
            .add-tag {
                p {
                    margin: 0;
                    font-size: 12px;
                    line-height: 12px;
                    cursor: default;
                    box-sizing: border-box;
                    color: #685c8a;
                    &:hover {
                        opacity: 0.8;
                    }
                    &:active {
                        opacity: 0.5;
                    }
                }
            }

            .tag-input {
                position: absolute;
            }
        }
    }
</style>
