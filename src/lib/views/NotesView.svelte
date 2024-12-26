<script>
    import Waveform from "../player/Waveform.svelte";
</script>

<div>
    <!-- <Seekbar duration={$currentSong?.fileInfo?.duration} /> -->
    <Waveform />
</div>

<style lang="scss">
    div {
        border-radius: 5px;
        border-left: 0.7px solid #ffffff13;
        border-right: 0.7px solid #ffffff09;
        border-bottom: 0.7px solid #ffffff1f;
        border-top: 0.7px solid #ffffff0e;
        margin: 5px 5px 0 0;
        background-color: var(--waveform-bg);


        &:hover {
            background-color: var(--waveform-bg-hover);
        }
    }
</style>