import type { BaseTranslation } from "../i18n-types";

const en = {
    infoPopup: {
        buildBy: "Built by ",
        version: "version",
        releaseNotes: "Release Notes"
    },
    sidebar: {
        search: "Search",
        library: "Library",
        albums: "Albums",
        favorites: "Favourites",
        playlists: "Playlists",
        smartPlaylists: "Smart Playlists",
        artistsToolkit: "Artist's Toolkit",
        map: "Map",
        internetArchive: "Internet Archive",
        stats: "Stats"
    },
    library: {
        fields: {
            title: "Title",
            artist: "Artist",
            composer: "Composer",
            album: "Album",
            track: "Track",
            year: "Year",
            dateAdded: "Date Added",
            genre: "Genre",
            origin: "Origin",
            duration: "Duration"
        }
    },
    bottomBar: {
        queue: "Queue",
        lyrics: "Lyrics",
        lossySelector: {
            lossy: "lossy",
            lossless: "lossless",
            both: "lossy + lossless"
        },
        nextUp: "Next Up",
        stats: {
            songs: "songs",
            artists: "artists",
            albums: "albums"
        }
    }
} satisfies BaseTranslation;

export default en;
