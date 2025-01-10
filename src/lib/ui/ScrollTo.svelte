<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fly } from "svelte/transition";

    export let equalizer = false;

    const dispatch = createEventDispatcher();

    function handleClick() {
        dispatch("click");
    }
</script>

<div
    in:fly={{ duration: 150, y: 30 }}
    out:fly={{ duration: 150, y: 30 }}
    class="scroll-now-playing"
    on:click={handleClick}
>
    {#if equalizer}
        <div class="eq">
            <span class="eq1" />
            <span class="eq2" />
            <span class="eq3" />
        </div>
    {/if}
    <p><slot /></p>
</div>

<style lang="scss">
    .scroll-now-playing {
        position: absolute;
        bottom: 0.5em;
        left: 0;
        right: 0;
        padding: 0.5em 1em;
        border-radius: 10px;
        background-color: var(--scrollto-bg);
        border: 1px solid var(--scrollto-border);
        box-shadow: 10px 10px 10px var(--scrollto-shadow);
        color: var(--scrollto-text);
        margin: auto;
        width: fit-content;
        z-index: 11;
        display: inline-flex;
        align-items: center;
        gap: 10px;
        cursor: default;
        user-select: none;

        @media only screen and (max-width: 522px) {
            display: none;
        }
        &:hover {
            background-color: var(--scrollto-hover-bg);
            border: 1px solid var(--scrollto-hover-border);
            box-shadow: 10px 10px 10px var(--scrollto-hover-shadow);
        }
        &:active {
            background-color: var(--scrollto-active-bg);
            border: 2px solid var(--scrollto-active-border);
            box-shadow: 10px 10px 10px var(--scrollto-active-shadow);
        }

        .eq {
            width: 15px;
            padding: 0.5em;
            position: relative;

            span {
                display: inline-block;
                width: 3px;
                background-color: var(--scrollto-eq);
                position: absolute;
                bottom: 0;
            }

            .eq1 {
                height: 13px;
                left: 0;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0s;
            }

            .eq2 {
                height: 15px;
                left: 6px;
                animation-name: talleq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.17s;
            }

            .eq3 {
                height: 13px;
                left: 12px;
                animation-name: shorteq;
                animation-duration: 0.5s;
                animation-iteration-count: infinite;
                animation-delay: 0.34s;
            }
        }
        p {
            margin: 0;
        }
    }

    @keyframes shorteq {
        0% {
            height: 10px;
        }
        50% {
            height: 5px;
        }
        100% {
            height: 10px;
        }
    }
    @keyframes talleq {
        0% {
            height: 15px;
        }
        50% {
            height: 8px;
        }
        100% {
            height: 15px;
        }
    }
</style>
