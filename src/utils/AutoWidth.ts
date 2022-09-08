function measureMyInputText(input) {
    var c = document.createElement("canvas");
    var ctx = c.getContext("2d");
    ctx.font = font(input);
    var txtWidth = ctx.measureText(input.value).width;

    return txtWidth;
}

function font(element) {
    var prop = ["font-style", "font-variant", "font-weight", "font-size", "font-family"];
    var font = "";
    for (var x in prop)
        font += window.getComputedStyle(element, null).getPropertyValue(prop[x]) + " ";

    return font;
}

export const autoWidth = (node) => {
    /* Constants */
    const update = new Event("update");
    const buffer = 7;
  
    /* Functions */
    const init = () => {
      addStyles();
      observeElement();
      addEventListeners();
      setInitialWidth();
    };
  
    const dispatchUpdateEvent = () => {
      node.dispatchEvent(update);
    };
  
    const setInitialWidth = () => {
      let width;
  
      if (node.placeholder && !node.value) {
        node.value = node.placeholder;
        node.style.width = "0px";
        width = node.scrollWidth;
        node.value = "";
      } else {
        node.style.width = "0px";
        width = node.scrollWidth;
      }
  
      node.style.width = width + buffer + "px";
    };
  
    const setWidth = () => {
      node.style.width = "0px";
      node.style.width = measureMyInputText(node) + buffer + "px";
    };
  
    const addStyles = () => {
      node.style.boxSizing = "border-box";
    };
  
    const observeElement = () => {
      let elementPrototype = Object.getPrototypeOf(node);
      let descriptor = Object.getOwnPropertyDescriptor(elementPrototype, "value");
      Object.defineProperty(node, "value", {
        get: function () {
          return descriptor.get.apply(this, arguments);
        },
        set: function () {
          descriptor.set.apply(this, arguments);
          dispatchUpdateEvent();
        },
      });
    };
  
    const addEventListeners = () => {
      node.addEventListener("input", () => {
        dispatchUpdateEvent();
      });
      node.addEventListener("update", setWidth);
    };
  
    const removeEventListeners = () => {
      node.removeEventListener("input", dispatchUpdateEvent);
      node.removeEventListener("update", setWidth);
    };
  
    if (node.tagName.toLowerCase() !== "input") {
      throw new Error(
        "svelte-input-auto-width can only be used on input elements."
      );
    } else {
      init();
  
      return {
        destroy() {
          removeEventListeners();
        },
      };
    }
  };
  