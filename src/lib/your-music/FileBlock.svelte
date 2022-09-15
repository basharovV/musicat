<script lang="ts">
    import type { ArtistFileItem } from "src/App";

    export let item: ArtistFileItem;
    export let mediaType: "audio" | "video" | "other";
    if (item.filename.match(/.(wav|mp3|aiff|ogg|flac)$/)) {
        mediaType = "audio";
    } else if (item.filename.match(/.(mp4|mov)/)) {
        mediaType = "video";
    } else {
        mediaType = "other";
    }
</script>

<div class="item">
    <div class="container">
        <div class="background">
            <iconify-icon
                icon={mediaType === "video"
                    ? "bxs:video"
                    : "system-uicons:audio-wave"}
            />
        </div>
        <div class="item-info {mediaType}">
            <iconify-icon
                icon={mediaType === "video"
                    ? "dashicons:editor-video"
                    : "bi:file-earmark-play"}
            />

            <p>{item.filename}</p>
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
        width: fit-content;
        display: flex;
        flex-direction: column;

        .container {
            height: 60px;
            overflow: hidden;
            border-radius: 4px;
            border: 1px dashed rgb(97, 97, 97);
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
                p {
                    margin: 0;
                    line-height: 1em;
                }
                iconify-icon {
                    width: auto;
                    height: fit-content;
                    display: flex;
                    color: rgb(212, 212, 66);
                }

                &.audio > iconify-icon {
                    color: rgb(199, 69, 199);
                }
                &.video > iconify-icon {
                    color: rgb(224, 72, 72);
                }
            }

            &:hover {
                border: 1px solid rgb(199, 69, 199);
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
