import runTippy, { type Instance } from "tippy.js";
import type { Props } from "tippy.js";

/**
 * An interface of [Tippy.js Props](https://atomiks.github.io/tippyjs/v6/html-content/)
 */
export type TippyProps = Partial<Omit<Props, "trigger">> & {
    show?: boolean;
};

export interface TippyReturn {
    update: (newProps: TippyProps) => void;
    destroy: () => void;
}
export type Tippy = (element: HTMLElement, props?: TippyProps) => TippyReturn;
/**
 * Svelte action for rendering a [Tippy.JS](https://atomiks.github.io/tippyjs/) tooltip
 *
 * @example
 * ```svelte
 * <script lang="ts">
 *   import {tippy} from '$lib/tippy';
 *   import 'tippy.js/dist/tippy.css'; // optional
 * </script>
 * <button use:tippy={{content: 'Test'}}>Test</button>
```
 * @param element The element to target (omitted with use)
 * @param props [Tippy.js Props](https://atomiks.github.io/tippyjs/v6/all-props/)
 */
export const optionalTippy: Tippy = (element, props) => {
    let tippy: Instance;
    if (props) {
        if (props["show"] !== undefined) {
            if (props["show"]) {
                delete props["show"];
                tippy = runTippy(element, props);
            } else {
                // Do nothing
            }
        } else {
            tippy = runTippy(element, props);
        }
    }
    return {
        destroy() {
            if (tippy) {
                tippy.destroy();
            }
        },
        update(newProps) {
            if (newProps) {
                if (newProps["show"] !== undefined) {
                    if (!newProps["show"] && tippy) {
                        tippy.destroy();
                        return;
                    }
                }
                tippy = runTippy(element, newProps);
                tippy.setProps(newProps);
            } else if (!newProps || (!newProps["show"] && tippy)) {
                tippy.destroy();
            }
        },
    };
};
