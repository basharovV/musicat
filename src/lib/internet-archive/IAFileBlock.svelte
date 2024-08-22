<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { IAFile } from "../../App";
    import { fileToDownload, userSettings } from "../../data/store";
    import webAudioPlayer, {
        currentSrc,
        isIAPlaying
    } from "../player/WebAudioPlayer";
    import Icon from "../ui/Icon.svelte";
    import { currentThemeObject } from "../../theming/store";

    export let file: IAFile;
    let downloadProgress = null;

    async function download() {
        console.log("download");
        downloadProgress = 0;
        try {
            file.downloadLocation = `${$userSettings.downloadLocation}/${file.title ?? file.name}`;
            // Append extension if doesn't exist
            if (!file.downloadLocation.match(/\/[^\/]+\.[^\/]+$/)) {
                file.downloadLocation += `.${file.format}`;
            }
            $fileToDownload = file;
            await invoke("download_file", {
                url: file.previewSrc,
                path: file.downloadLocation
            });
        } catch (err) {
            console.error(err);
        }
    }
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<div class="file" role="listitem">
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        class="left"
        on:click={() => {
            if ($currentSrc === file.previewSrc) {
                webAudioPlayer.togglePlay();
            } else webAudioPlayer.playFromUrl(file);
        }}
    >
        <Icon
            icon={$currentSrc === file.previewSrc && $isIAPlaying
                ? "fe:pause"
                : "fe:play"}
            color={$currentThemeObject["icon-primary"]}
            size={30}
        />
        <div class="info">
            <p class="title">
                {file?.title ?? file.name}
            </p>
            <p class="format">
                {file.format}
            </p>
        </div>
    </div>
    <div class="download" on:click={download}>
        <Icon icon="hugeicons:download-05" />
        <p class="size">
            {(Number(file.size) / (1024 * 1000)).toFixed(2)} MB
        </p>
    </div>
</div>

<style lang="scss">
    .file {
        display: grid;
        grid-template-columns: 1fr 70px;
        border-radius: 5px;
        background-color: color-mix(in srgb, var(--inverse) 10%, transparent);
        margin: 0.25em;
        align-items: center;
        justify-content: space-between;
        border: 0.7px solid color-mix(in srgb, var(--inverse) 20%, transparent);

        .left {
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 10px;
            flex: 1;
            height: 100%;

            .info {
                width: 100%;
                display: flex;
                flex-direction: column;
                align-items: flex-start;
                text-align: start;
                .title {
                }
                .format {
                    color: var(--text-secondary);
                }
            }

            &:hover {
                cursor: pointer;
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 5%,
                    transparent
                );
            }
            &:active {
                cursor: pointer;
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 10%,
                    transparent
                );
            }
        }
        .download {
            padding: 0.5em;
            border-left: 0.7px solid
                color-mix(in srgb, var(--inverse) 20%, transparent);
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            gap: 2px;
            height: 100%;

            &:hover {
                cursor: pointer;
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 5%,
                    transparent
                );
            }
            &:active {
                cursor: pointer;
                background-color: color-mix(
                    in srgb,
                    var(--inverse) 10%,
                    transparent
                );
            }
        }
        .size {
            color: var(--text-secondary);
            white-space: nowrap;
            font-size: 12px;
        }

        p {
            margin: 0;
            text-align: start;
        }
    }
</style>
