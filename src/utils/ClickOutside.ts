export function clickOutside(element, callbackFunction) {
    function onClick(event) {
        if (!element.contains(event.target)) {
            console.log('clicked ', event.target, element);
            callbackFunction();
        }
    }
    
    document.body.addEventListener('click', onClick);
    
    return {
        update(newCallbackFunction) {
            callbackFunction = newCallbackFunction;
        },
        destroy() {
            document.body.removeEventListener('click', onClick);
        }
    }
}