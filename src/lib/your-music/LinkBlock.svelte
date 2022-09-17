<script lang="ts">
    import type { ArtistLinkItem } from "src/App";
    import { draggedScrapbookItems } from "../../data/store";
    import { open } from "@tauri-apps/api/shell";

    export let item: ArtistLinkItem;

    export let style: "dashed" | "outline" = "dashed";
    async function openUrl() {
        open(item.url);
    }

    function onDragStart(ev: DragEvent) {
        console.log("dragStart");
        // Add this element's id to the drag payload so the drop handler will
        // know which element to add to its tree
        $draggedScrapbookItems = [item];
        $draggedScrapbookItems = $draggedScrapbookItems;
    }
</script>

<div
    class="item {style}"
    draggable="true"
    on:dragstart={onDragStart}
    on:click={openUrl}
>
    <div class="container">
        {#if item.imageUrl} <img src={item.imageUrl} alt="thumnbail" /> {/if}
        <div class="item-info">
            <iconify-icon icon="akar-icons:link-chain" />
            <p>{item.name}</p>
        </div>
    </div>

    <div class="tags">
        {#each item.tags as tag}
            <p>{tag}</p>
        {/each}
    </div>
</div>

<style lang="scss">
    .item {
        position: relative;
        width: 140px;
        display: flex;
        flex-direction: column;
        border-radius: 4px;
        background-color: rgba(138, 138, 138, 0.067);

        &.dashed {
            .container {
                border-style: dashed;
            }
        }

        &.outline {
            .container {
                border-style: solid;
            }
        }

        .container {
            height: 60px;
            overflow: hidden;
            border-radius: 4px;
            border-width: 1px;
            border-color: rgb(97, 97, 97);
            background-color: rgba(138, 138, 138, 0.067);
            padding: 0.7em 1em;
            display: flex;
            align-items: flex-end;
            justify-content: flex-end;
            position: relative;
            cursor: pointer;

            .item-info {
                display: flex;
                width: 100%;
                height: fit-content;
                align-items: flex-end;
                gap: 5px;
                z-index: 2;
                p {
                    margin: 0;
                    line-height: 1em;
                    width: 100%;
                    text-align: end;
                }
                iconify-icon {
                    width: auto;
                    height: fit-content;
                    display: flex;
                    color: rgb(70, 227, 227);
                }
            }
            &:hover {
                border: 1px solid rgb(70, 227, 227);
            }
            img {
                width: 100%;
                height: 100%;
                position: absolute;
                object-fit: cover;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                z-index: 0;
                opacity: 0.4;
            }
        }

        .tags {
            display: flex;
            flex-direction: row;
            flex-wrap: wrap;
            gap: 5px;
            > p {
                font-size: 12px;
                margin: 0;
                width: fit-content;
                border-radius: 3px;
                color: rgba(255, 127, 80, 1);
                /* border: 1px solid rgb(255, 255, 255, 0.2); */
                padding: 0 0.3em;
            }
        }
    }
</style>
