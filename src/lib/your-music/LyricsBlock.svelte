<script lang="ts">
    import type { ArtistFileItem, ArtistLyricsItem } from "src/App";
    import Icon from "../ui/Icon.svelte";

    export let item: ArtistLyricsItem;
</script>

<div class="item">
    <div class="container">
        <div class="background">
            <img src="images/scribbles.png" />
        </div>
        <div class="item-info">
            <Icon icon="bi:file-earmark-text" />
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
            }
            &:hover {
                border: 1px solid rgb(212, 212, 66);
            }
            img {
                width: 100%;
                height: 100%;
                object-fit: contain;
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
