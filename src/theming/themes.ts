import type { Theme } from "../types/theme.d.ts";

import tmAmphibian from "./themes/amphibian.yaml";
import tmDark from "./themes/dark.yaml";
import tmLight from "./themes/light.yaml";
import tmLight2 from "./themes/light2.yaml";
import tmTurquoise from "./themes/turquoise.yaml";
import tmRed from "./themes/red.yaml";
import tmWinamp from "./themes/winamp.yaml";
import tmZokugunObsidium from "./themes/zokugun-obsidium.yaml";

export const allFonts = {
    "2Peas": {
        "--font": "2Peas",
    },
};

export const DEFAULT_THEME: Theme = tmDark;

export const allThemes: { [key: string]: Theme } = {
    amphibian: tmAmphibian,
    dark: tmDark,
    light: tmLight,
    light2: tmLight2,
    turquoise: tmTurquoise,
    red: tmRed,
    winamp: tmWinamp,
    "zokugun-obsidium": tmZokugunObsidium,
};

const base16 = import.meta.glob("./themes/base16/base16-*.yaml", {
    import: "default",
    eager: true,
});

for (const [path, theme] of Object.entries(base16)) {
    const key = /\.\/themes\/base16\/base16-(.*)\.yaml/.exec(path)[1];

    allThemes[`base16-${key}`] = theme as Theme;
}

export const lightThemes: { [key: string]: Theme } = {};
export const darkThemes: { [key: string]: Theme } = {};

for (const [key, theme] of Object.entries(allThemes).sort(([_a, a], [_b, b]) =>
    a["display-name"].localeCompare(b["display-name"]),
)) {
    if (theme.variant === "light") {
        lightThemes[key] = theme;
    } else {
        darkThemes[key] = theme;
    }
}
