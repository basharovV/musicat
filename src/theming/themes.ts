type ThemeType = "dark" | "light";
export interface Theme {
    type: ThemeType;
    text: string;
    "text-secondary": string;
    "text-active": string;
    "text-inactive": string;
    background: string;
    accent: string;
    "accent-secondary": string;
    "icon-primary": string;
    "icon-secondary": string;
    "transport-controls": string;
    "transport-favorite": string;
    "transport-shuffle": string;
    "panel-background": string;
    "library-text-color": string;
    "library-playing-bg": string;
    "library-playing-text": string;
    "library-playing-icon": string;
    "library-highlight-bg": string;
    "library-hover-bg": string;
    "library-header-bg": string;
    "library-header-active-bg": string;
    "library-clickable-cell-bg": string;
    "library-clickable-cell-hover-bg": string;
    "library-favourite-icon": string;
    "smart-playlist-builder-bg": string;
    "smart-playlist-builder-block-bg": string;
    "smart-playlist-builder-block-focused-bg": string;
    "smart-playlist-builder-block-input-bg": string;
}
export const allFonts = {
    "2Peas": {
        "--font": "2Peas"
    }
};

export const DEFAULT_THEME: Theme = {
    "type": "dark",
    "text": "#ffffffde",
    "text-secondary": "#8f9093",
    "text-active": "#ffffff",
    "text-inactive": "#676464",
    "background": "#242026",
    "accent": "#45fffcf3",
    "accent-secondary": "#da69ff",
    "icon-primary": "#ffffff",
    "icon-secondary": "#474747",
    "transport-controls": "#ffffff",
    "transport-favorite": "#59cd7a",
    "transport-shuffle": "#e1ff00",
    "panel-background": "rgba(36, 33, 34, 0.948)",
    "library-text-color": "#d3d3d3",
    "library-playing-bg": "#5123dd",
    "library-playing-text": "#00ddff",
    "library-playing-icon": "#00ddff",
    "library-highlight-bg": "#2e3357",
    "library-hover-bg": "#1f1f1f",
    "library-header-bg": "#71658e7e",
    "library-header-active-bg": "#604d8d",
    "library-clickable-cell-bg": "#71658e1e",
    "library-clickable-cell-hover-bg": "#8c7dae36",
    "library-favourite-icon": "#5123dd",
    "smart-playlist-builder-bg": "#4d347c",
    "smart-playlist-builder-block-bg": "#7256be56",
    "smart-playlist-builder-block-focused-bg": "#7256be99",
    "smart-playlist-builder-block-input-bg": "#231b4d45"
};

export const allThemes: { [key: string]: Theme } = {
    light: {
        ...DEFAULT_THEME,
        "type": "light",
        "text": "#282230",
        "text-secondary": "#7388c6",
        "text-active": "#38353b",
        "text-inactive": "#707879",
        "background": "#f1f1f1",
        "accent": "#5d00fff3",
        "accent-secondary": "#da69ff",
        "icon-primary": "#2c0b3e",
        "icon-secondary": "#575151",
        "transport-controls": "#333135",
        "transport-favorite": "#49ab65",
        "transport-shuffle": "#9804d3",
        "panel-background": "#cfcfcf",
        "library-text-color": "#3e4040",
        "library-playing-bg": "#36a4b5",
        "library-playing-text": "#ffffff",
        "library-playing-icon": "#1a1718",
        "library-highlight-bg": "#9fb0b4",
        "library-hover-bg": "#c2cfd2",
        "library-header-bg": "#95afb37e",
        "library-header-active-bg": "#7bb1b7",
        "library-clickable-cell-bg": "#8ebac546",
        "library-clickable-cell-hover-bg": "#6fb1beae",
        "library-favourite-icon": "#5123dd",
        "smart-playlist-builder-bg": "#347c75",
        "smart-playlist-builder-block-bg": "#56beb556",
        "smart-playlist-builder-block-focused-bg": "#569abe99",
        "smart-playlist-builder-block-input-bg": "#1b474d6f"
    },
    dark: DEFAULT_THEME,
    amphibian: {
        "type": "dark",
        "text": "#b6e1e3de",
        "text-secondary": "#819be8",
        "text-active": "#83e4ff",
        "text-inactive": "#668e93",
        "background": "#233346",
        "accent": "#00ff62f3",
        "accent-secondary": "#da69ff",
        "icon-primary": "#ffffff",
        "icon-secondary": "#797676",
        "transport-controls": "#e9d6fb",
        "transport-favorite": "#59cd7a",
        "transport-shuffle": "#e1ff00",
        "panel-background": "#181e25",
        "library-text-color": "#8cb3b8",
        "library-playing-bg": "#23cedd",
        "library-playing-text": "#000000",
        "library-playing-icon": "#1a1718",
        "library-highlight-bg": "#2e4f57",
        "library-hover-bg": "#1d2a30",
        "library-header-bg": "#65888e7e",
        "library-header-active-bg": "#4d788d",
        "library-clickable-cell-bg": "#286e7f46",
        "library-clickable-cell-hover-bg": "#188da5ae",
        "library-favourite-icon": "#5123dd",
        "smart-playlist-builder-bg": "#347c75",
        "smart-playlist-builder-block-bg": "#56beb556",
        "smart-playlist-builder-block-focused-bg": "#569abe99",
        "smart-playlist-builder-block-input-bg": "#1b474d6f"
    },
    turquoise: {
        ...DEFAULT_THEME,
        "background": "#01796f",
        "text-inactive": "#b0b0b0"
    },
    red: {
        ...DEFAULT_THEME,
        "text": "#f1f1f1",
        "background": "#562931"
    }
};
