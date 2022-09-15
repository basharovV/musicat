<script lang="ts">
    export let fontSize;
    export let isFullScreen;
    export let lyrics = "";
</script>

<div class={fontSize} class:full-screen={isFullScreen}>
    <textarea placeholder="Start typing...">{lyrics}</textarea>
</div>

<style lang="scss">
    div {
        text-align: center;
        width: 100%;
        height: 100%;

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
