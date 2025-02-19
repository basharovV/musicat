import path from "path";
import { safeDump, safeLoad } from "enhanced-yaml";
import { Scheme, Template } from "./types";
import mustache from "mustache";
import { resolveContrast } from "./resolve-contrast";
import { resolve } from "@asamuzakjp/css-color";
import { writeFile } from "fs-extra";

export async function renderTheme(
    template: Template,
    scheme: Scheme,
): Promise<boolean> {
    let output = mustache.render(template.templateData, scheme);

    const data = safeLoad(output) as { [index: string]: string };

    const properties = {};
    const resolvings: { key: string; value: string }[] = [];
    const contrasts: { key: string; value: string }[] = [];

    for (const [key, value] of Object.entries(data)) {
        if (/^(?:rgba?|hsl|color-mix)\(/.test(value)) {
            resolvings.push({ key, value });
        } else if (value.startsWith("color-contrast(")) {
            contrasts.push({ key, value });
        } else {
            properties[`--${key}`] = value;
        }
    }

    for (const { key, value } of resolvings) {
        const newValue = resolve(value, {
            customProperty: properties,
            format: "hexAlpha",
        });

        if (newValue) {
            data[key] = newValue;

            properties[`--${key}`] = newValue;
        }
    }

    for (const { key, value } of contrasts) {
        const newValue = resolveContrast(value, properties);

        if (newValue) {
            data[key] = newValue;

            properties[`--${key}`] = newValue;
        }
    }

    output = safeDump(
        data,
        {
            preserveOriginalOrdering: true,
        },
        output,
    );

    const outputPath = path.join(
        template.outputDir,
        `base16-${scheme["scheme-slug"]}${template.extension}`,
    );

    await writeFile(outputPath, output);

    return true;
}
