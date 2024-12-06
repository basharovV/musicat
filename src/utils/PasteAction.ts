export function capturePaste(node, callback) {
    // Handler for the paste event
    function handlePaste(event) {
        if (callback && typeof callback === "function") {
            callback(event);
        } else {
            console.warn("No callback provided to capturePaste action");
        }
    }

    // Attach the event listener
    node.addEventListener("paste", handlePaste);

    // Cleanup function when the action is destroyed
    return {
        destroy() {
            node.removeEventListener("paste", handlePaste);
        }
    };
}
