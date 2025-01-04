<!-- Rounded switch -->
<script lang="ts">
    export let checked = false;
    export let textOff = "ON";
    export let textOn = "OFF";
</script>

<div class="container">
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <p class:active={!checked} on:click={() => (checked = false)}>{textOff}</p>
    <label class="switch">
        <input type="checkbox" bind:checked />
        <span class="slider round"></span>
    </label>
    <!-- svelte-ignore a11y-no-noninteractive-element-to-interactive-role -->
    <p role="button" class:active={checked} on:click={() => (checked = true)}>
        {textOn}
    </p>
</div>

<style lang="scss">
    /* The switch - the box around the slider */
    $size: 12px;

    .container {
        display: flex;
        align-items: center;
        gap: 10px;
        p {
            margin: 0;
            font-size: 12px;
            opacity: 0.6;
            cursor: default;
            &.active {
                opacity: 1;
            }
        }
    }
    .switch {
        position: relative;
        display: inline-block;
        width: $size * 2;
        height: $size;
    }

    /* Hide default HTML checkbox */
    .switch input {
        opacity: 0;
        width: 0;
        height: 0;
    }

    /* The slider */
    .slider {
        position: absolute;
        cursor: pointer;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        /* background-color: color-mix(in srgb, var(--type-bw-inverse) 10%, transparent); */
        border: 1px solid
            color-mix(in srgb, var(--type-bw-inverse) 30%, transparent);
        -webkit-transition: 0.4s;
        transition: 0.4s;
    }

    .slider:before {
        position: absolute;
        content: "";
        height: $size * 0.8;
        width: $size * 0.8;
        left: 0px;
        bottom: 0px;
        background-color: var(--accent);
        -webkit-transition: 0.4s;
        transition: 0.4s;
    }

    input:checked + .slider:before {
        transform: translateX($size);
    }

    /* Rounded sliders */
    .slider.round {
        border-radius: 34px;
    }

    .slider.round:before {
        border-radius: 50%;
    }
</style>
