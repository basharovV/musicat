<script lang="ts">
    import AudioMotionAnalyzer from "audiomotion-analyzer";
    import { currentSong, volume } from "../data/store";
    import { onMount } from "svelte";
    import audioPlayer from "./AudioPlayer";

    let container;
    let audioMotion: AudioMotionAnalyzer;
    let source: HTMLAudioElement;

    currentSong.subscribe((song) => {
        // if (audioMotion?.connectedSources?.length)
        //   audioMotion?.disconnectInput(audioMotion.connectedSources[0]);
        // audioMotion?.connectInput(audioPlayer.audioFile);
    });
    volume.subscribe((vol) => {
        if (audioMotion) audioMotion.volume = vol;
        localStorage.setItem("volume", String(vol));
    });
    onMount(() => {
        source = audioPlayer.audioFile;
        audioMotion = new AudioMotionAnalyzer(container, {
            source: source,
            volume: source.volume,
            barSpace: 0.1,
            connectSpeakers: true,
            ledBars: true,
            showPeaks: true,
            height: 100,
            mode: 7,
            showBgColor: false,
            bgAlpha: 0,
            overlay: true,
            showScaleX: false
        });
    });
</script>

<div bind:this={container} />
