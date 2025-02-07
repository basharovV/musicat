import { resolve } from "@asamuzakjp/css-color";
import { tokenize, TokenType } from "@csstools/css-tokenizer";
import { readability, TinyColor } from "@ctrl/tinycolor";

export function resolveContrast(
    value: string,
    properties: { [index: string]: string },
): string | null {
    const tokens: any[] = tokenize({
        css: value,
    });

    if (
        tokens[0][0] !== TokenType.Function ||
        tokens[0][4].value !== "color-contrast"
    ) {
        return null;
    }

    tokens.shift();
    tokens.splice(-2, 2);

    const baseColor = extractColor(tokens, properties);
    if (!baseColor) {
        return null;
    }

    skipSpaces(tokens);

    if (tokens[0][0] !== TokenType.Ident && tokens[0][4].value !== "vs") {
        return null;
    }

    tokens.shift();

    let bestColor: TinyColor | null = null;
    let bestContrast = 0;

    while (tokens.length) {
        const color = extractColor(tokens, properties);
        if (!color) {
            return null;
        }

        let contrast = readability(baseColor, color);

        if (contrast > bestContrast) {
            bestColor = color;
            bestContrast = contrast;
        }

        skipComma(tokens);
    }

    if (bestColor) {
        return bestColor.toHex8String();
    } else {
        return null;
    }
}

function extractColor(
    tokens: any[],
    properties: { [index: string]: string },
): TinyColor | null {
    skipSpaces(tokens);

    switch (tokens[0][0]) {
        case TokenType.Function: {
            switch (tokens[0][4].value) {
                case "color-mix": {
                    const colorTokens: string[] = [tokens.shift()[1]];
                    let nest = 1;

                    while (nest !== 0 && tokens.length) {
                        const [type = "", tokenValue = ""] = tokens.shift() as [
                            TokenType,
                            string,
                        ];

                        colorTokens.push(tokenValue);

                        switch (type) {
                            case TokenType.Function:
                            case TokenType.OpenParen: {
                                nest += 1;
                                break;
                            }
                            case TokenType.CloseParen: {
                                nest -= 1;
                                break;
                            }
                        }
                    }

                    const cssValue = colorTokens.join("");
                    const value = resolve(cssValue, {
                        customProperty: properties,
                        format: "hexAlpha",
                    });

                    if (!value) {
                        return null;
                    }

                    const color = new TinyColor(value);

                    if (color.isValid) {
                        return color;
                    }

                    break;
                }
                case "var": {
                    tokens.shift();

                    skipSpaces(tokens);

                    if (tokens[0][0] !== TokenType.Ident) {
                        return null;
                    }

                    const value = properties[tokens[0][4].value];

                    if (!value) {
                        return null;
                    }

                    tokens.shift();

                    skipSpaces(tokens);

                    tokens.shift();

                    const color = new TinyColor(value);

                    if (color.isValid) {
                        return color;
                    }

                    break;
                }
            }
            break;
        }
    }

    return null;
}

function skipComma(tokens: any[]): void {
    skipSpaces(tokens);

    if (tokens.length && tokens[0][0] === TokenType.Comma) {
        tokens.shift();

        skipSpaces(tokens);
    }
}

function skipSpaces(tokens: any[]): void {
    while (tokens.length && tokens[0][0] === TokenType.Whitespace) {
        tokens.shift();
    }
}
