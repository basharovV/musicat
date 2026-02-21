export function animateHeight(node: HTMLElement) {
    function getChildDimensions() {
        const child = node.firstElementChild as HTMLElement;
        if (!child) return { width: 0, height: 0 };

        // Temporarily unconstrain to get natural dimensions
        const prevWidth = node.style.width;
        node.style.width = "fit-content";

        const style = getComputedStyle(child);
        const dims = {
            width:
                child.offsetWidth +
                parseFloat(style.marginLeft) +
                parseFloat(style.marginRight),
            height:
                child.offsetHeight +
                parseFloat(style.marginTop) +
                parseFloat(style.marginBottom),
        };

        // Restore
        node.style.width = prevWidth;

        return dims;
    }
    function update() {
        const dims = getChildDimensions();
        const parentTop = node.clientTop;
        // Restore transition and apply new dimensions in next frame
        requestAnimationFrame(() => {
            node.style.width = dims.width + "px";
            node.style.height = dims.height + "px";
        });
    }
    const resizeObserver = new ResizeObserver(update);

    const mutationObserver = new MutationObserver(() => {
        resizeObserver.disconnect();
        if (node.firstElementChild) {
            resizeObserver.observe(node.firstElementChild);
        }
        update();
    });

    mutationObserver.observe(node, { childList: true });
    if (node.firstElementChild) resizeObserver.observe(node.firstElementChild);
    update();

    return {
        destroy: () => {
            resizeObserver.disconnect();
            mutationObserver.disconnect();
        },
    };
}
