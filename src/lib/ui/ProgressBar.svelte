<script lang="ts">

    // seconds
    export let percent = 0;

    let seekBar;

    let showHoverHead = false;
</script>

<div class="container">
    <div class="seekbar-container">
        <div class="bg" bind:this={seekBar} class:hovered={showHoverHead}></div>
        <div
            class="progress"
            bind:this={seekBar}
            class:hovered={showHoverHead}
            style="width:{percent}%;"
        ></div>
    </div>
</div>

<style lang="scss">
    .container {
        display: flex;
        width: 100%;
        flex-direction: column;
        height: auto;
        padding: 0;
        align-items: center;
        justify-content: space-between;

        /* background: rgba(0, 0, 0, 0.954); */
        color: white;
        /* box-shadow: -5px -10px 5px rgba(0, 0, 0, 0.071); */
        z-index: 4;
    }

    p {
        margin: 0;
    }

    @keyframes scroll {
        from {
            left: 0%;
        }
        to {
            left: -100%;
        }
    }

    .seekbar-container {
        width: 100%;
        height: 15px;
        align-items: center;
        display: flex;
        position: relative;
        margin-top: 1em;
        /* cursor: ew-resize; */
    }

    .bg {
        background: rgba(255, 255, 255, 0.262);
        height: 2px;
        width: 100%;
        overflow: visible;
        align-items: center;
        display: flex;
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;

        svg {
            height: 10px;
            width: 10px;
            fill: rgb(0, 197, 108);
            position: absolute;
            overflow: visible;
            pointer-events: none;
            transition: all 0.16s cubic-bezier(0.075, 0.82, 0.165, 1);
            top: -4px;
        }
    }

    .progress {
        background: #23DD7F;
        height: 2px;
        width: 100%;
        overflow: visible;
        align-items: center;
        display: flex;
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        transition: width 0.3s ease-in-out;

        svg {
            height: 10px;
            width: 10px;
            fill: rgb(0, 197, 108);
            position: absolute;
            overflow: visible;
            pointer-events: none;
            transition: all 0.16s cubic-bezier(0.075, 0.82, 0.165, 1);

            top: -4px;
        }
    }
</style>
