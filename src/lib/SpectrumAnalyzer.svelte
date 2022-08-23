<script lang="ts">
  import AudioMotionAnalyzer from "audiomotion-analyzer";
  import { currentSong } from "../data/store";
  import { onMount } from "svelte";
  import audioPlayer from "./AudioPlayer";

  let container;
  let audioMotion: AudioMotionAnalyzer;
  let source: MediaElementAudioSourceNode;

  currentSong.subscribe((song) => {
    // if (audioMotion?.connectedSources?.length)
    //   audioMotion?.disconnectInput(audioMotion.connectedSources[0]);
    // audioMotion?.connectInput(audioPlayer.audioFile);
  });

  onMount(() => {
    source = audioPlayer.source;
    audioMotion = new AudioMotionAnalyzer(container, {
      source: source,
      barSpace: 0.1,
      connectSpeakers: false,
      ledBars: true,
      showPeaks: true,
      height: 100,
      mode: 7,
      showBgColor: false,
      bgAlpha: 0,
      overlay: true,
      showScaleX: false
    });
    if (audioMotion?.connectedSources?.length)
      audioMotion?.disconnectOutput(audioMotion.connectedSources[0]);
  });
</script>

<div bind:this={container} />
