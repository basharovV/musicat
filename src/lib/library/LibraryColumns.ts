import { get } from "svelte/store";
import type { ColumnViewModel } from "../../App";
import LL from "../../i18n/i18n-svelte";

export function getAllColumns(): ColumnViewModel[] {
    return [
        {
            name: get(LL).library.fields.title(),
            value: "title",
            viewProps: {
                width: 100,
                autoWidth: true,
            },
        },
        {
            name: get(LL).library.fields.artist(),
            value: "artist",
            viewProps: {
                width: 100,
                autoWidth: true,
            },
        },
        {
            name: get(LL).library.fields.composer(),
            value: "composer",
            viewProps: {
                width: 100,
                autoWidth: true,
            },
        },
        {
            name: get(LL).library.fields.album(),
            value: "album",
            viewProps: {
                width: 100,
                autoWidth: true,
            },
        },
        {
            name: get(LL).library.fields.albumArtist(),
            value: "albumArtist",
            viewProps: {
                width: 100,
                autoWidth: true,
            },
        },
        {
            name: get(LL).library.fields.track(),
            value: "trackNumber",
            viewProps: {
                width: 63,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.dateAdded(),
            value: "dateAdded",
            displayValue: "viewModel.timeSinceAdded",
            viewProps: {
                width: 100,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.compilation(),
            value: "compilation",
            viewProps: {
                width: 63,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.year(),
            value: "year",
            viewProps: {
                width: 63,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.genre(),
            value: "genre",
            viewProps: {
                width: 100,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.origin(),
            value: "originCountryName",
            viewProps: {
                width: 100,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.duration(),
            value: "duration",
            viewProps: {
                width: 63,
                autoWidth: false,
            },
        },
        {
            name: get(LL).library.fields.tags(),
            value: "tags",
            viewProps: {
                width: 100,
                autoWidth: false,
            },
        },
    ];
}
