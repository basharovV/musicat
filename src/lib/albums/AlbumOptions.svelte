<script lang="ts">
    import { uiPreferences } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import Divider from "../ui/Divider.svelte";
    import Dropdown from "../ui/Dropdown.svelte";
    import Icon from "../ui/Icon.svelte";

    $: showSingles = $uiPreferences.albumsViewShowSingles;
    $: showInfo = $uiPreferences.albumsViewShowInfo;
    $: minWidth = $uiPreferences.albumsViewGridSize;
    const fields = [
        {
            value: "title",
            label: $LL.albums.options.orderByFields.title(),
        },
        {
            value: "artist",
            label: $LL.albums.options.orderByFields.artist(),
        },
        {
            value: "year",
            label: $LL.albums.options.orderByFields.year(),
        },
    ];

    $: orderBy = fields.find(
        (f) => f.value === $uiPreferences.albumsViewSortBy,
    );
</script>

<div class="options" data-tauri-drag-region>
    <div class="order-by">
        <p>{$LL.albums.options.orderBy()}</p>
        <Dropdown
            options={fields}
            selected={orderBy}
            onSelect={(v) => {
                $uiPreferences.albumsViewSortBy = v;
            }}
        />
    </div>
    <Divider />
    <div class="toggle">
        <Icon
            icon={showSingles ? "mdi:show" : "mdi:hide"}
            color={showSingles
                ? "var(--icon-primary)"
                : "var(--icon-secondary)"}
            boxed
            onClick={() => {
                $uiPreferences.albumsViewShowSingles = !showSingles;
            }}
        />
        <p>{$LL.albums.options.showSingles()}</p>
    </div>
    <div class="toggle">
        <Icon
            icon={showInfo ? "mdi:show" : "mdi:hide"}
            color={showInfo ? "var(--icon-primary)" : "var(--icon-secondary)"}
            boxed
            onClick={() => {
                $uiPreferences.albumsViewShowInfo = !showInfo;
            }}
        />
        <p>{$LL.albums.options.showInfo()}</p>
    </div>
    <Divider />
    <label
        >{$LL.albums.options.gridSize()}
        <input
            type="range"
            min={100}
            max={400}
            value={minWidth}
            on:input={(ev) => {
                $uiPreferences.albumsViewGridSize = Number(ev.target.value);
            }}
        /></label
    >
</div>

<style lang="scss">
    .options {
        display: flex;
        gap: 10px;
        align-items: center;
        margin-right: 5px;
        * {
            margin: 0;
            line-height: initial;
            vertical-align: middle;
        }

        .order-by {
            display: flex;
            gap: 3px;
            color: var(--text-secondary);
        }
        .toggle {
            display: flex;
            align-items: center;
            color: var(--text-secondary);
            cursor: default;
            :global(svg) {
                opacity: 0.7;
            }
        }
    }

    label {
        display: flex;
        flex-direction: row-reverse;
        gap: 4px;
        align-items: center;
        color: var(--text-secondary);
    }
    input[type="checkbox"] {
        padding: 0;
        margin: 0;
    }
    input[type="range"] {
        appearance: none;
        outline: none;
        border: none;
        box-shadow: none;
        max-width: 100px;
        &::-webkit-slider-thumb {
            appearance: none;
            background-color: rgb(132, 175, 166);
            border-radius: 2px;
            color: red;
            width: 10px;
            height: 10px;
            top: -2.5px;
            left: 0;
            position: relative;
        }
        &::-webkit-slider-runnable-track {
            background-color: var(--icon-secondary);
            appearance: none;
            border-radius: 10px;
            outline: none;
            height: 4px;
        }
        ::-moz-range-track {
            background: #ade8ff;
            height: 4px;
        }
    }
</style>
