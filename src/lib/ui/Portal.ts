// utils/portal.ts
export function portal(
    node: HTMLElement,
    target: HTMLElement | string = "body",
) {
    let targetEl: HTMLElement;

    function update(newTarget: HTMLElement | string) {
        targetEl =
            typeof newTarget === "string"
                ? document.querySelector(newTarget)
                : newTarget;
        targetEl.appendChild(node);
    }

    function destroy() {
        node.remove();
    }

    update(target);
    return { update, destroy };
}
