<script lang="ts">
    import { open } from "@tauri-apps/api/shell";
    import type { ArtistFileItem } from "src/App";
    import { draggedScrapbookItems } from "../../data/store";
    import {
        getMetadataFromFile,
        getSongFromMetadata
    } from "../../data/LibraryImporter";
    import audioPlayer from "../AudioPlayer";
    import { convertFileSrc } from "@tauri-apps/api/tauri";

    export let item: ArtistFileItem;
    export let style: "dashed" | "outline" = "dashed";

    async function openFile() {
        if (item.fileType.type === "audio") {
            // Play it in this app
            const metadata = await getMetadataFromFile(item.path, item.name);
            if (!metadata) {
                open(item.path);
                return;
            }
            const song = await getSongFromMetadata(
                item.path,
                item.name,
                metadata
            );
            audioPlayer.playSong(song);
        } else {
            // Play in default system app
            open(item.path);
        }
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
    class="item {item.fileType.type} {style}"
    on:click={openFile}
    draggable="true"
    on:dragstart={onDragStart}
>
    <div class="container">
        {#if item.fileType.type === "image"}
            <img src={convertFileSrc(item.path)} alt="thumnbail" />
        {:else if item.fileType.type === "video"}
            <!-- svelte-ignore a11y-media-has-caption -->
            <video preload="metadata" controls={false}>
                <source
                    src="{convertFileSrc(item.path)}#t=0.1"
                    type="video/mp4"
                />
            </video>
        {:else}
            <div class="background">
                <iconify-icon
                    icon={item.fileType.type === "video"
                        ? "bxs:video"
                        : "system-uicons:audio-wave"}
                />
            </div>
        {/if}
        <div class="item-info">
            {#if item.fileType.type === "audio"}
                <iconify-icon icon="bi:file-earmark-play" />
            {:else if item.fileType.type === "video"}
                <iconify-icon icon="dashicons:editor-video" />
            {:else if item.fileType.type === "image"}
                <iconify-icon icon="dashicons:editor-video" />
            {:else if item.fileType.type === "txt"}
                <iconify-icon icon="dashicons:editor-video" />
            {/if}
            <p>{item.name}</p>
        </div>
    </div>
</div>

<style lang="scss">
    .item {
        position: relative;
        width: fit-content;
        display: flex;
        flex-direction: column;
        max-width: 165px;

        &.audio {
            iconify-icon {
                color: rgb(199, 69, 199);
            }
        }
        &.video {
            iconify-icon {
                color: rgb(224, 72, 72);
            }
        }
        &.txt {
            iconify-icon {
                color: rgb(212, 212, 66);
            }
        }

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
                    overflow: hidden;
                    text-overflow: ellipsis;
                }
                iconify-icon {
                    width: auto;
                    height: fit-content;
                    display: flex;
                }
            }

            img,
            video {
                width: 100%;
                height: 100%;
                min-height: 96px;
                min-width: 170px;
                position: absolute;
                object-fit: cover;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                z-index: 0;
                opacity: 0.4;
            }

            &:hover {
                border: 1px solid rgb(199, 69, 199);
                .txt > & {
                    border: 1px solid rgb(212, 212, 66);
                }
            }
            .background {
                width: 100%;
                height: 100%;
                position: absolute;
                object-fit: cover;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                z-index: 0;
                opacity: 0.1;
                display: flex;
                align-items: center;
                justify-content: center;
                iconify-icon {
                    font-size: 40px;
                    color: white;
                }
            }
        }
    }
</style>
