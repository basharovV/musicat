export function clickOutside(element, callbackFunction) {
    function onClick(event) {
        if (!element.contains(event.target)) {
            console.log('clicked ', event.target, element);
            callbackFunction();
        }
    }
    
    // For some reason the click event from the button that shows the component is fired here
    // so we need to wait before adding a listener. 
    setTimeout(() => {
        document.body.addEventListener('click', onClick);
    }, 0);
    
    return {
        update(newCallbackFunction) {
            callbackFunction = newCallbackFunction;
        },
        destroy() {
            document.body.removeEventListener('click', onClick);
        }
    }
}