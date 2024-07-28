type ThemeType = "dark" | "light";
export interface Theme {
    type: ThemeType;
    /**  Primary text color */
    text: string;
    /**  Secondary text color, for dimmed text */
    "text-secondary": string;
    /**  Text color for active items */
    "text-active": string;
    /**  Text color for inactive items */
    "text-inactive": string;
    /**  Window background (will be made more translucent on macOS) */
    background: string;
    /**  Overlays, popups, menus */
    "overlay-bg": string;
    /**  Primary accent color used for active items that need to stand out */
    accent: string;
    "accent-secondary": string;
    "icon-primary": string;
    "icon-secondary": string;
    "button-bg": string;
    "button-text": string;
    "input-bg": string;
    /** Play, pause, next, previous */
    "transport-controls": string;
    "transport-favorite": string;
    "transport-shuffle": string;
    /** Main panel background */
    "panel-background": string;
    "library-text-color": string;
    "library-playing-bg": string;
    "library-playing-text": string;
    "library-playing-icon": string;
    "library-highlight-bg": string;
    "library-hover-bg": string;
    "library-header-bg": string;
    "library-header-text": string;
    "library-header-active-bg": string;
    "library-clickable-cell-bg": string;
    "library-clickable-cell-hover-bg": string;
    "library-favourite-icon": string;
    "smart-playlist-builder-bg": string;
    "smart-playlist-builder-text": string;
    "smart-playlist-builder-block-bg": string;
    "smart-playlist-builder-block-focused-bg": string;
    "smart-playlist-builder-block-input-bg": string;
    /** Oscilloscope line color */
    "oscilloscope": string;
}
export const allFonts = {
    "2Peas": {
        "--font": "2Peas"
    }
};

export const DEFAULT_THEME: Theme = {
    "type": "dark",
    "text": "#ffffffde",
    "text-secondary": "#9fa0a2",
    "text-active": "#ffffff",
    "text-inactive": "#676464",
    "background": "#242026",
    "overlay-bg": "#3c3c3f33",
    "accent": "#45fffcf3",
    "accent-secondary": "#5123dd",
    "button-bg": "#1c1b1b",
    "button-text": "#ffffff",
    "input-bg": "#4d494966",
    "icon-primary": "#ffffff",
    "icon-secondary": "#474747",
    "transport-controls": "#ffffff",
    "transport-favorite": "#59cd7a",
    "transport-shuffle": "#e1ff00",
    "panel-background": "#242026b3",
    "library-text-color": "#d3d3d3",
    "library-playing-bg": "#5123dd",
    "library-playing-text": "#00ddff",
    "library-playing-icon": "#00ddff",
    "library-highlight-bg": "#2e3357",
    "library-hover-bg": "#1f1f1f",
    "library-header-bg": "#71658e7e",
    "library-header-text": "#ffffffde",
    "library-header-active-bg": "#604d8d",
    "library-clickable-cell-bg": "#71658e1e",
    "library-clickable-cell-hover-bg": "#8c7dae36",
    "library-favourite-icon": "#5123dd",
    "smart-playlist-builder-bg": "#4d347c",
    "smart-playlist-builder-text": "#ffffffde",
    "smart-playlist-builder-block-bg": "#7256be56",
    "smart-playlist-builder-block-focused-bg": "#7256be99",
    "smart-playlist-builder-block-input-bg": "#231b4d45",
    oscilloscope: "#14D8BD"
};

export const allThemes: { [key: string]: Theme } = {
    light: {
        ...DEFAULT_THEME,
        "type": "light",
        "text": "#282230",
        "text-secondary": "#56575b",
        "text-active": "#38353b",
        "text-inactive": "#707879",
        "background": "#e8e1e1",
        "overlay-bg": "#f1f1f1c0",
        "accent": "#5d00fff3",
        "accent-secondary": "#da69ff",
        "button-bg": "#282b2c",
        "button-text": "#ede8f0",
        "input-bg": "#86808066",
        "icon-primary": "#2c0b3e",
        "icon-secondary": "#716a6a",
        "transport-controls": "#333135",
        "transport-favorite": "#49ab65",
        "transport-shuffle": "#9804d3",
        "panel-background": "#dfd8d8",
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
        "smart-playlist-builder-block-input-bg": "#1b474d6f",
        oscilloscope: "#017b94"
    },
    light2: {
        ...DEFAULT_THEME,
        "type": "light",
        "text": "#282230",
        "text-secondary": "#56575b",
        "text-active": "#38353b",
        "text-inactive": "#646067",
        "background": "#e9e5efd9",
        "overlay-bg": "#f1f1f1c0",
        "accent": "#5d00fff3",
        "accent-secondary": "#da69ff",
        "button-bg": "#282b2c",
        "button-text": "#ede8f0",
        "input-bg": "#86808066",
        "icon-primary": "#2c0b3e",
        "icon-secondary": "#716a6a",
        "transport-controls": "#333135",
        "transport-favorite": "#49ab65",
        "transport-shuffle": "#9804d3",
        "panel-background": "#e3dfe8",
        "library-text-color": "#3e4040",
        "library-playing-bg": "#5123dd",
        "library-playing-text": "#ffffff",
        "library-playing-icon": "#efe2e6",
        "library-highlight-bg": "#5123dd45",
        "library-hover-bg": "#5123dd1c",
        "library-header-bg": "#998bbb7e",
        "library-header-text": "#312c2c",
        "library-header-active-bg": "#997be1",
        "library-clickable-cell-bg": "#9891aa1c",
        "library-clickable-cell-hover-bg": "#5123dd6e",
        "library-favourite-icon": "#5123dd",
        "smart-playlist-builder-bg": "#b49bff",
        "smart-playlist-builder-text": "#ffffffde",
        "smart-playlist-builder-block-bg": "#a691f3",
        "smart-playlist-builder-block-focused-bg": "#8b76d0",
        "smart-playlist-builder-block-input-bg": "#907dd1",
        oscilloscope: "#590194"
    },
    dark: DEFAULT_THEME,
    amphibian: {
        ...DEFAULT_THEME,
        "type": "dark",
        "text": "#b6e1e3de",
        "text-secondary": "#819be8",
        "text-active": "#83e4ff",
        "text-inactive": "#668e93",
        "background": "#233346",
        "overlay-bg": "#2333466d",
        "accent": "#00ff62f3",
        "accent-secondary": "#da69ff",
        "button-bg": "#486d7b",
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
