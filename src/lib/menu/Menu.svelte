<script lang="ts">
    import { onDestroy } from "svelte";
    import { fade } from "svelte/transition";
    import { clickOutside } from "../../utils/ClickOutside";
    import MenuOption from "./MenuOption.svelte";

    interface MenuItem {
        text: string;
        description: string;
        source: any;
    }

    export let x;
    export let y;
    export let items: MenuItem[] = [];
    export let onItemSelected = null;
    export let onClickOutside = null;
    export let position: 'auto'|'manual' = 'auto';

    let hoveredItemIdx = 0;
    $: numberOfItems = items.length;

    function onKeyUp() {
        console.log("keyup");

        if (hoveredItemIdx > 0) {
            hoveredItemIdx--;
        } else {
            hoveredItemIdx = 0;
        }
    }

    function onKeyDown() {
        console.log("keydown");
        console.log("items", numberOfItems);
        if (hoveredItemIdx < numberOfItems - 1) {
            hoveredItemIdx = hoveredItemIdx + 1;
            console.log("idx:", hoveredItemIdx);
        }
    }

    function onEnter() {
        // Select item callback
        onItemSelected && onItemSelected(items[hoveredItemIdx].source);
    }

    // whenever x and y is changed, restrict box to be within bounds
    $: (() => {
        if (!menuEl || position === 'manual') return;

        const rect = menuEl.getBoundingClientRect();
        x = Math.min(window.innerWidth - rect.width - 30, x);
        if (y > window.innerHeight - rect.height) y -= rect.height;
    })();

    let menuEl;

    function onKeyPressed(event) {
        if (event.keyCode === 38) {
            event.preventDefault();
            // up
            onKeyUp();
        } else if (event.keyCode === 40) {
            // down
            event.preventDefault();
            onKeyDown();
        } else if (event.keyCode === 13) {
            // 'Enter' to select
            event.preventDefault();
            onEnter();
            console.log("onEnter");
        } else if (event.keyCode === 27) {
            // 'Esc' to close
            event.preventDefault();
            onClickOutside && onClickOutside();
        }
    }

    if (items.length > 0) {
        /**
         * Keyboard navigation only works when passing in list of items, instead of using slot
         */
        addEventListener("keydown", onKeyPressed);
    }

    onDestroy(() => {
        removeEventListener("keydown", onKeyPressed);
    });
</script>

<div
    transition:fade={{ duration: 100 }}
    bind:this={menuEl}
    use:clickOutside={() => {
        onClickOutside && onClickOutside();
    }}
    style="top: {y}px; left: {x}px;"
>
    {#each items as item, idx}
        <MenuOption
            text={item.text}
            description={item.description}
            isHighlighted={hoveredItemIdx === idx}
            onClick={() => {
                onItemSelected && onItemSelected(item.source);
            }}
        />
    {:else}
        <slot />
    {/each}
</div>

<style>
    div {
        position: absolute;
        display: grid;
        border: 1px solid rgba(255, 255, 255, 0.23);
        box-shadow: 2px 2px 5px 0px #0002;
        border-radius: 8px;
        background-color: #20202296;
        backdrop-filter: blur(8px);
        padding: 3px;
        font-weight: 400;
        box-shadow: 10px 10px 10px 0px #0002;
        z-index: 20;
        max-width: 300px;
        font-family: system-ui, Avenir, Helvetica, Arial, sans-serif;
    }
</style>
