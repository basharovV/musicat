<script lang="ts">
    import { onMount } from "svelte";
    import type { IAItem } from "../../App";
    import audioPlayer, {
        currentSrc,
        isPlaying
    } from "../player/WebAudioPlayer";
    import Seekbar from "../sidebar/Seekbar.svelte";
    import Icon from "../ui/Icon.svelte";

    export let item: IAItem;

    let currentTime = 0;

    $: elapsedTime = `${(~~(currentTime / 60))
        .toString()
        .padStart(2, "0")}:${(~~(currentTime % 60))
        .toString()
        .padStart(2, "0")}`;

    $: durationText = `${(~~(item.duration / 60)).toString().padStart(2, "0")}:${(~~(
        item.duration % 60
    ))
        .toString()
        .padStart(2, "0")}`;

    onMount(() => {
        if ($currentSrc === item.previewSrc) {
            currentTime = audioPlayer.audioFile.currentTime;
        }
        audioPlayer.onTimeUpdate = (time) => {
            if ($currentSrc === item.previewSrc) {
                currentTime = time;
            } else {
                currentTime = 0;
            }
        };
    });
</script>

<div class="container">
    <Icon
        icon={$currentSrc === item.previewSrc && $isPlaying
            ? "fe:pause"
            : "fe:play"}
        color="#ded2de"
        size={30}
        onClick={() => {
            if ($currentSrc === item.previewSrc) {
                audioPlayer.togglePlay();
            } else audioPlayer.playFromUrl(item);
        }}
    />
    <p class="elapsed-time">{elapsedTime} / {durationText}</p>
    <div class="seekbar">
        <Seekbar
            playerTime={currentTime}
            onSeek={(time) => {
                audioPlayer.audioFile.currentTime = time;
            }}
            duration={item.duration}
        />
    </div>
</div>

<style lang="scss">
    .container {
        display: flex;
        padding: 1em;
        border-radius: 5px;
        padding: 10px;
        gap: 7px;
        align-items: center;
        .elapsed-time {
            opacity: 0.5;
            font-size: 12px;
            margin: 0;
        }
        .seekbar {
            flex: 1;
        }
    }
</style>
