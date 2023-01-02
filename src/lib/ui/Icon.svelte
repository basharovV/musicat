<script lang="ts">
    export let icon;
    export let color = "white";
    export let onClick = (e: MouseEvent) => {};
</script>

<iconify-icon {icon} style={`color:${color}`} on:click={onClick} />

<style lang="scss">
    iconify-icon {
        padding: 6px;
        font-size: 15px;
        border-radius: 4px;
        &:hover {
            background-color: rgba(0, 0, 0, 0.187);
        }
    }
</style>
