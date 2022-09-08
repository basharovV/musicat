<script>
    export let value;
    export let onChange = () => {};
    export let fullWidth = false;
</script>

<input bind:value on:input={onChange} class:full-width={fullWidth} />

<style lang="scss">
    input {
        line-height: 0.9em;
        padding: 0.3em;
        background-color: rgba(53, 48, 48, 0.349);
        border: 1px solid rgb(81, 76, 76);
        border-radius: 2px;
        font-size: 14px;
        &:focus {
            background-color: rgba(102, 92, 92, 0.349);
        }
        &.full-width {
            width: 100%;
        }
    }
</style>
