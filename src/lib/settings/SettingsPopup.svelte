<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";

    import { register, unregisterAll } from "@tauri-apps/api/globalShortcut";
    import type { LLM, MiniPlayerLocation } from "src/App";
    import { onDestroy, onMount } from "svelte";
    import { focusTrap } from "svelte-focus-trap";
    import {
        importStatus,
        isSettingsOpen,
        userSettings
    } from "../../data/store";
    import { clickOutside } from "../../utils/ClickOutside";
    import Input from "../ui/Input.svelte";
    import Icon from "../ui/Icon.svelte";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import { open } from "@tauri-apps/api/dialog";
    import { audioDir, downloadDir } from "@tauri-apps/api/path";
    import { importPaths } from "../../data/LibraryImporter";
    import tippy from "svelte-tippy";
    import LL from "../../i18n/i18n-svelte";
    import { allThemes } from "../../theming/themes";

    let version = getVersion();
    let commaSeparatedFilenames = $userSettings.albumArtworkFilenames.join(",");
    let miniPlayerLocations: MiniPlayerLocation[] = [
        "bottom-left",
        "bottom-right",
        "top-left",
        "top-right"
    ];

    let llms: LLM[] = ["gpt-3.5-turbo", "gpt-4", "ollama"];

    function onUpdateFilenames() {
        console.log("filenames", commaSeparatedFilenames);
        $userSettings.albumArtworkFilenames = commaSeparatedFilenames
            .split(",")
            .map((t) => t.trim());
    }

    $: commaSeparatedFolders = $userSettings.foldersToWatch.join(",");

    function onUpdateFolders() {
        $userSettings.foldersToWatch = commaSeparatedFolders
            .split(",")
            .map((t) => t.trim());
    }

    function onClose() {
        $isSettingsOpen = false;
    }

    async function openFolderSelector() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: await audioDir()
        });
        if (Array.isArray(selected)) {
            // user selected multiple directories
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single directory

            $userSettings.foldersToWatch.push(selected);
            $userSettings = $userSettings;

            await importPaths([selected], false);
        }
    }

    async function openDefaultDownloadDirSelector() {
        // Open a selection dialog for directories
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: await downloadDir()
        });
        if (Array.isArray(selected)) {
            // user selected multiple directories
        } else if (selected === null) {
            // user cancelled the selection
        } else {
            console.log("selected", selected);
            // user selected a single directory
            $userSettings.downloadLocation = selected;
            $userSettings = $userSettings;
        }
    }

    function removeFolder(folder) {
        $userSettings.foldersToWatch = commaSeparatedFolders
            .split(",")
            .map((t) => t.trim())
            .filter((f) => f !== folder);
    }

    onMount(async () => {
        await register("Esc", () => {
            onClose();
        });
    });

    onDestroy(() => {
        unregisterAll();
    });
</script>

