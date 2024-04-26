<script>
    import Waveform from "../player/Waveform.svelte";
</script>

<div>
    <!-- <Seekbar duration={$currentSong?.fileInfo?.duration} /> -->
    <Waveform />
</div>

<style lang="scss">
    div {
        min-height: 70px;
        border-radius: 5px;
        border-left: inset 0.7px solid #ffffff1f;
        border-bottom: 0.7px solid #ffffff1f;
        margin: 5px 5px 0 0;
        background-color: #9c86b10f;


        &:hover {
            background-color: #9c86b11c;
        }
    }
</style>