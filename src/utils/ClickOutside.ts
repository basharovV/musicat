export function clickOutside(element, callbackFunction) {
    function onClick(event) {
        if (!element.contains(event.target)) {
            callbackFunction();
        }
    }

    // For some reason the click event from the button that shows the component is fired here
    // so we need to wait before adding a listener.
    setTimeout(() => {
        document.addEventListener("click", onClick);
        document.addEventListener("contextmenu", onClick);
    }, 0);

    return {
        update(newCallbackFunction) {
            callbackFunction = newCallbackFunction;
        },
        destroy() {
            document.removeEventListener("click", onClick);
            document.removeEventListener("contextmenu", onClick);
        },
    };
}
