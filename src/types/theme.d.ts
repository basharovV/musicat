type ThemeVariant = "dark" | "light";

export interface Theme {
    "display-name": string;
    variant: ThemeVariant;
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
    /**  Overlays, popups */
    "overlay-bg": string;
    "overlay-shadow": string;
    /**  Primary accent color used for active items that need to stand out */
    accent: string;
    "accent-secondary": string;
    /** Buttons */
    "button-active-bg": string;
    "button-active-disabled-text": string;
    "button-active-disabled-bg": string;
    "button-solid-bg": string;
    "button-text": string;
    /** Icons */
    "icon-boxed-hover-bg": string;
    "icon-disabled": string;
    "icon-primary": string;
    "icon-secondary": string;
    "icon-secondary-hover": string;
    "icon-tertiary": string;
    "icon-tertiary-hover": string;
    /** Inputs */
    "input-alt-focus-outline": string;
    "input-bg": string;
    "input-focus-bg": string;
    "input-focus-outline": string;
    "input-placeholder-text": string;
    /** */
    "header-opacity": number;
    "header-text": string;
    inverse: string;
    link: string;
    "link-hover": string;
    "shadow-gradient-bg": string;
    title: string;
    "type-bw": string;
    "type-bw-inverse": string;
    /** Album */
    "album-gridsize-line-bg": string;
    "album-gridsize-thumb": string;
    "album-hover-info-bg": string;
    "album-hover-info-text": string;
    "album-playing-pause-bg": string;
    "album-playing-pause-border": string;
    "album-playing-pause-icon": string;
    "album-playing-pause-hover-bg": string;
    "album-playing-pause-hover-icon": string;
    "album-playing-play-bg": string;
    "album-playing-play-border": string;
    "album-playing-play-icon": string;
    "album-playing-play-hover-bg": string;
    "album-playing-play-hover-icon": string;
    "album-playing-shadow": string;
    "album-playing-title-bg": string;
    "album-playing-title-text": string;
    /** Analytics/Stats */
    "analytics-border": string;
    "analytics-text-primary": string;
    "analytics-text-secondary": string;
    "analytics-timeline-track-bg": string;
    "analytics-timeline-vline-bg": string;
    /** Artist's Toolkit */
    "atk-icon-audio": string;
    "atk-icon-link": string;
    "atk-icon-lyric": string;
    "atk-icon-video": string;
    /** Library */
    "library-clickable-cell-bg": string;
    "library-clickable-cell-hover-bg": string;
    "library-column-divider": string;
    "library-favourite-hover-icon": string;
    "library-favourite-icon": string;
    "library-header-active-bg": string;
    "library-header-bg": string;
    "library-header-text": string;
    "library-odd-row-bg": string;
    "library-highlight-bg": string;
    "library-hover-bg": string;
    "library-playing-bg": string;
    "library-playing-icon": string;
    "library-playing-text": string;
    "library-playing-title": string;
    "library-scrollbar-bg": string;
    "library-scrollbar-hover-bg": string;
    "library-text": string;
    "library-title": string;
    /** Lyrics */
    "lyrics-text": string;
    /** Map View */
    "mapview-region-bg": string;
    "mapview-region-border": string;
    "mapview-region-hover-bg": string;
    "mapview-region-hover-border": string;
    "mapview-region-selected-bg": string;
    "mapview-region-selected-border": string;
    "mapview-region-selected-hover-bg": string;
    "mapview-region-selected-hover-border": string;
    "mapview-scale-1": string;
    "mapview-scale-2": string;
    "mapview-tooltip-artist": string;
    "mapview-tooltip-bg": string;
    "mapview-tooltip-border": string;
    "mapview-tooltip-text": string;
    /** Menus */
    "menu-bg": string;
    "menu-border": string;
    "menu-checkbox-off": string;
    "menu-checkbox-on": string;
    "menu-divider": string;
    "menu-item-confirm-bg": string;
    "menu-item-destructive-hover-bg": string;
    "menu-item-destructive-hover-text": string;
    "menu-item-disabled": string;
    "menu-item-highlight-bg": string;
    "menu-item-highlight-border": string;
    "menu-item-highlight-text": string;
    "menu-item-highlight-secondary-bg": string;
    "menu-item-highlight-secondary-border": string;
    "menu-item-highlight-secondary-text": string;
    "menu-item-loading-from": string;
    "menu-item-loading-to": string;
    "menu-item-text": string;
    "menu-shadow": string;
    /** Panels */
    "panel-background": string;
    "panel-primary-border-main": string;
    "panel-primary-border-accent1": string;
    "panel-primary-border-accent2": string;
    "panel-secondary-border-main": string;
    "panel-secondary-border-accent": string;
    "panel-separator": string;
    "panel-shadow-bg": string;
    /** Popups */
    "popup-backdrop": string;
    "popup-body-bg": string;
    "popup-header-bg-lone": string;
    "popup-header-bg-many": string;
    "popup-header-border": string;
    "popup-section-bg": string;
    "popup-section-border": string;
    "popup-section-title-bg": string;
    "popup-section-title-text": string;
    "popup-track-artwork-about": string;
    "popup-track-artwork-found": string;
    "popup-track-artwork-notfound": string;
    "popup-track-data-field-bg": string;
    "popup-track-metadata-prompt-error": string;
    "popup-track-metadata-title": string;
    "popup-track-metadata-validation-error": string;
    "popup-track-metadata-validation-warning": string;
    /** Progress bar */
    "progressbar-track-bg": string;
    "progressbar-value-bg": string;
    /** Prune */
    "prune-button-border": string;
    "prune-button-delete-bg": string;
    "prune-button-delete-pressed-shadow": string;
    "prune-button-keep-bg": string;
    "prune-button-keep-pressed-shadow": string;
    "prune-fordelete": string;
    /** Oscilloscope */
    oscilloscope: string;
    /** ScrollTo */
    "scrollto-active-bg": string;
    "scrollto-active-border": string;
    "scrollto-active-shadow": string;
    "scrollto-bg": string;
    "scrollto-border": string;
    "scrollto-eq": string;
    "scrollto-hover-bg": string;
    "scrollto-hover-border": string;
    "scrollto-hover-shadow": string;
    "scrollto-shadow": string;
    "scrollto-text": string;
    /** Seekbar */
    "seekbar-line-bg": string;
    "seekbar-hoverhead": string;
    "seekbar-thumb": string;
    "seekbar-tooltip-bg": string;
    "seekbar-tooltip-text": string;
    /** Sidebar */
    "sidebar-info-artist-active-bg": string;
    "sidebar-info-artist-hover-bg": string;
    "sidebar-info-title-hover-bg": string;
    "sidebar-info-title-hover-border": string;
    "sidebar-item-drag-bg": string;
    "sidebar-item-hover-bg": string;
    "sidebar-item-hover-text": string;
    "sidebar-item-selected-pipe-bg": string;
    "sidebar-node-inactive-hover-bg": string;
    "sidebar-node-inactive-hover-opacity": number;
    "sidebar-node-inactive-hover-text": string;
    "sidebar-player-disabled-bg": string;
    "sidebar-player-disabled-text": string;
    "sidebar-search-focus-bg": string;
    "sidebar-search-border": string;
    /** Smart playlist */
    "smart-playlist-builder-bg": string;
    "smart-playlist-builder-text": string;
    "smart-playlist-builder-block-bg": string;
    "smart-playlist-builder-block-text": string;
    "smart-playlist-builder-block-focused-bg": string;
    "smart-playlist-builder-block-input-bg": string;
    "smart-playlist-builder-block-input-text": string;
    /** Play, pause, next, previous */
    "transport-control": string;
    "transport-control-hover": string;
    "transport-favorite": string;
    "transport-favorite-hover": string;
    "transport-shuffle": string;
    "transport-shuffle-hover": string;
    "transport-volume-line-bg": string;
    "transport-volume-thumb-bg": string;
    "transport-volume-thumb-icon": string;
    "transport-volume-thumb-secondary": string;
    /** Waveform */
    "waveform-bg": string;
    "waveform-bg-hover": string;
    "waveform-cursor": string;
    "waveform-region-loop": string;
    "waveform-progress": string;
    "waveform-region-current": string;
    "waveform-wave": string;
    "waveform-hover-label-bg": string;
    "waveform-hover-label-text": string;
    "waveform-hover-line": string;
    "waveform-hoverhead-line-bg": string;
    /** Wiki */
    "wiki-bg": string;
    "wiki-close-prompt-bg": string;
    "wiki-close-prompt-text": string;
    "wiki-header-bg": string;
    "wiki-inarticle-bg": string;
    "wiki-pill-bg": string;
    "wiki-pill-border": string;
    "wiki-pill-text": string;
    "wiki-pill-hover-bg": string;
    "wiki-pill-hover-text": string;
    "wiki-mention-bg": string;
    "wiki-mention-border": string;
    "wiki-mention-text": string;
}
