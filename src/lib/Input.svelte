<script>
    export let value;
    export let onChange = (evt) => {};
    export let fullWidth = false;
    export let autoCompleteValue = null;
    export let onEnterPressed = null;

    function onKeyDown(evt) {
        if (evt.keyCode === 13) {
            if (onEnterPressed) {
                onEnterPressed();
            }
        }
    }
</script>

<div>
    <input bind:value on:input={onChange} class:full-width={fullWidth} on:keydown={onKeyDown}/>

    {#if autoCompleteValue}
        <p class="autocomplete">{autoCompleteValue}</p>
    {/if}
</div>

<style lang="scss">
    div {
        position: relative;
    }
    input {
        line-height: 0.9em;
        padding: 0.3em;
        background-color: rgba(53, 48, 48, 0.349);
        border: 1px solid rgb(81, 76, 76);
        border-radius: 2px;
        font-size: 14px;
        z-index: 1;
        &:focus {
            background-color: rgba(102, 92, 92, 0.104);
        }
        &.full-width {
            width: 100%;
        }
    }

    .autocomplete {
        position: absolute;
        pointer-events: none;
        left: 6px;
        right: 0px;
        bottom: 0px;
        top: 0px;
        max-width: 100%;
        text-overflow: ellipsis;
        white-space: nowrap;
        overflow: hidden;
        margin: auto 0;
        height: fit-content;
        vertical-align: middle;
        line-height: normal;
        text-align: left;
        user-select: none;
        cursor: default;
        opacity: 0.5;
        z-index: 0;
    }
</style>
