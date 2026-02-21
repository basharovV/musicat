<!-- PortalMenu.svelte -->
<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { portal } from "../Portal";
    import Menu from "./Menu.svelte";
    import type { MenuItem, MenuSection } from "../../../App";

    export let anchor: HTMLElement;
    export let items: MenuItem[] = [];
    export let placeholder: string;
    export let sections: MenuSection[] = null;
    export let onItemSelected = null;
    export let onClose: () => void;

    let x = 0;
    let y = 0;
    let menuComponent: Menu;
    let containerEl: HTMLDivElement;

    function reposition() {
        if (!anchor || !containerEl) return;
        const anchorRect = anchor.getBoundingClientRect();
        const menuRect = containerEl.getBoundingClientRect();
        const gap = 4;

        let newX = anchorRect.left;
        let newY = anchorRect.bottom + gap;

        // Flip upward if clipped
        if (newY + menuRect.height > window.innerHeight) {
            newY = anchorRect.top - menuRect.height - gap;
        }
        // Prevent right overflow
        if (newX + menuRect.width > window.innerWidth) {
            newX = anchorRect.right - menuRect.width;
        }

        x = newX;
        y = newY;
    }

    onMount(() => {
        reposition();
        window.addEventListener("resize", reposition);
        window.addEventListener("scroll", reposition, true); // capture scroll anywhere
    });

    onDestroy(() => {
        window.removeEventListener("resize", reposition);
        window.removeEventListener("scroll", reposition, true);
    });
</script>

<div bind:this={containerEl} use:portal={"body"} style="position:contents">
    <Menu
        bind:this={menuComponent}
        {items}
        {sections}
        {x}
        {y}
        fixed={true}
        position="manual"
        onItemSelected={(item) => {
            onItemSelected?.(item);
            onClose();
        }}
        onClickOutside={onClose}
        maxHeight={200}
        minWidth={210}
        scrollable
        {placeholder}
    ></Menu>
</div>

<style lang="scss">
</style>