<container>
    <div class="popup" use:clickOutside={onClose} use:focusTrap>
        <header>
            <div class="close">
                <Icon
                    icon="mingcute:close-circle-fill"
                    onClick={() => onClose()}
                />
                <small>ESC</small>
            </div>
            <div class="title-container">
                <h2>{$LL.settings.title()}</h2>
                <small>{$LL.settings.subtitle()}</small>
            </div>
        </header>

        <section>
            <table>
                <tbody>
                    <tr>
                        <th colspan="2">{$LL.settings.library()}</th>
                    </tr>
                    <tr>
                        <td class="folders">
                            <p>{$LL.settings.foldersToWatch()}</p>

                            {#if $importStatus.isImporting}
                                <small>Importing..</small>
                            {:else if $userSettings.foldersToWatch.length}
                                <small>
                                    {$LL.settings.folder(
                                        $userSettings.foldersToWatch.length
                                    )}
                                </small>
                            {/if}
                        </td>
                        <td>
                            {#each $userSettings.foldersToWatch as folder}
                                <div class="folder-item">
                                    <p>{folder}</p>
                                    <div
                                        use:tippy={{
                                            content:
                                                "Removing a folder will not remove the tracks from your library.",
                                            placement: "right"
                                        }}
                                    >
                                        <Icon
                                            icon="mingcute:close-circle-fill"
                                            onClick={() => {
                                                removeFolder(folder);
                                            }}
                                        />
                                    </div>
                                </div>
                            {/each}
                            <ButtonWithIcon
                                theme="transparent"
                                icon="material-symbols:folder"
                                text="Add folder"
                                onClick={openFolderSelector}
                                size="small"
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>Cover art file names</td>
                        <td
                            ><Input
                                bind:value={commaSeparatedFilenames}
                                onChange={onUpdateFilenames}
                                fullWidth
                                small
                            /></td
                        >
                    </tr>
                    <tr>
                        <td>Download location</td>
                        <td>
                            <div class="download-location">
                                <p>{$userSettings.downloadLocation}</p>
                                <Icon
                                    icon="material-symbols:folder"
                                    onClick={() => {
                                        openDefaultDownloadDirSelector();
                                    }}
                                />
                            </div>
                        </td>
                    </tr>
                </tbody>
                <tbody>
                    <tr>
                        <th colspan="2">{$LL.settings.interface()}</th>
                    </tr>
                    <tr>
                        <td>Mini-player location</td>
                        <td>
                            <select
                                bind:value={$userSettings.miniPlayerLocation}
                            >
                                {#each miniPlayerLocations as location}
                                    <option value={location}>{location}</option>
                                {/each}
                            </select></td
                        >
                    </tr>
                    <tr>
                        <td>Theme</td>
                        <td>
                            <select bind:value={$userSettings.theme}>
                                {#each Object.keys(allThemes) as theme}
                                    <option value={theme}>{theme}</option>
                                {/each}
                            </select></td
                        >
                    </tr>
                </tbody>
                <tbody>
                    <tr>
                        <th colspan="2">{$LL.settings.features()}</th>
                    </tr>
                    <tr>
                        <td>Enable AI features</td>
                        <td
                            ><input
                                type="checkbox"
                                bind:checked={$userSettings.aiFeaturesEnabled}
                            /></td
                        >
                    </tr>
                    <tr>
                        <td>Enable Artist's Toolkit</td>
                        <td
                            ><input
                                type="checkbox"
                                bind:checked={$userSettings.isArtistsToolkitEnabled}
                            /></td
                        >
                    </tr>
                    <tr>
                        <td>AI Model (LLM)</td>
                        <td>
                            <select bind:value={$userSettings.llm}>
                                {#each llms as llm}
                                    <option value={llm}>{llm}</option>
                                {/each}
                            </select></td
                        >
                    </tr>
                    <tr>
                        <td>OpenAI API Key</td>
                        <td
                            ><Input
                                bind:value={$userSettings.openAIApiKey}
                                fullWidth
                                small
                            /></td
                        >
                    </tr>
                    <tr>
                        <td>Genius API Key</td>
                        <td
                            ><Input
                                bind:value={$userSettings.geniusApiKey}
                                fullWidth
                                small
                            /></td
                        >
                    </tr>
                </tbody>
            </table>
        </section>
    </div></container
>

<style lang="scss">
    container {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: rgba(0, 0, 0, 0.187);
    }

    .popup {
        width: fit-content;
        max-height: 85%;
        min-width: 500px;
        max-width: 550px;
        margin: auto;
        display: flex;
        flex-direction: column;
        position: relative;
        align-items: center;
        border-radius: 5px;
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(53, 51, 51);
        background-color: var(--overlay-bg);
        backdrop-filter: blur(10px);
        box-shadow: 0px 5px 40px rgba(0, 0, 0, 0.259);
        overflow: auto;

        @media only screen and (max-width: 400px) {
            display: none;
        }
    }

    header {
        display: flex;
        flex-direction: column;
        align-items: center;
        position: sticky;
        top: 0;
        padding: 0.4em 0;
        width: 100%;
        background-color: color-mix(
            in srgb,
            var(--background) 26%,
            transparent
        );
        border-bottom: 1px solid rgb(53, 51, 51);
        backdrop-filter: blur(10px);
        z-index: 20;

        .title-container {
            small {
                opacity: 0.5;
            }
        }

        @media only screen and (max-width: 700px) {
            .title-container {
                display: flex;
                flex-direction: column;
                align-items: flex-start;
                align-self: flex-start;
                margin-left: 4em;
                h2 {
                    margin: 0.5em;
                }
                small {
                    display: none;
                }
            }

            .button-container {
                top: 0.4em;
                right: 15px;
                small {
                    display: none;
                }
            }
            .close {
                top: 1.5em;
            }
        }

        @media only screen and (min-width: 701px) {
            .title-container {
                display: flex;
                flex-direction: column;
                align-items: center;
                text-align: center;
                h2 {
                    margin: 0.2em;
                }
                small {
                    margin: 0;
                }
            }
        }
    }

    .close {
        position: absolute;
        top: 1.7em;
        left: 2em;
        z-index: 20;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 10px;
        small {
            opacity: 0.3;
        }
    }

    section {
        width: 100%;
        padding: 0 2em 2em 2em;
        table {
            width: 100%;
            border-collapse: separate;
            border-spacing: 0 0.4em;
            tr {
                th {
                    text-align: start;
                    vertical-align: middle;
                    padding: 1em 0;
                }
                td {
                    vertical-align: middle;
                    p {
                        margin: 0;
                    }
                    * {
                        color: var(--text);
                    }

                    &.folders {
                        vertical-align: top;
                    }
                }
                td:nth-child(1) {
                    padding-right: 10px;
                    text-align: start;
                    opacity: 0.7;
                    width: 40%;
                }
                td:nth-child(2) {
                    text-align: start;
                }
            }
        }
    }

    .download-location {
        display: flex;
        gap: 10px;
        align-items: center;
    }

    .folder-item {
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: space-between;
        border: 1px solid grey;
        border-radius: 6px;
        padding: 1px 10px;
        margin-bottom: 5px;
        position: relative;
        word-wrap: anywhere;
        word-break: break-all;
        width: 100%;

        > div {
            display: flex;
        }

        p {
            margin: 0;
            font-size: 0.9em;
            color: var(--text);
            white-space: wrap;
            max-width: 300px;
            line-height: initial;
        }
    }
</style>
