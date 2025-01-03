<script lang="ts">
    import { query } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
</script>

<div>
    <h2>{$LL.library.queryNoResultsPlaceholder({ query: $query.query })}</h2>
</div>

<style lang="scss">
    div {
        position: absolute;
        top: 0;
        bottom: 0;
        left: 0;
        right: 0;
        display: flex;
        margin: auto;
        height: fit-content;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        z-index: 12;
        small {
            margin-top: 1em;
            opacity: 0.5;
        }
    }
</style>
