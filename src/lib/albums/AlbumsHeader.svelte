<script lang="ts">
    import { onMount } from "svelte";
    import {
        isQueueOpen,
        isSidebarOpen,
        os,
        uiPreferences,
    } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import Divider from "../ui/Divider.svelte";
    import Dropdown from "../ui/Dropdown.svelte";
    import Icon from "../ui/Icon.svelte";
    import { throttle } from "lodash-es";
    import Menu from "../ui/menu/Menu.svelte";
    import MenuDivider from "../ui/menu/MenuDivider.svelte";

    let element: HTMLElement;
    let optionsPanelPosition = { x: 0, y: 0 };
    let showOptionsPanel = false;

    $: innerWidth = 800;
    $: showInfo = $uiPreferences.albumsViewShowInfo;
    $: minWidth = $uiPreferences.albumsViewGridSize;
    $: showAllOptions = innerWidth >= 480;
    $: showSingles = $uiPreferences.albumsViewShowSingles;

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

    onMount(async () => {
        window.onresize = throttle(() => {
            onResize();
        }, 200);

        onResize(); // run once
    });

    function onResize() {
        innerWidth = element.getBoundingClientRect().width;
    }

    function toogleOptionsPanel(e) {
        if (showOptionsPanel) {
            showOptionsPanel = false;
        } else {
            optionsPanelPosition = { x: e.clientX, y: e.clientY };
            showOptionsPanel = true;
        }
    }
</script>

<div class="header" data-tauri-drag-region bind:this={element}>
    <h1
        class:window-controls-offset={!$isSidebarOpen &&
            !$isQueueOpen &&
            $os === "macos"}
        data-tauri-drag-region
    >
        {$LL.albums.title()}
    </h1>
    <div class="options" data-tauri-drag-region>
        {#if showAllOptions}
            <div class="order-by">
                <Icon
                    icon="heroicons-solid:sort-descending"
                    color="var(--icon-primary)"
                    boxed
                />
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
                    icon={showSingles
                        ? "mynaui:one-circle-solid"
                        : "mynaui:one-circle-solid-off"}
                    color={showSingles
                        ? "var(--icon-primary)"
                        : "var(--icon-secondary)"}
                    boxed
                    onClick={() => {
                        $uiPreferences.albumsViewShowSingles = !showSingles;
                    }}
                />
            </div>
            <div class="toggle">
                <Icon
                    icon={showInfo ? "mdi:information" : "mdi:information-off"}
                    color={showInfo
                        ? "var(--icon-primary)"
                        : "var(--icon-secondary)"}
                    boxed
                    onClick={() => {
                        $uiPreferences.albumsViewShowInfo = !showInfo;
                    }}
                />
            </div>
            <Divider />
            <label>
                <Icon
                    icon="material-symbols:grid-view-rounded"
                    color="var(--icon-primary)"
                    boxed
                />
                <input
                    type="range"
                    min={100}
                    max={400}
                    value={minWidth}
                    on:input={(ev) => {
                        $uiPreferences.albumsViewGridSize = Number(
                            ev.target.value,
                        );
                    }}
                />
            </label>
        {:else}
            <Icon
                icon="iconamoon:settings-fill"
                color="var(--icon-primary)"
                boxed
                onClick={toogleOptionsPanel}
            />
        {/if}
    </div>

    {#if showOptionsPanel}
        <div class="options-panel">
            <Menu {...optionsPanelPosition} onClickOutside={toogleOptionsPanel}>
                <div class="order-by">
                    <Icon
                        icon="heroicons-solid:sort-descending"
                        color="var(--icon-primary)"
                        boxed
                    />
                    <Dropdown
                        options={fields}
                        selected={orderBy}
                        onSelect={(v) => {
                            $uiPreferences.albumsViewSortBy = v;
                        }}
                    />
                </div>
                <MenuDivider />
                <div class="controls">
                    <div class="toggle">
                        <Icon
                            icon={showSingles
                                ? "mynaui:one-circle-solid"
                                : "mynaui:one-circle-solid-off"}
                            color={showSingles
                                ? "var(--icon-primary)"
                                : "var(--icon-secondary)"}
                            boxed
                            onClick={() => {
                                $uiPreferences.albumsViewShowSingles =
                                    !showSingles;
                            }}
                        />
                    </div>
                    <div class="toggle">
                        <Icon
                            icon={showInfo
                                ? "mdi:information"
                                : "mdi:information-off"}
                            color={showInfo
                                ? "var(--icon-primary)"
                                : "var(--icon-secondary)"}
                            boxed
                            onClick={() => {
                                $uiPreferences.albumsViewShowInfo = !showInfo;
                            }}
                        />
                    </div>
                </div>
                <MenuDivider />
                <label>
                    <input
                        type="range"
                        min={100}
                        max={400}
                        value={minWidth}
                        on:input={(ev) => {
                            $uiPreferences.albumsViewGridSize = Number(
                                ev.target.value,
                            );
                        }}
                    />
                    <Icon
                        icon="material-symbols:grid-view-rounded"
                        color="var(--icon-primary)"
                        boxed
                    />
                </label>
            </Menu>
        </div>
    {/if}
</div>

<style lang="scss">
    .header {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: flex-end;
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
            gap: 10px;
            align-items: center;
            margin: 0 5px 0 20px;
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
    }

    .options-panel {
        .order-by {
            display: flex;
            gap: 3px;
            color: var(--text-secondary);
        }
        .controls {
            display: flex;
            justify-content: center;
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
</style>
