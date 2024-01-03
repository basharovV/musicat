<script lang="ts">
    import { importStatus, isSettingsOpen, selectedPlaylistId } from "../data/store";

    import { openTauriImportDialog } from "../data/LibraryImporter";
</script>

<div>
    {#if $selectedPlaylistId}
        <h2>Empty playlist</h2>
        <p>🪣</p>
    {:else if $importStatus.isImporting}
        <h1>🥁Importing...</h1>
        <h2>{$importStatus.currentFolder}</h2>
        <h3>
            Track {$importStatus.importedTracks} / {$importStatus.totalTracks}
        </h3>
    {:else}
        <h3>Click "Import library" or just drag + drop a folder here</h3>
        <p>You can always add more music later</p>
        <button on:click={openTauriImportDialog}>Import library +</button>
        <small>Supports MP3, FLAC, OGG, AAC and WAV</small>
        <small>or</small>
        <p>add folders to watch</p>
        <button on:click={() => $isSettingsOpen = true}>Add folders</button>
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
