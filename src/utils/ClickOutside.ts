function isElementInsideBounds(element1, element2) {
    const rect1 = element1.getBoundingClientRect();
    const rect2 = element2.getBoundingClientRect();
    console.log("rect1", rect1, "rect2", rect2);
    return (
        rect1.top >= rect2.top &&
        rect1.right <= rect2.right &&
        rect1.bottom <= rect2.bottom &&
        rect1.left >= rect2.left
    );
}

export function clickOutside(element, callbackFunction) {
    function onClick(event) {
        if (!isElementInsideBounds(event.target, element)) {
            console.log("clicked ", event.target, element);
            callbackFunction();
        }
    }

    // For some reason the click event from the button that shows the component is fired here
    // so we need to wait before adding a listener.
    setTimeout(() => {
        document.body.addEventListener("click", onClick);
        document.body.addEventListener("contextmenu", onClick);
    }, 0);

    return {
        update(newCallbackFunction) {
            callbackFunction = newCallbackFunction;
        },
        destroy() {
            document.body.removeEventListener("click", onClick);
            document.body.removeEventListener("contextmenu", onClick);
        },
    };
}
