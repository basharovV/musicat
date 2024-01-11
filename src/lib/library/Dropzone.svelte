<div class="dropzone">
    <img alt="Drop the mic" src="images/drop-the-mic.gif" />
</div>

<style lang="scss">
    .dropzone {
        display: flex;
        align-items: center;
        justify-content: center;
        position: absolute;
        width: 80vw;
        height: 80vh;
        left: 0;
        right: 0;
        top: 0;
        bottom: 0;
        margin: auto;
        background-color: #00a4ba;
        z-index: 20;
        border: 3px dashed rgb(211, 57, 57);
        border-radius: 5px;
        opacity: 1;
        transition: opacity 0.2s ease-in-out;
        * {
            pointer-events: none;
        }
        img {
            width: 300px;
        }
    }
</style>
