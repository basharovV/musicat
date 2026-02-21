<script lang="ts">
    import tippy from "svelte-tippy";
    import "../tippy.css";

    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";

    import LL from "../../i18n/i18n-svelte";
    import { getImageExtension } from "../../utils/FileUtils";

    export let artworkFocused = false;
    export let shouldDisplayArtwork = false;
    export let displayArtworkSrc = null;
    export let artworkSrc = null;
    export let artworkFormat = null;
    export let artworkToSetSrc = null;
    export let artworkToSetFormat = null;

    $: artworkExtension = getImageExtension(artworkToSetFormat);

    export let deleteArtwork = () => {};
    export let openArtworkFilePicker = () => {};
    export let artworkSetAction;
    export let isMultiArt = false;
    export let artworkFileName = null;
    export let loadingType;
    export let fetchArtwork = () => {};
    export let searchArtwork = () => {};
    export let saveArtworkToFile = () => {};
    export let saveArtworkToFolder = () => {};
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<section class="artwork-section boxed">
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="left">
        <div
            class="artwork-container"
            class:focused={artworkFocused}
            on:mouseenter={() => (artworkFocused = true)}
            on:mouseleave={() => (artworkFocused = false)}
            use:tippy={{
                content: $LL.trackInfo.artworkTooltip(),
                placement: "bottom",
                trigger: "focusin",
            }}
        >
            <div
                class="artwork-frame"
                use:tippy={{
                    content: $LL.trackInfo.artwork.pasteTooltip(),
                    placement: "right",
                }}
            >
                {#if shouldDisplayArtwork}
                    <img alt="" class="artwork" src={displayArtworkSrc} />
                    {#if artworkFocused && artworkSrc && artworkFormat}
                        <div class="artwork-options">
                            <Icon
                                icon={artworkToSetSrc
                                    ? "mingcute:close-circle-fill"
                                    : "ant-design:delete-outlined"}
                                onClick={deleteArtwork}
                            />
                            <Icon
                                icon="material-symbols:folder"
                                onClick={openArtworkFilePicker}
                            />
                        </div>
                    {/if}
                {:else}
                    <div class="artwork-placeholder">
                        {#if artworkFocused}
                            <div
                                class="artwork-options"
                                on:click={openArtworkFilePicker}
                            >
                                <Icon icon="material-symbols:folder" />
                            </div>
                        {:else}
                            <Icon icon="mdi:music-clef-treble" />
                        {/if}
                    </div>
                {/if}
            </div>
        </div>
        {#if artworkSetAction}
            <small>{$LL.trackInfo.artworkReadyToSave()}</small>
        {:else if isMultiArt}
            <small>{$LL.trackInfo.multiArtwork()}</small>
        {:else if artworkFileName}
            <small>{artworkFileName}</small>
        {:else if artworkSrc}
            <small>{$LL.trackInfo.encodedInFile()}</small>
        {:else}
            <small class="notfound">{$LL.trackInfo.noArtwork()}</small>
        {/if}
        <span
            use:tippy={{
                allowHTML: true,
                content: $LL.trackInfo.artworkTooltipBody(),
                placement: "left",
            }}
            ><Icon icon="ic:round-info" /><small
                >{$LL.trackInfo.aboutArtwork()}</small
            ></span
        >
    </div>
    <div class="artwork-actions">
        <div class="fetch-options">
            <ButtonWithIcon
                icon={loadingType === "artwork"
                    ? "line-md:loading-loop"
                    : "ic:twotone-downloading"}
                text={$LL.trackInfo.artwork.fetchButton.title()}
                theme="transparent"
                onClick={fetchArtwork}
                fullWidth
                tooltip={{
                    content: $LL.trackInfo.artwork.fetchButton.tooltip(),
                    placement: "bottom",
                }}
            />
            <ButtonWithIcon
                icon="mdi:search-web"
                text={$LL.trackInfo.artwork.searchButton.title()}
                theme="transparent"
                onClick={searchArtwork}
                fullWidth
                tooltip={{
                    content: $LL.trackInfo.artwork.searchButton.tooltip(),
                    placement: "bottom",
                }}
            />
        </div>
        <div class="save-options">
            <ButtonWithIcon
                icon="material-symbols:save-outline"
                text={$LL.trackInfo.artwork.saveButton.file()}
                theme="translucent"
                onClick={saveArtworkToFile}
                fullWidth
                disabled={!artworkSetAction}
            />
            <ButtonWithIcon
                icon="material-symbols:folder"
                text={artworkFileName && artworkSetAction === "delete"
                    ? $LL.trackInfo.artwork.saveButton.deleteFolderArt({
                          file: artworkFileName,
                      })
                    : $LL.trackInfo.artwork.saveButton.folder({
                          file: artworkExtension
                              ? `cover.${artworkExtension}`
                              : `cover.jpg`,
                      })}
                theme="translucent"
                onClick={saveArtworkToFolder}
                fullWidth
                disabled={!artworkSetAction ||
                    (artworkSetAction === "delete" && !artworkFileName) ||
                    isMultiArt}
            />
        </div>
    </div>
</section>

<style lang="scss">
    .artwork-section {
        width: 100%;
        box-sizing: border-box;
        padding: 1em;
        align-items: start;
        display: flex;
        flex-direction: row;
        gap: 1em;
        .left > small {
            color: var(--popup-track-artwork-found);
            white-space: nowrap;
            text-overflow: ellipsis;
            overflow: hidden;
            display: block;

            &.notfound {
                color: var(--popup-track-artwork-notfound);
            }
        }
        span {
            cursor: default;
            margin: 0.3em auto 0;
            width: fit-content;
            display: flex;
            gap: 5px;
            align-items: center;
            color: var(--popup-track-artwork-about);
            user-select: none;
            &:hover {
                opacity: 0.7;
            }

            &:active {
                opacity: 0.5;
            }
        }
    }

    .artwork-container {
        padding: 0em;
        width: 160px;
        height: 160px;
        border-radius: 4px;
        overflow: hidden;
        cursor: pointer;
        caret-color: transparent;
        border: 1px solid
            color-mix(in srgb, var(--background) 60%, var(--inverse));

        &:hover {
            border: 1px solid rgb(from var(--inverse) r g b / 0.517);
        }

        &.focused {
            background-image: linear-gradient(
                    90deg,
                    silver 50%,
                    transparent 50%
                ),
                linear-gradient(90deg, silver 50%, transparent 50%),
                linear-gradient(0deg, silver 50%, transparent 50%),
                linear-gradient(0deg, silver 50%, transparent 50%);
            background-repeat: repeat-x, repeat-x, repeat-y, repeat-y;
            background-size:
                15px 2px,
                15px 2px,
                2px 15px,
                2px 15px;
            background-position:
                left top,
                right bottom,
                left bottom,
                right top;
            animation: border-dance 2s infinite linear;

            .artwork-frame {
                img {
                    transform: scale(0.96);
                }
            }
        }

        @keyframes border-dance {
            0% {
                background-position:
                    left top,
                    right bottom,
                    left bottom,
                    right top;
            }

            100% {
                background-position:
                    left 15px top,
                    right 15px bottom,
                    left bottom 15px,
                    right top 15px;
            }
        }

        .artwork-frame {
            width: 100%;
            height: 100%;
            box-sizing: border-box;
            user-select: none;
            /* border-radius: 3px; */
            /* border: 1px solid rgb(94, 94, 94); */
            display: flex;
            align-items: center;
            justify-content: center;
            position: relative;
            img {
                height: 100%;
                width: 100%;
            }

            .artwork-options {
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: row;
                align-items: center;
                justify-content: center;
                gap: 2em;
                background-color: var(--background);
            }

            .artwork-placeholder {
                opacity: 0.2;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 1em;
            }
        }
    }

    .artwork-actions {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        gap: 1em;
        flex-grow: 1;

        .fetch-options {
            display: flex;
            flex-direction: row;
        }

        .save-options {
            margin-top: 1em;
        }

        > div {
            display: flex;
            flex-direction: column;
            text-align: left;
            gap: 1em;
            line-height: normal;
            align-items: center;
            small {
                max-width: 250px;
                padding: 0 4px;
                opacity: 0.5;
            }
        }
    }
</style>
