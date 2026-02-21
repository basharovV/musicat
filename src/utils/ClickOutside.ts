// clickOutside.ts
export function clickOutside(
    element: HTMLElement,
    callbackFunction: () => void,
) {
    function onClick(event: MouseEvent) {
        if (!document.contains(element)) return; // guard stale element
        if (!element.contains(event.target as Node)) {
            callbackFunction && callbackFunction();
        }
    }

    let frameId: number;
    frameId = requestAnimationFrame(() => {
        document.addEventListener("click", onClick);
        document.addEventListener("contextmenu", onClick);
    });

    return {
        update(newCallbackFunction: () => void) {
            callbackFunction = newCallbackFunction;
        },
        destroy() {
            cancelAnimationFrame(frameId); // cancel if destroyed before frame fires
            document.removeEventListener("click", onClick);
            document.removeEventListener("contextmenu", onClick);
        },
    };
}
