import path from "path";
import { globby } from "globby";
import { Scheme } from "./types";
import { readFile } from "fs-extra";
import { buildScheme } from "./build-scheme";
import { load as loadYaml } from "js-yaml";

export async function loadSchemes(store: string): Promise<Scheme[]> {
    console.log("Parsing scheme definitions...");

    const schemes: Scheme[] = [];

    const files = await globby("*.yaml", {
        cwd: path.join(store, "schemes"),
        absolute: true,
    });

    for (const file of files) {
        const content = await readFile(file, "utf-8");
        const data = loadYaml(content);

        schemes.push(buildScheme(data));
    }

    console.log(`${schemes.length} schemes found.`);

    return schemes;
}
