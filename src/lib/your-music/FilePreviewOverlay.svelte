<script lang="ts">
    import { convertFileSrc, invoke } from "@tauri-apps/api/core";
    import { fade, fly } from "svelte/transition";
    import { scrapbookPreviewItem } from "../../data/store";
    import Icon from "../ui/Icon.svelte";

    let textContent: string | null = null;
    let textError: string | null = null;

    // Text files aren't just src-able like images/video -- read the contents (via the
    // spawn_blocking-safe command, not @tauri-apps/plugin-fs's readTextFile -- see the comment on
    // read_text_file_blocking in src-tauri/src/files.rs) whenever a new one is opened.
    $: if ($scrapbookPreviewItem?.fileType.type === "txt") {
        textContent = null;
        textError = null;
        invoke<string>("read_text_file_blocking", { path: $scrapbookPreviewItem.path })
            .then((content) => (textContent = content))
            .catch((err) => (textError = String(err)));
    }

    function close() {
        $scrapbookPreviewItem = null;
    }

    function onKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") close();
    }
</script>

<svelte:window on:keydown={onKeydown} />

{#if $scrapbookPreviewItem}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="backdrop" transition:fade={{ duration: 150 }} on:click={close}>
        <container transition:fly={{ y: 20, duration: 200 }} on:click|stopPropagation>
            <header>
                <span class="title">{$scrapbookPreviewItem.name}</span>
                <Icon icon="material-symbols:close" onClick={close} boxed />
            </header>
            <div class="body">
                {#if $scrapbookPreviewItem.fileType.type === "image"}
                    <!-- svelte-ignore a11y-missing-attribute -->
                    <img src={convertFileSrc($scrapbookPreviewItem.path)} />
                {:else if $scrapbookPreviewItem.fileType.type === "video"}
                    <!-- svelte-ignore a11y-media-has-caption -->
                    <video
                        src={convertFileSrc($scrapbookPreviewItem.path)}
                        controls
                        autoplay
                    />
                {:else if $scrapbookPreviewItem.fileType.type === "txt"}
                    {#if textError}
                        <p class="error">{textError}</p>
                    {:else if textContent !== null}
                        <pre>{textContent}</pre>
                    {:else}
                        <p class="loading">Loading…</p>
                    {/if}
                {/if}
            </div>
        </container>
    </div>
{/if}

<style lang="scss">
    @import "../../styles/mixins.scss";

    .backdrop {
        position: fixed;
        inset: 0;
        z-index: 1000;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: var(--popup-backdrop);
    }

    container {
        @include popup;
        display: flex;
        flex-direction: column;
        max-width: 80vw;
        max-height: 80vh;
        overflow: hidden;
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1em;
        padding: 0.75em 0.75em 0.75em 1.25em;
        border-bottom: 1px solid var(--border);

        .title {
            font-size: 0.9em;
            font-weight: 600;
            color: var(--primary);
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
    }

    .body {
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: auto;
        padding: 1em;

        img,
        video {
            max-width: 100%;
            max-height: 70vh;
            object-fit: contain;
            border-radius: 4px;
        }

        pre {
            max-width: 60vw;
            max-height: 65vh;
            overflow: auto;
            margin: 0;
            padding: 1em;
            font-family: monospace;
            font-size: 0.85em;
            white-space: pre-wrap;
            word-break: break-word;
            color: var(--primary);
            text-align: left;
        }

        .loading,
        .error {
            margin: 0;
            opacity: 0.6;
        }

        .error {
            color: var(--destructive);
        }
    }
</style>
