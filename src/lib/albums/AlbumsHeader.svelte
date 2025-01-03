<script lang="ts">
    import {
        isQueueOpen,
        isSidebarOpen,
        os,
        uiPreferences
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import Dropdown from "../ui/Dropdown.svelte";

    $: showSingles = $uiPreferences.albumsViewShowSingles;
    $: showInfo = $uiPreferences.albumsViewShowInfo;
    $: minWidth = $uiPreferences.albumsViewGridSize;
    const fields = [
        {
            value: "title",
            label: $LL.albums.options.orderByFields.title()
        },
        {
            value: "artist",
            label: $LL.albums.options.orderByFields.artist()
        },
        {
            value: "year",
            label: $LL.albums.options.orderByFields.year()
        }
    ];

    $: orderBy = fields.find(
        (f) => f.value === $uiPreferences.albumsViewSortBy
    );
</script>

<div class="header" data-tauri-drag-region>
    <h1
        class:window-controls-offset={!$isSidebarOpen &&
            !$isQueueOpen &&
            $os === "macos"}
        data-tauri-drag-region
    >
        {$LL.albums.title()}
    </h1>
    <!-- {#if count}<p>{count} {count === 1 ? "album" : "albums"}</p>{/if} -->
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
        <label
            >{$LL.albums.options.showSingles()}
            <input
                type="checkbox"
                checked={showSingles}
                on:change={(ev) => {
                    $uiPreferences.albumsViewShowSingles = ev.target.checked;
                }}
            /></label
        >
        <label
            >{$LL.albums.options.showInfo()}
            <input
                type="checkbox"
                checked={showInfo}
                on:change={(ev) => {
                    $uiPreferences.albumsViewShowInfo = ev.target.checked;
                }}
            /></label
        >
        <label
            >{$LL.albums.options.gridSize()}
            <input
                type="range"
                min={100}
                max={400}
                value={minWidth}
                on:input={(ev) => {
                    $uiPreferences.albumsViewGridSize = ev.target.value;
                }}
            /></label
        >
    </div>
</div>

<style lang="scss">
    .header {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 20px;
        grid-column: 1 /3;
        grid-row: 1;

        h1 {
            margin: 0;
            font-family: "Snake";
            flex-grow: 1;
            text-align: left;
            font-size: 3em;
            opacity: 0.3;
            margin-left: 0.1em;
            color: var(--header-text);
            opacity: var(--header-opacity);

            &.window-controls-offset {
                margin-left: 70px;
            }
        }

        .options {
            display: flex;
            gap: 20px;
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
    }
</style>
