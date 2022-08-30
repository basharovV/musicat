<script lang="ts">
    import { importStatus } from "../data/store";

    import { openTauriImportDialog } from "../data/LibraryImporter";
</script>

<div>
    {#if $importStatus.isImporting}
        <h1>🥁Importing...</h1>
        <h2>{$importStatus.currentFolder}</h2>
        <h3>
            Track {$importStatus.importedTracks} / {$importStatus.totalTracks}
        </h3>
    {:else}
        <h2>An app for those of us that still have a local music library</h2>
        <h3>Click "Import library" or just drag + drop a folder here</h3>
        <p>You can always add more music later</p>
        <button on:click={openTauriImportDialog}>Import library +</button>
        <small>Supports MP3, FLAC, OGG, AAC and WAV</small>
    {/if}
</div>

<style lang="scss">
    div {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;

        small {
            margin-top: 1em;
            opacity: 0.5;
        }
    }
</style>
