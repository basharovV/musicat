import sveltePreprocess from "svelte-preprocess";
import mdsvexConfig from "./mdsvex.config.js";
import { mdsvex } from "mdsvex";

export default {
    extensions: [".svelte", ...mdsvexConfig.extensions],

    // Consult https://github.com/sveltejs/svelte-preprocess
    // for more information about preprocessors
    preprocess: [sveltePreprocess(), mdsvex(mdsvexConfig)],

    onwarn: (warning, handler) => {
        if (warning.code.startsWith("a11y-") || warning.code === "css-unused-selector") {
            return;
        }
        handler(warning);
    }
};
