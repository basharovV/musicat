import { slugify } from "./slugify";
import { Scheme } from "./types";
import { reverseHex } from "./reverse-hex";
import { toDecimal } from "./to-decimal";

export function buildScheme(data): Scheme {
    let scheme = {
        "scheme-name": data.name,
        "scheme-author": data.author,
        "scheme-slug": slugify(data.name),
        "scheme-variant": data.variant,
    };

    for (let [key, value] of Object.entries(
        data.palette as { [index: string]: string },
    )) {
        const hex = value.replace(/^#/, "");
        const bytecode = Buffer.from(hex, "hex");
        const red = bytecode[0];
        const green = bytecode[1];
        const blue = bytecode[2];

        scheme[`${key}-hex`] = hex;
        scheme[`${key}-hex-r`] = red.toString(16);
        scheme[`${key}-rgb-r`] = red;
        scheme[`${key}-hex-g`] = green.toString(16);
        scheme[`${key}-rgb-g`] = green;
        scheme[`${key}-hex-b`] = blue.toString(16);
        scheme[`${key}-rgb-b`] = blue;
        scheme[`${key}-hex-bgr`] = reverseHex(hex);
        scheme[`${key}-dec-r`] = toDecimal(red);
        scheme[`${key}-dec-g`] = toDecimal(green);
        scheme[`${key}-dec-b`] = toDecimal(blue);
    }

    return scheme as Scheme;
}
