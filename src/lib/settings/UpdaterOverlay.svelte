<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { fade, fly } from "svelte/transition";
    import { updaterStatus } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import ProgressBar from "../ui/ProgressBar.svelte";

    function inlineMarkdown(text: string): string {
        return text
            .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
            .replace(/`(.+?)`/g, "<code>$1</code>");
    }

    function renderMarkdown(md: string): string {
        const safe = md
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
        const lines = safe.split("\n");
        const parts: string[] = [];
        let inList = false;

        for (const line of lines) {
            if (line.startsWith("### ")) {
                if (inList) {
                    parts.push("</ul>");
                    inList = false;
                }
                parts.push(`<h3>${inlineMarkdown(line.slice(4))}</h3>`);
            } else if (line.startsWith("## ")) {
                if (inList) {
                    parts.push("</ul>");
                    inList = false;
                }
                parts.push(`<h2>${inlineMarkdown(line.slice(3))}</h2>`);
            } else if (line.startsWith("# ")) {
                if (inList) {
                    parts.push("</ul>");
                    inList = false;
                }
                parts.push(`<h1>${inlineMarkdown(line.slice(2))}</h1>`);
            } else if (line.startsWith("- ") || line.startsWith("* ")) {
                if (!inList) {
                    parts.push("<ul>");
                    inList = true;
                }
                parts.push(`<li>${inlineMarkdown(line.slice(2))}</li>`);
            } else if (line.trim() === "") {
                if (inList) {
                    parts.push("</ul>");
                    inList = false;
                }
            } else {
                if (inList) {
                    parts.push("</ul>");
                    inList = false;
                }
                parts.push(`<p>${inlineMarkdown(line)}</p>`);
            }
        }

        if (inList) parts.push("</ul>");
        return parts.join("");
    }

    function dismiss() {
        $updaterStatus = { status: "idle" };
    }

    function install() {
        $updaterStatus = {
            ...$updaterStatus,
            status: "downloading",
            progress: 0,
        };
        invoke("install_update").catch((e) => {
            $updaterStatus = { status: "error", error: String(e) };
        });
    }

    $: canDismiss =
        $updaterStatus.status !== "downloading" &&
        $updaterStatus.status !== "installing";

    $: title =
        {
            available: $LL.updater.titleAvailable(),
            downloading: $LL.updater.titleDownloading(),
            installing: $LL.updater.titleInstalling(),
            "up-to-date": $LL.updater.titleUpToDate(),
            error: $LL.updater.titleError(),
        }[$updaterStatus.status] ?? "";
</script>

{#if $updaterStatus.status !== "idle" && $updaterStatus.status !== "checking"}
    <div class="backdrop" transition:fade={{ duration: 150 }}>
        <container transition:fly={{ y: 20, duration: 200 }}>
            <header>
                <Icon icon="hugeicons:download-05" />
                <span class="title">{title}</span>
                {#if canDismiss}
                    <Icon
                        icon="material-symbols:close"
                        onClick={dismiss}
                        boxed
                    />
                {:else}
                    <div />
                {/if}
            </header>

            <div class="body">
                {#if $updaterStatus.status === "available"}
                    <p class="version">v{$updaterStatus.version}</p>
                    {#if $updaterStatus.notes}
                        <div class="notes">
                            {@html renderMarkdown($updaterStatus.notes)}
                            {@html renderMarkdown($updaterStatus.notes)}
                            {@html renderMarkdown($updaterStatus.notes)}
                            {@html renderMarkdown($updaterStatus.notes)}
                            {@html renderMarkdown($updaterStatus.notes)}
                            {@html renderMarkdown($updaterStatus.notes)}
                        </div>
                    {/if}
                    <div class="actions">
                        <ButtonWithIcon
                            theme="transparent"
                            fullWidth
                            size="small"
                            text={$LL.updater.later()}
                            onClick={dismiss}
                        />
                        <ButtonWithIcon
                            theme="active"
                            fullWidth
                            size="small"
                            text={$LL.updater.install()}
                            onClick={install}
                        />
                    </div>
                {:else if $updaterStatus.status === "downloading"}
                    <ProgressBar percent={$updaterStatus.progress ?? 0} />
                    <p class="detail">
                        {$LL.updater.downloading({
                            percent: Math.round($updaterStatus.progress ?? 0),
                        })}
                    </p>
                {:else if $updaterStatus.status === "installing"}
                    <ProgressBar percent={100} />
                {:else if $updaterStatus.status === "up-to-date"}
                    <div class="actions">
                        <ButtonWithIcon
                            theme="solid"
                            size="small"
                            text={$LL.updater.ok()}
                            onClick={dismiss}
                        />
                    </div>
                {:else if $updaterStatus.status === "error"}
                    {#if $updaterStatus.error}
                        <p class="detail error">{$updaterStatus.error}</p>
                    {/if}
                    <div class="actions">
                        <ButtonWithIcon
                            theme="translucent"
                            size="small"
                            text={$LL.updater.ok()}
                            onClick={dismiss}
                        />
                    </div>
                {/if}
            </div>
        </container>
    </div>
{/if}

<style lang="scss">
    @import "../../styles/mixins.scss";

    .backdrop {
        position: absolute;
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
        min-width: 400px;
        max-width: 420px;
        overflow: hidden;
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1em;
        padding: 1em 1em 1em 1.5em;
        border-bottom: 1px solid var(--border);

        .title {
            font-size: 1em;
            font-weight: 600;
            color: var(--primary);
        }
    }

    .body {
        display: flex;
        flex-direction: column;
        gap: 0.75em;
        padding: 1.25em 1.5em 1.5em;
    }

    .version {
        margin: 0;
        font-size: 1.4em;
        font-weight: 500;
        color: var(--primary);
        opacity: 0.85;
    }

    .detail {
        margin: 0;
        font-size: 0.82em;
        opacity: 0.55;

        &.error {
            word-break: break-word;
        }
    }

    .notes {
        max-height: 240px;
        overflow-y: auto;
        opacity: 0.75;
        border-radius: 6px;
        padding: 0.2em 1em;
        text-align: left;
        border-radius: 10px;

        :global(h1),
        :global(h2),
        :global(h3) {
            margin: 0.5em 0 0.25em;
            font-size: 1em;
            font-weight: 600;
        }

        :global(p) {
            margin: 0.25em 0;
        }

        :global(ul) {
            margin: 0.25em 0;
            padding-left: 1.25em;
        }

        :global(li) {
            margin: 0.1em 0;
        }

        :global(code) {
            font-family: monospace;
            background: var(--border);
            border-radius: 3px;
            padding: 0.1em 0.3em;
        }

        :global(strong) {
            font-weight: 600;
            opacity: 0.9;
        }
    }

    .actions {
        display: flex;
        gap: 0.5em;
        justify-content: center;
        margin-top: 0.25em;
    }
</style>
