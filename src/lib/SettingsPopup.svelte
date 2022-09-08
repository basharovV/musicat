<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";

    import {
        isInfoPopupOpen,
        isSettingsOpen,
        userSettings
    } from "../data/store";
    import Input from "./Input.svelte";
    import ReleaseNotes from "./ReleaseNotes.svelte";
    import { clickOutside } from "../utils/ClickOutside";
    import hotkeys from "hotkeys-js";
    import { onDestroy, onMount } from "svelte";
    import { focusTrap } from "svelte-focus-trap";
    import { register, unregisterAll } from "@tauri-apps/api/globalShortcut";

    let version = getVersion();
    let commaSeparatedFilenames = $userSettings.albumArtworkFilenames.join(",");

    function onUpdateFilenames(_) {
        console.log("filenames", commaSeparatedFilenames);
        $userSettings.albumArtworkFilenames = commaSeparatedFilenames
            .split(",")
            .map((t) => t.trim());
    }

    function onClose() {
        $isSettingsOpen = false;
    }

    onMount(async () => {
        await register("Esc", () => {
            onClose();
        });
    });

    onDestroy(() => {
        unregisterAll();
    })

</script>

<container>
    <div class="popup" use:clickOutside={onClose} use:focusTrap>
        <header>
            <div class="close">
                <iconify-icon
                    icon="mingcute:close-circle-fill"
                    on:click={onClose}
                />
                <small>ESC</small>
            </div>
            <div class="title-container">
                <h2>Settings</h2>
                <small>Not sure what to put here</small>
            </div>
        </header>

        <section>
            <table>
                <tr>
                    <td>Cover art file names</td>
                    <td
                        ><Input
                            bind:value={commaSeparatedFilenames}
                            onChange={onUpdateFilenames}
                            fullWidth
                        /></td
                    >
                </tr>
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
        margin: auto;
        display: flex;
        flex-direction: column;
        position: relative;
        align-items: center;
        border-radius: 5px;
        /* background-color: rgba(0, 0, 0, 0.187); */
        border: 1px solid rgb(53, 51, 51);
        background: rgba(67, 65, 73, 0.89);
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
        background: rgba(67, 65, 73, 0.89);
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
        padding: 2em;
        table {
            width: 100%;
        }
    }
</style>
