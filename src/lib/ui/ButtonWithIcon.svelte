<script lang="ts">
    export let icon;
    export let onClick;
    export let text;
</script>

<div on:click={onClick}>
    <iconify-icon class="close-icon" icon={icon} />
    <p>{text}</p>
</div>

<style lang="scss">
    div {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 5px;
        /* margin-left: 1em; */
        font-weight: normal;
        /* background-color: rgba(240, 248, 255, 0.088); */
        border: 1px solid rgba(255, 255, 255, 0.2);
        padding: 0.3em 0.5em;
        border-radius: 4px;
        color: rgba(255, 255, 255, 0.811);
        white-space: nowrap;
        p {
            margin: 0;
        }
        &:active {
            opacity: 0.8;
        }
        &:hover {
            background-color: rgba(240, 248, 255, 0.088);
        }
    }
</style>
