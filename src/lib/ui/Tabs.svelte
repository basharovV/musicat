<script lang="ts">
    import ButtonWithIcon from "./ButtonWithIcon.svelte";

    export let tabs = [];
    export let currentTab;
</script>

<nav class="tabs-nav">
    {#each tabs as tab}
        <ButtonWithIcon
            size="small"
            theme={tab.id === currentTab ? "active" : "transparent"}
            onClick={() => (currentTab = tab.id)}
            text={tab.label}
            icon={tab.icon}
        />
    {/each}
</nav>

<style lang="scss">
    .tabs-nav {
        display: flex;
        width: 100%;
        border-bottom: 1px solid var(--popup-header-border);
        padding: 0.5em 1em;
        gap: 5px;
        align-items: center;
        justify-content: center;
    }
</style>
