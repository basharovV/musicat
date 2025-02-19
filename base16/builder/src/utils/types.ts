type BaseId =
    | "base00"
    | "base01"
    | "base02"
    | "base03"
    | "base04"
    | "base05"
    | "base06"
    | "base07"
    | "base08"
    | "base09"
    | "base0A"
    | "base0B"
    | "base0C"
    | "base0D"
    | "base0E"
    | "base0F";

type HexValues = {
    [K in
        | `${BaseId}-hex`
        | `${BaseId}-hex-r`
        | `${BaseId}-hex-g`
        | `${BaseId}-hex-b`
        | `${BaseId}-hex-bgr`]: string;
};

type RgbValues = {
    [K in `${BaseId}-rgb-r` | `${BaseId}-rgb-g` | `${BaseId}-rgb-b`]: number;
};

type DecValues = {
    [K in `${BaseId}-dec-r` | `${BaseId}-dec-g` | `${BaseId}-dec-b`]: string;
};

type SchemeColors = HexValues & RgbValues & DecValues;

export type Scheme = {
    "scheme-name": string;
    "scheme-author": string;
    "scheme-slug": string;
    "scheme-variant": string;
} & SchemeColors;

export type Template = {
    templateData: string;
    extension: string;
    outputDir: string;
    templateSlug: string;
    prefix: string;
};
