<script>
    import { getContext } from "svelte";

    export let isDisabled = false;
    export let isDestructive = false;
    export let text = "";
    export let description = "";
    export let confirmText = "";
    export let isConfirming = false;
    export let isHighlighted = false;

    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();

    const { dispatchClick } = getContext("menu");

    const handleClick = (e) => {
        if (isDisabled) return;

        dispatch("click");
        dispatchClick();
    };

    $: {
        console.log("highlighted", isHighlighted);
    }
</script>

<div
    class:disabled={isDisabled}
    class:destructive={isDestructive}
    class:confirming={isConfirming}
    class:highlighted={isHighlighted}
    on:click={handleClick}
>
    {#if isConfirming}
        {confirmText}
    {:else if text}
        <span>
            <p>{text}</p>
            {#if description}
                <small>
                    {description}
                </small>
            {/if}
        </span>
    {:else}
        <slot />
    {/if}
</div>

<style lang="scss">
    div {
        padding: 0px 10px;
        cursor: default;
        font-size: 13.5px;
        display: flex;
        align-items: center;
        grid-gap: 5px;
        color: rgb(255, 255, 255);
        border-radius: 2.5px;
        &:hover,
        &.highlighted {
            background: #c6c8ca;
            color: rgb(37, 36, 36);
        }
        &.disabled {
            color: rgba(174, 174, 174, 0.4);
            &:hover {
                background: transparent;
            }
        }
        &.confirming {
            background: #d2630e;
            &:hover {
                background: #d2630e !important;
            }
        }
        &.destructive {
            &:hover {
                background: #d20e32;
                color: white;
            }
        }

        span {
            display: block;
            padding: 0.2em 0 0.3em 0;
            p {
                margin: 0;
                line-height: 20px;
            }

            small {
                margin: 0;
                line-height: 14px;
                display: block;
                opacity: 0.7;
            }
        }
    }
</style>
