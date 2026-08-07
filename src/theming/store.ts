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

function systemPrefersDarkTheme(): boolean {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function applyTheme(userSettings: UserSettings): Theme {
    // Priority order
    // 1. User selected theme
    // 2. Theme variant (auto, dark or light)

    const theme = userSettings.theme;
    const followSystemTheme = userSettings.followSystemTheme;

    let themeObject: Theme;

    // "auto" isn't a real entry in `allThemes` (it's the settings default from
    // `settings.rs`'s `default_theme()`, meaning "no explicit theme chosen yet") -- so it, and
    // any other unrecognized theme name, falls back to matching system preference, same as
    // `followSystemTheme` does. Previously this looked up `allThemes["auto"]`, found nothing,
    // and then unconditionally dereferenced the not-found result a few lines down, throwing.
    if (followSystemTheme || !theme || theme === "auto") {
        themeObject = systemPrefersDarkTheme()
            ? DEFAULT_THEME_DARK
            : DEFAULT_THEME_LIGHT;
    } else {
        const foundTheme = Object.entries(allThemes).find(
            (t) => t[0] === theme,
        );
        if (foundTheme) {
            themeObject = foundTheme[1];
        } else {
            console.error("Theme not found", theme);
            themeObject = systemPrefersDarkTheme()
                ? DEFAULT_THEME_DARK
                : DEFAULT_THEME_LIGHT;
        }
        tauriWindow.getCurrentWindow().setTheme(themeObject.variant);
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

    // `--panel-background`/`--panel-primary-border-accent1`/`--panel-primary-border-accent2`/
    // `--panel-separator`/`--panel-primary-border-main`/`--panel-secondary-border-main` are
    // referenced across many views (Artist's Toolkit, Sidebar, Settings, LyricsView, MapView,
    // AnalyticsView, InternetArchiveView, AudioAnalyzer) but were never actually part of the
    // `Theme` schema (see `types/theme.d.ts`) or any theme `.yaml` -- `var(--undefined)` with no
    // fallback makes the whole `border`/`background-color` declaration invalid, so those borders
    // never rendered in *any* theme. Aliased to this theme's own `panel`/`border` here (after
    // derived-value resolution, so these are always plain literals) rather than editing every
    // theme file individually. There are other undefined variables in other, unrelated views
    // (menus, popups, buttons, sidebar item states, etc.) not covered by this -- a much larger
    // pre-existing gap, out of scope here.
    const extraPanelVars = themeObject as unknown as Record<string, string>;
    extraPanelVars["panel-background"] = themeObject.panel;
    extraPanelVars["panel-primary-border-accent1"] = themeObject.border;
    extraPanelVars["panel-primary-border-accent2"] = themeObject.border;
    extraPanelVars["panel-separator"] = themeObject.border;
    extraPanelVars["panel-primary-border-main"] = themeObject.border;
    extraPanelVars["panel-secondary-border-main"] = themeObject.border;

    // Apply data-variant attribute
    document.documentElement.setAttribute("data-variant", themeObject.variant);

    return themeObject;
}
