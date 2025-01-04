<script lang="ts">
    import { current, streamInfo } from "../../data/store";

    $: playbackRate =
        $current.song?.fileInfo?.bitDepth *
        $current.song?.fileInfo?.sampleRate *
        $current.song?.fileInfo?.channels;

    let diff = $streamInfo.receiveRate / 1000 - playbackRate;
    let status = "ok";
    if (diff > 0 && diff < 10000) {
        status = "good";
    } else if (diff < 0) {
        status = "bad";
    }

    $: bufferedTime = (
        $streamInfo.bufferedSamples / $current.song?.fileInfo?.sampleRate ?? 0
    ).toFixed(2);
</script>

<div>
    <h3>WebRTC Stream info</h3>
    <p>Received: {($streamInfo.bytesReceived / 1000000).toFixed(2)} MB</p>
    <p>Playback rate: {playbackRate / 1000} kbits/s</p>
    <p>
        Receive rate: <span class={status}
            >{($streamInfo.receiveRate / 1000).toFixed(2)} kbits/s</span
        >
    </p>
    <p>Buffered samples: {$streamInfo.bufferedSamples}</p>
    <p>Buffered time: {bufferedTime} seconds</p>
    <p>Played samples: {$streamInfo.playedSamples}</p>
    <p>Sample index: {$streamInfo.sampleIdx}</p>
    <p>Played time: {$streamInfo.timestamp.toFixed(2)} seconds</p>
</div>

<style lang="scss">
    h3 {
        margin: 0;
    }
    p {
        margin: 0;
        text-align: left;
    }

    span {
        &.good {
            color: green;
        }
        &.bad {
            color: red;
        }
    }
    div {
        position: fixed;
        top: 0;
        right: 0;
        width: 300px;
        margin: 1em;
        border: 2px solid rgb(159, 7, 186);
        background: rgba(36, 38, 38, 0.999);
        color: white;
        z-index: 100;
        padding: 1em;
        border-radius: 5px;
    }
</style>
