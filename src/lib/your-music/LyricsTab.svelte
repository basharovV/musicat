<script lang="ts">
    export let fontSize;
    export let isFullScreen;
    export let lyrics;
    export let onLyricsUpdated;
    export let enabled = false;

    $: {
        console.log("lyrics", lyrics);
    }
    $: hasLyrics = enabled && typeof lyrics === 'string';

    function onLyricsChanged(e) {
        onLyricsUpdated(e.target.value);
    }
</script>

<div class={fontSize} class:full-screen={isFullScreen}>
    {#if hasLyrics}
        <textarea
            placeholder="Start typing..."
            on:input={onLyricsChanged}
            bind:value={lyrics}
        />
    {:else}
        <br />
        <p>Create a project to add lyrics</p>
    {/if}
</div>

<style lang="scss">
    div {
        text-align: center;
        width: 100%;
        height: 100%;

        p {
            opacity: 0.5;
        }

        textarea {
            appearance: none;
            width: 100%;
            height: 100%;
            padding: 3rem 2em;
            text-align: center;
            outline: none;
            border: none;
            background: none;
            font-size: inherit;
            line-height: 1.8em;
            resize: none;
        }

        &.font-12 {
            font-size: 12px;
        }

        &.font-14 {
            font-size: 14px;
        }

        &.font-20 {
            font-size: 20px;
        }

        &.font-24 {
            font-size: 24px;
        }

        &.font-32 {
            font-size: 32px;
        }

        &.font-48 {
            font-size: 48px;
        }
    }
</style>
