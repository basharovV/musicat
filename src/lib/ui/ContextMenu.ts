// contextMenu.ts
import type { ComponentType, SvelteComponent } from "svelte";
import { tick } from "svelte";

export interface ContextMenuOptions {
    component: ComponentType;
    props?: Record<string, any>;
}

export async function closeCurrentMenu() {
    // Close all existing
    const existingMenus = document.body.querySelectorAll(
        ".musicat-context-menu",
    );
    existingMenus?.forEach((el) => {
        document.body.removeChild(el);
    });
}

/**
 * Imperatively opens a context menu at a given mouse event position.
 * Closes all other context menus before opening the new one.
 */
export async function openContextMenu(
    event: MouseEvent,
    options: ContextMenuOptions,
) {
    event.preventDefault();
    event.stopPropagation();

    const { component: MenuComponent, props = {} } = options;

    // Close all existing
    closeCurrentMenu();

    const container = document.createElement("div");
    container.className = "musicat-context-menu";
    container.style.position = "fixed";

    container.style.top = `${event.clientY}px`;
    container.style.left = `${event.clientX}px`;

    container.style.zIndex = "9999";
    document.body.appendChild(container);

    let instance: SvelteComponent | null = null;

    const close = () => {
        instance?.$destroy();
        instance = null;
        container.remove();
    };

    instance = new MenuComponent({
        target: container,
        props: { ...props, close },
    });

    // TODO: Adjust position based on available space after component mounts
    await tick();
    await new Promise(requestAnimationFrame);
    const rect = container.firstChild?.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    console.log("rect", rect, "vw", vw, "vh", vh);

    if (rect.right > vw) {
        container.style.left = `${Math.max(0, vw - rect.width - 8)}px`;
    }
    if (rect.bottom > vh) {
        container.style.top = `${Math.max(0, vh - rect.height - 8)}px`;
    }

    const handleClick = (e: MouseEvent) => {
        if (!container.contains(e.target as Node)) close();
    };
    setTimeout(() =>
        window.addEventListener("click", handleClick, { once: true }),
    );

    return { close };
}

export function contextMenu(node: HTMLElement, options: ContextMenuOptions) {
    let current: { close: () => void } | null = null;

    async function handleContextMenu(event: MouseEvent) {
        current?.close();
        current = await openContextMenu(event, options);
    }

    node.addEventListener("contextmenu", handleContextMenu);

    return {
        destroy() {
            node.removeEventListener("contextmenu", handleContextMenu);
            current?.close();
        },
        update(newOptions: ContextMenuOptions) {
            options = newOptions;
        },
    };
}
