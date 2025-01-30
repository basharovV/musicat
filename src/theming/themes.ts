import type { Theme } from "../types/theme.d.ts";

import tmDark from "./themes/dark.yaml";
import tmLight from "./themes/light.yaml";

export const allFonts = {
    "2Peas": {
        "--font": "2Peas",
    },
};

export const DEFAULT_THEME_DARK: Theme = tmDark;
export const DEFAULT_THEME_LIGHT: Theme = tmLight;

export const allThemes: { [key: string]: Theme } = {
    dark: tmDark,
    light: tmLight,
};

const cores: { [key: string]: Theme } = import.meta.glob("./themes/*.yaml", {
    import: "default",
    eager: true,
});

for (const [path, theme] of Object.entries(cores)) {
    const key = /\.\/themes\/(.*)\.yaml/.exec(path)[1];

    if (theme.variant === "dark") {
        allThemes[key] = { ...tmDark, ...theme };
    } else {
        allThemes[key] = { ...tmLight, ...theme };
    }
}

const base16: { [key: string]: Theme } = import.meta.glob(
    "./themes/base16/base16-*.yaml",
    {
        import: "default",
        eager: true,
    },
);

for (const [path, theme] of Object.entries(base16)) {
    const key = /\.\/themes\/base16\/base16-(.*)\.yaml/.exec(path)[1];

    allThemes[`base16-${key}`] = theme;
}

export const lightThemes: { [key: string]: Theme } = {};
export const darkThemes: { [key: string]: Theme } = {};

for (const [key, theme] of Object.entries(allThemes).sort(([_a, a], [_b, b]) =>
    a["display-name"].localeCompare(b["display-name"]),
)) {
    if (theme.variant === "dark") {
        darkThemes[key] = theme;
    } else {
        lightThemes[key] = theme;
    }
}
