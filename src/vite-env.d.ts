/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module "*.md" {
    import type { SvelteComponentTyped } from "svelte";
    export default class Comp extends SvelteComponentTyped {
        $$prop_def: {};
    }
    export const metadata: Record<string, any>;
}
