import path from "path";
import { loadSchemes } from "./utils/load-schemes";
import { loadTemplates } from "./utils/load-templates";
import { renderTheme } from "./utils/render-theme";

const ROOT = process.cwd();

async function build(store: string) {
    const storePath = path.join(ROOT, store);

    const schemes = await loadSchemes(storePath);
    const templates = await loadTemplates(storePath);

    console.log("Rendering themes...");

    let result = 0;

    for (const template of templates) {
        for (const scheme of schemes) {
            if (await renderTheme(template, scheme)) {
                result += 1;
            }
        }
    }

    console.log(`${result} themes rendered.`);
}

build("..");
