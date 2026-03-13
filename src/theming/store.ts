import type { Readable, Writable } from "svelte/store";
import { derived, writable } from "svelte/store";
import { userSettings } from "../data/store";
import {
    allFonts,
    allThemes,
    DEFAULT_THEME_DARK,
    DEFAULT_THEME_LIGHT,
    type Theme,
} from "./themes";
import type { UserSettings } from "../App";

export const currentMode = writable();

export const currentThemeName: Readable<string> = derived(
    userSettings,
    (u) => u.theme,
);
export const currentFontName: Writable<string> = writable("2Peas");
export const currentFont: Readable<Object> = derived(
    currentFontName,
    (n) => allFonts[n],
);
export const currentThemeObject: Readable<Theme> = derived(
    userSettings,
    (n) => {
        return applyTheme(n);
    },
);

export function applyTheme(userSettings: UserSettings): Theme {
    // Priority order
    // 1. User selected theme
    // 2. Theme variant (auto, dark or light)

    const theme = userSettings.theme;
    const followSystemTheme = userSettings.followSystemTheme;

    if (followSystemTheme) {
        console.log(
            "matchMedia",
            window.matchMedia("(prefers-color-scheme: dark)").matches,
        );
        return window.matchMedia("(prefers-color-scheme: dark)").matches
            ? DEFAULT_THEME_DARK
            : DEFAULT_THEME_LIGHT;
    } else if (theme) {
        const foundTheme = Object.entries(allThemes).find(
            (t) => t[0] === theme,
        );
        console.log("foundTheme", foundTheme, allThemes);
        return foundTheme ? foundTheme[1] : DEFAULT_THEME_DARK;
    } else {
        return DEFAULT_THEME_DARK;
    }
}
