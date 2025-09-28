// clickOutside.ts
export function clickOutside(
    element: HTMLElement,
    callbackFunction: () => void,
) {
    function onClick(event: MouseEvent) {
        if (!element.contains(event.target as Node)) {
            callbackFunction && callbackFunction();
        }
    }

    // Delay to avoid catching the initiating click
    setTimeout(() => {
        document.addEventListener("click", onClick);
        document.addEventListener("contextmenu", onClick);
    }, 0);

    return {
        update(newCallbackFunction: () => void) {
            callbackFunction = newCallbackFunction;
        },
        destroy() {
            document.removeEventListener("click", onClick);
            document.removeEventListener("contextmenu", onClick);
        },
    };
}
