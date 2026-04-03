<script lang="ts">
    import { importStatus, popupOpen } from "../../data/store";

    import { openTauriImportDialog } from "../../data/LibraryUtils";
    import LL from "../../i18n/i18n-svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    import ProgressBar from "../ui/ProgressBar.svelte";
    import CassetteLoading from "./CassetteLoading.svelte";
</script>

<div class="container">
    {#if $importStatus.isImporting}
        <h1>{$LL.library.empty.importing()}</h1>
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
        <h3>{$LL.library.empty.title()}</h3>
        <p>{$LL.library.empty.subtitle()}</p>
        <ButtonWithIcon
            onClick={openTauriImportDialog}
            text={$LL.library.empty.import()}
        />
        <small>{$LL.library.empty.formats()}</small>
        <p>{$LL.library.empty.addFoldersToWatch()}</p>
        <ButtonWithIcon
            onClick={() => ($popupOpen = "settings")}
            text={$LL.settings.title()}
        ></ButtonWithIcon>
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
