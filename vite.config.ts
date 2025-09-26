import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { analyzer } from "vite-bundle-analyzer";
import yaml from "@modyfi/vite-plugin-yaml";
import { nodePolyfills } from "vite-plugin-node-polyfills";

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        svelte(),
        yaml(),
        nodePolyfills({
            include: ["events", "buffer", "process"],
            globals: {
                Buffer: true,
                process: true,
            },
        }),
    ],
    optimizeDeps: {
        esbuildOptions: {
            target: "esnext",
            // Node.js global to browser globalThis
            define: {
                global: "globalThis",
            },
        },
    },
    worker: {
        format: "es",
    },
    css: {
        preprocessorOptions: {
            scss: {
                api: "modern",
                silenceDeprecations: [
                    "legacy-js-api",
                    "global-builtin",
                    "import",
                    "color-functions",
                ],
            },
        },
    },
});
