<script lang="ts">
    import { afterUpdate, onMount, setContext } from "svelte";
    import { createCSSTemplate } from "./css";
    import isObject from "./isObject";
    import {
        currentFont,
        currentMode,
        currentThemeName,
        currentThemeObject
    } from "./store";
    import { allThemes } from "./themes";

    const STORAGE_KEY = "__svelte-themer__theme";
    const CONTEXT_KEY = "theme";
    const VARIABLE_PREFIX = "theme";
    const VALID_MODES = ["auto", "light", "dark"];
    const INVALID_THEMES_MESSAGE = "Invalid themes object supplied";
    const INVALID_PREFIX_MESSAGE = "Invalid prefix string supplied";
    const INVALID_MODE_MESSAGE = `Invalid mode string supplied, must be one of: ${VALID_MODES.join(
        ", "
    )}`;

    /**
     * Specify the key used for local storage
     * @type {string} [key='__svelte-themer__theme']
     */
    export let key = STORAGE_KEY;
    /**
     * Themes collection
     * @type {Object} themes - theme object
     */
    export let themes = allThemes;
    /**
     * Sets the specified theme as active
     * @type {string | null} [theme='dark']
     */
    export let theme = null;
    /**
     * Specify custom CSS variable prefix
     * @type {string | null} [prefix='theme']
     */
    export let prefix = VARIABLE_PREFIX;
    /**
     * Specify preferred theme mode
     * @type {"auto" | "dark" | "light"} [mode='auto']
     */
    export let mode = "auto";
    /**
     * Site default CSS variables
     * @type {Object} [base={}]
     */
    export let base = {};

    if (!isObject(themes) || !Object.keys(themes).length)
        throw new Error(INVALID_THEMES_MESSAGE);
    if (typeof prefix === "string" && !prefix.trim().length)
        throw new Error(INVALID_PREFIX_MESSAGE);
    if (!VALID_MODES.includes(mode)) throw new Error(INVALID_MODE_MESSAGE);

    const [fallback] = Object.keys(themes);

    let style;
    // create CSS
    $: if ($currentThemeObject && $currentFont) {
        style = createCSSTemplate(prefix, base);
    }

    setContext(CONTEXT_KEY, {
        current: currentThemeName,
        theme: currentThemeName
    });

    onMount(() => {
        // detect dark mode
        const darkSchemeQuery = matchMedia("(prefers-color-scheme: dark)");
        // determine the users preferred mode
        const preferredMode = darkSchemeQuery.matches ? "dark" : "light";
        // listen for media query status change
        darkSchemeQuery.addEventListener(
            "change",
            ({ matches }) =>
                mode === "auto" && currentMode.set(matches ? "dark" : "light")
        );

        // loading order: saved, prefers, fallback
        const saved = key ? localStorage && localStorage.getItem(key) : null;

        if (document) {
            document.documentElement.setAttribute(
                "data-theme",
                $currentThemeName
            );
        }
        if (key && localStorage) localStorage.setItem(key, $currentThemeName);

        return () =>
            key && localStorage && localStorage.setItem(key, $currentThemeName);
    });
</script>

<svelte:head>
    {@html style}
</svelte:head>

<slot>
    <!-- children -->
</slot>
