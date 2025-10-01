export function isInputFocused() {
    return (
        document.activeElement.tagName.toLowerCase() === "input" ||
        document.activeElement.tagName.toLowerCase() === "textarea"
    );
}
