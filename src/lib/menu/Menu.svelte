<script lang="ts">
    import { onDestroy } from "svelte";
    import { fade } from "svelte/transition";
    import type { MenuItem, MenuSection } from "../../App";
    import { clickOutside } from "../../utils/ClickOutside";
    import MenuOption from "./MenuOption.svelte";

    export let x = 0;
    export let y = 0;
    export let fixed = false;
    export let items: MenuItem[] = [];
    export let onItemSelected = null;
    export let onClickOutside = null;
    export let position: "auto" | "manual" | "relative" = "auto";
    export let maxHeight: number = null; // If this exists, we wrap the contents
    export let padding = 0;
    export let sections: MenuSection[] = null;

    export let hoveredSection = sections !== null ? 0 : null;

    let hoveredItemIdx = 0;
    $: numberOfItems =
        hoveredSection !== null
            ? sections[hoveredSection].items.length
            : items.length;

    function onKeyUp() {
        if (hoveredItemIdx > 0) {
            hoveredItemIdx--;
        } else {
            hoveredItemIdx = 0;
        }
    }

    function onKeyDown() {
        if (hoveredItemIdx < numberOfItems - 1) {
            hoveredItemIdx = hoveredItemIdx + 1;
        }
    }

    function onKeyLeft() {
        if (sections && hoveredSection > 0) {
            // Check if other sections has this index
            const sectionToSelect = sections[hoveredSection - 1];
            // length 14 < access 15 ok
            // length 13 < acess 15 out of bounds
            if (sectionToSelect.items.length < hoveredItemIdx - 1) {
                // Fallback to last index
                hoveredItemIdx = sections[hoveredSection - 1].items.length - 1;
            }
            hoveredSection = hoveredSection - 1;
        } else if (sections) {
            hoveredItemIdx = 0;
        }
    }

    function onKeyRight() {
        if (sections?.length && hoveredSection < sections.length) {
            // Check if other sections has this index
            const sectionToSelect = sections[hoveredSection + 1];
            // length 14 < access 15 ok
            // length 13 < acess 15 out of bounds
            if (sectionToSelect.items.length < hoveredItemIdx - 1) {
                // Fallback to last index
                hoveredItemIdx = sections[hoveredSection - 1].items.length - 1;
            }
            hoveredSection = hoveredSection + 1;
        }
    }

    function onEnter() {
        // Select item callback
        if (sections?.length) {
            onItemSelected &&
                onItemSelected(sections[hoveredSection].items[hoveredItemIdx]);
        } else {
            onItemSelected && onItemSelected(items[hoveredItemIdx].source);
        }
    }

    // whenever x and y is changed, restrict box to be within bounds
    $: (() => {
        if (!menuEl || position === "manual") return;

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
        } else if (event.keyCode === 37) {
            // left
            event.preventDefault();
            onKeyLeft();
        } else if (event.keyCode === 39) {
            // right
            event.preventDefault();
            onKeyRight();
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

    if (items.length > 0 || sections?.length > 0) {
        /**
         * Keyboard navigation only works when passing in list of items, instead of using slot
         */
        addEventListener("keydown", onKeyPressed);
    }

    onDestroy(() => {
        removeEventListener("keydown", onKeyPressed);
    });
</script>

<menu
    class:fixed
    class:relative={position === "relative"}
    transition:fade={{ duration: 100 }}
    bind:this={menuEl}
    use:clickOutside={() => {
        onClickOutside && onClickOutside();
    }}
    style="top: {y}px; left: {x}px;gap: {padding}px;{maxHeight
        ? `max-height: ${maxHeight}px`
        : ''}"
>
    {#if sections}
        <div class="sections">
            {#each sections as section, sectionIdx}
                <div class="section">
                    <p>{section.title}</p>
                    <div class="items">
                        {#each section.items as item, idx}
                            <MenuOption
                                text={item.text}
                                description={item.description}
                                isHighlighted={hoveredSection === sectionIdx &&
                                    hoveredItemIdx === idx}
                                color={item.color}
                                onClick={() => {
                                    onItemSelected && onItemSelected(item);
                                }}
                            />
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {:else}
        <div class="items">
            {#each items as item, idx}
                <MenuOption
                    text={item.text}
                    description={item.description}
                    isHighlighted={hoveredItemIdx === idx}
                    color={item.color}
                    onClick={() => {
                        onItemSelected && onItemSelected(item.source);
                    }}
                />
            {:else}
                <slot />
            {/each}
        </div>
    {/if}
</menu>

<style lang="scss">
    menu {
        position: absolute;
        display: flex;
        flex-direction: column;
        flex-wrap: wrap;
        border: 1px solid var(--menu-border);
        border-radius: 8px;
        background-color: var(--menu-bg);
        backdrop-filter: blur(8px);
        padding: 3px;
        font-weight: 400;
        box-shadow: 10px 10px 10px 0px var(--menu-shadow);
        z-index: 22;
        max-width: 300px;
        min-width: 65px;
        scale: 0.97;
        font-family: system-ui, Avenir, Helvetica, Arial, sans-serif;
        &.fixed {
            position: fixed;
        }
        &.relative {
            position: relative;
        }
    }

    .sections {
        display: flex;
        flex-direction: row;
        gap: 10px;
        padding: 0.2em 0.3em;

        .section {
            > p {
                opacity: 0.5;
                margin: 0.2em 0;
                text-align: center;
            }
            .items {
                min-width: 50px;
            }
        }
    }
</style>
