import type { Readable, Writable } from "svelte/store";
import { derived, writable } from "svelte/store";
import { userSettings } from "../data/store";
import {
    allFonts,
    allThemes,
    DEFAULT_THEME_DARK,
    DEFAULT_THEME_LIGHT,
} from "./themes";
import type { UserSettings } from "../App";
import { window as tauriWindow } from "@tauri-apps/api";
import type { Theme } from "../types/theme";
import { resolveDerived } from "./ColorUtils";

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

    let themeObject: Theme;

    if (followSystemTheme) {
        console.log(
            "matchMedia",
            window.matchMedia("(prefers-color-scheme: dark)").matches,
        );
        themeObject = window.matchMedia("(prefers-color-scheme: dark)").matches
            ? DEFAULT_THEME_DARK
            : DEFAULT_THEME_LIGHT;
    } else if (theme) {
        const foundTheme = Object.entries(allThemes).find(
            (t) => t[0] === theme,
        );
        if (!foundTheme) {
            console.error("Theme not found", theme);
            tauriWindow.getCurrentWindow().setTheme("dark");
            themeObject = DEFAULT_THEME_DARK;
        }
        tauriWindow.getCurrentWindow().setTheme(foundTheme[1].variant);

        themeObject = foundTheme ? foundTheme[1] : DEFAULT_THEME_DARK;
    } else {
        themeObject = DEFAULT_THEME_DARK;
    }

    // Rewrite css mix / alpha to 8-char hex

    // All values not containing other variables are considered core
    const core = Object.fromEntries(
        Object.entries(themeObject).filter(
            ([, value]) =>
                typeof value === "string" && !value.includes("var(--"),
        ),
    );

    // Resolve derived values
    Object.entries(themeObject).forEach(([key, value]) => {
        if (typeof value === "string" && value.includes("var(--")) {
            console.log("resolving derived", value);
            themeObject[key] = resolveDerived(value, core, themeObject.variant);
        }
    });

    // Apply data-variant attribute
    document.documentElement.setAttribute("data-variant", themeObject.variant);

    return themeObject;
}
