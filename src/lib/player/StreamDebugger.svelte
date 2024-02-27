<script lang="ts">
    import { streamInfo } from "../../data/store";



</script>

<div>
    <h3>WebRTC Stream info</h3>
    <p>received: {($streamInfo.bytesReceived / 1000000).toFixed(2)} MB</p>
    <p>Rate: {$streamInfo.receiveRate.toFixed(2)} KB/s</p>
    <p>buffered samples: {$streamInfo.bufferedSamples}</p>
    <p>played samples: {$streamInfo.playedSamples}</p>
    <p>timestamp: {$streamInfo.timestamp}</p>
</div>

<style lang="scss">
    h3 {
        margin: 0;
    }
    p {
        margin: 0;
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