import { derived, writable } from "svelte/store";
import type { Readable, Writable } from "svelte/store";
import { allFonts, allThemes, DEFAULT_THEME, type Theme } from "./themes";
import { userSettings } from "../data/store";

export const currentMode = writable();
export const currentThemeName: Readable<string> = derived(
    userSettings,
    (u) => u.theme
);
export const currentFontName: Writable<string> = writable("2Peas");
export const currentFont: Readable<Object> = derived(
    currentFontName,
    (n) => allFonts[n]
);
export const currentThemeObject: Readable<Theme> = derived(
    currentThemeName,
    (n) => allThemes[n] ?? DEFAULT_THEME
);
