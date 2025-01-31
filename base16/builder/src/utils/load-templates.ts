import path from "path";
import { globby } from "globby";
import { Template } from "./types";
import { readFile } from "fs-extra";
import { load as loadYaml } from "js-yaml";

export async function loadTemplates(store: string): Promise<Template[]> {
    console.log("Parsing template definitions...");

    const templates: Template[] = [];

    const files = await globby(path.join("*", "templates", "config.yaml"), {
        cwd: path.join(store, "templates"),
        absolute: true,
    });

    for (const file of files) {
        const content = await readFile(file, "utf-8");
        const data = loadYaml(content) as {
            [index: string]: {
                extension: string;
                output: string;
                prefix?: string;
            };
        };
        const templatesDir = path.dirname(file);
        const rootDir = path.dirname(templatesDir);
        const templateSlug = path.basename(rootDir);

        for (const key in data) {
            const { extension, output, prefix } = data[key];
            const templatePath = path.join(templatesDir, `${key}.mustache`);
            const templateData = await readFile(templatePath, "utf-8");

            templates.push({
                templateData,
                extension,
                outputDir: path.join(rootDir, output),
                templateSlug,
                prefix: prefix ?? "",
            });
        }
    }

    console.log(`${templates.length} templates found.`);

    return templates;
}
