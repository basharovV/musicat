<script lang="ts">
    import { onMount } from "svelte";
    import {
        currentIAFile,
        webPlayerBufferedRanges,
        webPlayerIsLoading,
    } from "../../data/store";
    import webAudioPlayer, {
        currentSrc,
        isIAPlaying,
    } from "../player/WebAudioPlayer";
    import Seekbar from "../sidebar/Seekbar.svelte";
    import Icon from "../ui/Icon.svelte";
    import LoadingSpinner from "../ui/LoadingSpinner.svelte";
    import { currentThemeObject } from "../../theming/store";

    let currentTime = 0;

    $: elapsedTime = `${(~~(currentTime / 60))
        .toString()
        .padStart(2, "0")}:${(~~(currentTime % 60))
        .toString()
        .padStart(2, "0")}`;

    $: durationText = `${(~~(($currentIAFile?.duration ?? 0) / 60)).toString().padStart(2, "0")}:${(~~(
        ($currentIAFile?.duration ?? 0) % 60
    ))
        .toString()
        .padStart(2, "0")}`;

    onMount(() => {
        if ($currentSrc === $currentIAFile?.previewSrc) {
            currentTime = webAudioPlayer.audioFile.currentTime;
        }
        webAudioPlayer.onTimeUpdate = (time) => {
            if ($currentSrc === $currentIAFile?.previewSrc) {
                currentTime = time;
            } else {
                currentTime = 0;
            }
        };
    });
</script>

<div class="container">
    <p class="title">{$currentIAFile?.title ?? $currentIAFile?.name ?? ""}</p>

    <div>
        {#if $webPlayerIsLoading && $currentIAFile}
            <LoadingSpinner />
        {:else}
            <Icon
                icon={$isIAPlaying ? "fe:pause" : "fe:play"}
                color={$currentThemeObject["icon-primary"]}
                size={30}
                disabled={!$currentIAFile}
                onClick={() => {
                    if ($currentSrc === $currentIAFile?.previewSrc) {
                        webAudioPlayer.togglePlay();
                    } else webAudioPlayer.playFromUrl($currentIAFile);
                }}
            />
        {/if}
    </div>
    <div class="seekbar">
        <Seekbar
            playerTime={currentTime}
            onSeek={(time) => {
                webAudioPlayer.audioFile.currentTime = time;
            }}
            duration={$currentIAFile?.duration ?? 0}
            buffered={$webPlayerBufferedRanges}
        />
    </div>

    <p class="elapsed-time">{elapsedTime} / {durationText}</p>
</div>

<style lang="scss">
    .container {
        display: flex;
        flex-direction: row;
        padding: 1em;
        border-radius: 5px;
        padding: 15px 10px;
        height: 60px;
        width: 100%;
        align-items: center;
        justify-content: center;

        .title {
            margin: 0;
            line-height: initial;
            position: absolute;

            top: 5px;
        }
        .seekbar {
            flex: 1;
            width: 100%;
        }
        .elapsed-time {
            position: absolute;
            opacity: 0.5;
            font-size: 12px;
            margin: 0;
            line-height: initial;
            top: 42px;
        }
    }
</style>
