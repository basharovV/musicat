<script>
    import Waveform from "../player/Waveform.svelte";
</script>

<div>
    <!-- <Seekbar duration={$currentSong?.fileInfo?.duration} /> -->
    <Waveform />
</div>

<style lang="scss">
    div {
        min-height: 80px;
        border-radius: 5px;
        border: 0.7px solid #ffffff1f;
        margin: 5px 5px 0 0;
        background-color: rgba(54, 48, 57, 0.164);
        /* box-shadow: inset darken(#3e3b3b49, 15%) 0 0.5em 0px 0; */


        &:hover {
            background-color: #9c86b10f;
        }
    }
</style>