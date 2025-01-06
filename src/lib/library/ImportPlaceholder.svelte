<script lang="ts">
    import {
        importStatus,
        popupOpen,
        selectedPlaylistFile,
    } from "../../data/store";

    import { openTauriImportDialog } from "../../data/LibraryUtils";
    import ProgressBar from "../ui/ProgressBar.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    import CassetteLoading from "./CassetteLoading.svelte";
</script>

<div class="container">
    {#if $selectedPlaylistFile}
        <h2>Empty playlist</h2>
        <p>🪣</p>
    {:else if $importStatus.isImporting}
        <h1>🥁Importing...</h1>
        {#if $importStatus.currentFolder}
            <small>{$importStatus.currentFolder}</small>
        {/if}
        <CassetteLoading />
        <div class="status">
            <LoadingSpinner />
            <p>{$importStatus.status ?? "Processing files..."}</p>
        </div>
        {#if $importStatus.percent !== null}
            <div class="progress">
                <ProgressBar percent={$importStatus.percent} />
            </div>
        {/if}
    {:else}
        <h3>Click "Import library" or just drag + drop a folder here</h3>
        <p>You can always add more music later</p>
        <button on:click={openTauriImportDialog}>Import library +</button>
        <small>Supports MP3, FLAC, OGG, AAC and WAV</small>
        <small>or</small>
        <p>add folders to watch</p>
        <button on:click={() => ($popupOpen = "settings")}>Add folders</button>
    {/if}
</div>

<style lang="scss">
    .container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;

        .status {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 10px;
        }

        h1 {
            margin: 0;
        }

        .progress {
            min-width: 300px;
        }

        small {
            margin-top: 1em;
            opacity: 0.5;
        }
    }
</style>
