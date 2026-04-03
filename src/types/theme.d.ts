type ThemeVariant = "light" | "dark";

export interface Theme {
    "display-name": string;
    variant: ThemeVariant;

    // Surfaces
    window: string;
    panel: string;
    overlay: string;
    soft: string;
    solid: string;
    shadow: string;
    "shadow-soft": string;

    // Accent colors
    accent: string;
    "accent-soft": string;
    "accent-softer": string;
    "accent-softest": string;
    "accent-text": string;
    "accent-play": string;
    "accent-love": string;

    // UI components
    "sidebar-active-nav-icon": string;
    "library-hover": string;
    "library-highlight": string;
    menu: string;
    "menu-hover": string;
    waveform: string;
    "waveform-progress": string;
    shuffle: string;
    repeat: string;

    // Text
    primary: string;
    secondary: string;
    muted: string;

    // Semantic
    border: string;
    destructive: string;
    positive: string;
    warning: string;
}
