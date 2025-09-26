// clickOutside.ts
export function clickOutside(
    element: HTMLElement,
    options: {
        callbackFunction: () => void;
        stopPropagation?: boolean;
    },
) {
    function onClick(event: MouseEvent) {
        if (!element.contains(event.target as Node)) {
            if (options?.stopPropagation) {
                event.stopPropagation();
            }
            options?.callbackFunction();
        }
    }

    // Delay to avoid catching the initiating click
    setTimeout(() => {
        document.addEventListener("click", onClick);
        document.addEventListener("contextmenu", onClick);
    }, 0);

    return {
        update(newCallbackFunction: () => void) {
            options.callbackFunction = newCallbackFunction;
        },
        destroy() {
            document.removeEventListener("click", onClick);
            document.removeEventListener("contextmenu", onClick);
        },
    };
}
