<script>
    export let value, min, max;
    export let rotRange = 2 * Math.PI * 0.83;
    export let pixelRange = 200;
    export let startRotation = -Math.PI * 0.83;

    let startY, startValue;
    $: valueRange = max - min;
    $: rotation = startRotation + ((value - min) / valueRange) * rotRange;

    function clamp(num, min, max) {
        return Math.max(min, Math.min(num, max));
    }

    function pointerMove({ clientY }) {
        const valueDiff = (valueRange * (clientY - startY)) / pixelRange;
        value = clamp(startValue - valueDiff, min, max);
    }

    function pointerDown({ clientY }) {
        console.log({ clientY });
        startY = clientY;
        startValue = value;
        window.addEventListener("pointermove", pointerMove);
        window.addEventListener("pointerup", pointerUp);
    }

    function pointerUp() {
        window.removeEventListener("pointermove", pointerMove);
        window.removeEventListener("pointerup", pointerUp);
    }
</script>

<div class="knob" style="--rotation: {rotation}" on:pointerdown={pointerDown} />

<style>
    .knob {
        display: block;
        width: 80px;
        height: 80px;
        padding: 0;
        border-radius: 50%;
        /* background-image: conic-gradient(white 0%, white 2%, black 2%, black 98%, white 98%, white 100%); */
        background-image: url("images/volume-knob.png");
        background-size: cover;
        transform: rotate(calc(var(--rotation) * 1rad));
        transform-origin: 50% 50%;
        box-shadow: 0px 2px 5px 4px rgba(0, 0, 0, 0.315);
    }
</style>
