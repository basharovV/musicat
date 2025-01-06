<script lang="ts">
    export let tags = [];
    export let selectedTags = [];
    function selectTag(tag: string) {
        if (selectedTags.includes(tag)) {
            selectedTags.splice(
                selectedTags.findIndex((t) => t === tag),
                1,
            );
        } else {
            selectedTags.push(tag);
        }
        selectedTags = selectedTags;
    }
</script>

<div class="tag-cloud">
    {#each tags as tag}
        <div
            class="tag"
            class:selected={selectedTags.includes(tag)}
            on:click={() => selectTag(tag)}
        >
            <p>{tag}</p>
        </div>
    {/each}
</div>

<style lang="scss">
    .tag-cloud {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        gap: 5px;
        justify-content: stretch;
    }

    .tag {
        border-radius: 20px;
        padding: 1px 10px;
        background-color: rgba(0, 0, 0, 0.346);
        box-sizing: border-box;
        border: 1px solid transparent;

        &:hover {
            border: 1px solid rgb(140, 140, 140);
        }

        &:active {
            opacity: 0.5;
        }

        &.selected {
            color: #d1c5f5;
            border-color: #5f32e7;
            background-color: #5123dd27;
        }

        p {
            margin: 0;
            user-select: none;
            cursor: default;
        }
    }
</style>
