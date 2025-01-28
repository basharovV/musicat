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

export const lightThemes = {
    light: tmLight,
    light2: tmLight2,
};

export const darkThemes = {
    amphibian: tmAmphibian,
    dark: tmDark,
    turquoise: tmTurquoise,
    red: tmRed,
    winamp: tmWinamp,
    "zokugun-obsidium": tmZokugunObsidium,
};
