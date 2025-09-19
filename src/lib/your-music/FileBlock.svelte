<script lang="ts">
    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-shell";
    import type { ArtistFileItem, Song } from "src/App";
    import { draggedScrapbookItems } from "../../data/store";
    import { setQueue } from "../../data/storeHelper";
    import Icon from "../ui/Icon.svelte";

    export let item: ArtistFileItem;
    export let style: "dashed" | "outline" = "outline";

    async function openFile() {
        if (item.fileType.type === "audio") {
            // Play it in this app
            const song = await invoke<Song>("get_song_metadata", {
                event: {
                    path: item.path,
                    isImport: false,
                    includeFolderArtwork: false,
                    includeRawTags: false,
                },
            });
            if (!song) {
                open(item.path);
                return;
            }
            setQueue([song], 0);
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
    role="button"
    tabindex="0"
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
                <Icon icon="system-uicons:audio-wave" size={40} />
            </div>
        {/if}
        <div class="item-info">
            {#if item.fileType.type === "audio"}
                <Icon
                    icon="bi:file-earmark-play"
                    color="var(--atk-icon-audio)"
                />
            {:else if item.fileType.type === "video"}
                <Icon
                    icon="dashicons:editor-video"
                    color="var(--atk-icon-video)"
                />
            {:else if item.fileType.type === "image"}
                <Icon icon="dashicons:editor-video" />
            {:else if item.fileType.type === "txt"}
                <Icon
                    icon="dashicons:editor-video"
                    color="var(--atk-icon-lyric)"
                />
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
            border-color: color-mix(
                in srgb,
                var(--type-bw-inverse) 11%,
                transparent
            );
            background-color: color-mix(
                in srgb,
                var(--type-bw-inverse) 4%,
                transparent
            );
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
            }
        }
    }
</style>
