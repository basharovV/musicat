import type { ThemeVariant } from "../types/theme";

function hexToRGB(hex: string): [number, number, number] {
    const n = parseInt(hex.replace("#", ""), 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function resolveAlpha(hex: string, alpha: number): string {
    const [r, g, b] = hexToRGB(hex);
    return rgbToHex(r, g, b, alpha);
}
/**
 * Converts 3, 6, or 8-digit hex to RGBA array.
 * Values are 0-255, except alpha which is 0-1.
 */
function hexToRGBA(hex: string): [number, number, number, number] {
    const cleanHex = hex.replace("#", "");

    // Standardize 3-digit hex (#f00 -> #ff0000)
    let fullHex = cleanHex;
    if (cleanHex.length === 3) {
        fullHex = cleanHex
            .split("")
            .map((char) => char + char)
            .join("");
    }

    const r = parseInt(fullHex.slice(0, 2), 16);
    const g = parseInt(fullHex.slice(2, 4), 16);
    const b = parseInt(fullHex.slice(4, 6), 16);

    // Default alpha to 1 if not present
    const aRaw = fullHex.length === 8 ? parseInt(fullHex.slice(6, 8), 16) : 255;
    const a = aRaw / 255;

    return [r, g, b, a];
}

function rgbToHex(r: number, g: number, b: number, a: number = 1): string {
    const toHex = (v: number) => Math.round(v).toString(16).padStart(2, "0");

    const hex = "#" + toHex(r) + toHex(g) + toHex(b);
    // Only append alpha if it's not fully opaque (optional, but cleaner)
    return a < 1 ? hex + toHex(a * 255) : hex;
}

function resolveColorMix(hexA: string, hexB: string, weightA: number): string {
    const [r1, g1, b1, a1] = hexToRGBA(hexA);
    const [r2, g2, b2, a2] = hexToRGBA(hexB);

    const w = weightA / 100;
    const invW = 1 - w;

    // Interpolate all 4 channels
    return rgbToHex(
        r1 * w + r2 * invW,
        g1 * w + g2 * invW,
        b1 * w + b2 * invW,
        a1 * w + a2 * invW,
    );
}

export function resolveDerived(
    expression: string,
    core: Record<string, string>,
    variant: ThemeVariant,
): string {
    const relativeRGB = expression.match(
        /rgb\(from var\(--(.+?)\) r g b \/ ([0-9.]+)\)/,
    );
    if (relativeRGB) {
        const [, key, alpha] = relativeRGB;
        const base = core[key];
        if (!base)
            throw new Error(
                `Cannot resolve var(--${key}) in derived expression. Expression: ${expression}`,
            );
        return resolveAlpha(base, parseFloat(alpha));
    }

    const colorMix = expression.match(
        /color-mix\(in srgb,\s*(.+?)\s+([0-9.]+)%,\s*(.+?)\)/,
    );
    console.log("color-mix", colorMix, core);
    if (colorMix) {
        const [, keyA, pctA, keyB] = colorMix;
        let colorA;
        let colorB;
        if (keyA.startsWith("var(--")) {
            let colorKey = keyA.match(/var\(--(.+?)\)/)[1];
            if (!core[colorKey])
                throw new Error(
                    `Cannot resolve var(--${colorKey}) in derived expression`,
                );
            colorA = core[colorKey];
        } else if (keyA.startsWith("#")) {
            colorA = keyA;
        } else {
            throw new Error(
                `Unknown color: ${keyB}. Derived colors must contain only var(--) or #hex values`,
            );
        }

        if (keyB.startsWith("var(--")) {
            let colorKey = keyB.match(/var\(--(.*)/)[1];
            if (!core[colorKey])
                throw new Error(
                    `Cannot resolve var(--${colorKey}) in derived expression`,
                );
            colorB = core[colorKey];
        } else if (keyB.startsWith("#")) {
            colorB = keyB;
        } else {
            throw new Error(
                `Unknown color: ${keyB}. Derived colors must contain only var(--) or #hex values`,
            );
        }
        const colorMixResult = resolveColorMix(
            colorA,
            colorB,
            parseFloat(pctA),
        );
        console.log(
            "color-mix",
            keyA,
            keyB,
            pctA,
            colorA,
            colorB,
            colorMixResult,
        );
        return colorMixResult;
    }

    console.log("Unknown expression", expression);

    return expression;
}
