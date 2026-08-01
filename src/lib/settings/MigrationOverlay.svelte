<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { migrateFromDexieIfNeeded, migrationState } from "../../data/migration";
    import { userSettings } from "../../data/store";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";

    async function retry() {
        await migrateFromDexieIfNeeded($userSettings);
    }
</script>

{#if $migrationState.status === "migrating" || $migrationState.status === "checking"}
    <div class="backdrop" transition:fade={{ duration: 150 }}>
        <container transition:fly={{ y: 20, duration: 200 }}>
            <LoadingSpinner />
            <p class="title">Setting up your library...</p>
            <p class="detail">
                One-time move to Musicat's new, faster storage. This can take
                a moment for larger libraries -- your existing data isn't
                touched until this finishes successfully.
            </p>
        </container>
    </div>
{:else if $migrationState.status === "failed"}
    <div class="backdrop" transition:fade={{ duration: 150 }}>
        <container transition:fly={{ y: 20, duration: 200 }}>
            <Icon icon="material-symbols:error-outline" size={28} />
            <p class="title">Couldn't finish setting up your library</p>
            <p class="detail">
                Your existing library is untouched -- nothing was lost. This
                just means the app can't show it yet. Restarting Musicat will
                try again automatically, or you can retry now.
            </p>
            {#if $migrationState.mismatches?.length}
                <div class="mismatches">
                    {#each $migrationState.mismatches.slice(0, 20) as mismatch}
                        <p>{mismatch}</p>
                    {/each}
                    {#if $migrationState.mismatches.length > 20}
                        <p>
                            ...and {$migrationState.mismatches.length - 20} more
                        </p>
                    {/if}
                </div>
            {/if}
            <div class="actions">
                <ButtonWithIcon
                    theme="active"
                    size="small"
                    text="Retry"
                    onClick={retry}
                />
            </div>
        </container>
    </div>
{/if}

<style lang="scss">
    @import "../../styles/mixins.scss";

    .backdrop {
        position: absolute;
        inset: 0;
        z-index: 2000;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: var(--popup-backdrop);
    }

    container {
        @include popup;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75em;
        min-width: 400px;
        max-width: 460px;
        padding: 2em 2em 1.75em;
        text-align: center;
    }

    .title {
        margin: 0.25em 0 0;
        font-size: 1.1em;
        font-weight: 600;
        color: var(--primary);
    }

    .detail {
        margin: 0;
        font-size: 0.85em;
        opacity: 0.65;
        line-height: 1.5;
    }

    .mismatches {
        max-height: 160px;
        overflow-y: auto;
        width: 100%;
        text-align: left;
        background: var(--border);
        border-radius: 6px;
        padding: 0.5em 0.75em;

        p {
            margin: 0.2em 0;
            font-size: 0.75em;
            font-family: monospace;
            opacity: 0.75;
            word-break: break-word;
        }
    }

    .actions {
        margin-top: 0.25em;
    }
</style>
