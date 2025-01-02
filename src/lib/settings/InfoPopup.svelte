<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";

    import { clickOutside } from "../../utils/ClickOutside";
    import Icon from "../ui/Icon.svelte";
    import ReleaseNotes from "./ReleaseNotes.svelte";

    import { _ } from "svelte-i18n";
    import LL from "../../i18n/i18n-svelte";
    let version = getVersion();
    export let onClickOutside;
    export let isReleaseNotesShown = false;
    let scrollToHeader: HTMLElement;
    let popup: HTMLElement;
</script>

<container>
    <div
        class="popup"
        class:expanded={isReleaseNotesShown}
        use:clickOutside={onClickOutside}
        bind:this={popup}
    >
        <!-- <img src="images/cd6.gif" /> -->
        <section class="info">
            <div>
                <h1>Musicat</h1>
            </div>
            <div class="app-icon">
                <img src="icon.png" />
            </div>
            <div class="dev-info">
                <small
                    >{$_("infoPopup.buildBy")}<a
                        href="https://vyacheslavbasharov.com"
                        target="_blank">Slav</a
                    ></small
                >
                <!-- <br /> -->
                {#await version then versionValue}<small>
                        {$LL.infoPopup.version()} {versionValue}</small
                    >{/await}
            </div>
        </section>
        <section class="release-notes">
            <div
                class="release-notes-title"
                bind:this={scrollToHeader}
                on:click|stopPropagation={() => {
                    if (!isReleaseNotesShown) {
                        isReleaseNotesShown = true;
                        setTimeout(() => {
                            scrollToHeader.scrollIntoView({
                                behavior: "smooth",
                            });
                        }, 10);
                    } else {
                        popup.scrollTo({
                            top: 0,
                            behavior: "smooth",
                        });
                        setTimeout(() => {
                            isReleaseNotesShown = false;
                        }, 150);
                    }
                }}
            >
                <h3>{$_("infoPopup.releaseNotes")}</h3>
                <div class="chevron" class:expanded={isReleaseNotesShown}>
                    <Icon icon="lucide:chevron-down" size={14} />
                </div>
            </div>
            {#if isReleaseNotesShown}
                <ReleaseNotes />
            {/if}
        </section>
    </div></container
>

<style lang="scss">
    container {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: rgba(0, 0, 0, 0.187);
    }
    .popup {
        min-width: 500px;
        max-width: 700px;
        min-height: 410px;
        max-height: 0;
        border-radius: 5px;
        position: relative;
        /* padding: 2em 0em 0; */
        background: rgb(74, 72, 77);
        color: white;
        border: 1px solid rgb(114, 114, 114);
        display: flex;
        flex-direction: column;
        flex: 10em;
        overflow: auto;
        transition: max-height 0.25s ease-in-out;
        &.expanded {
            max-height: 700px;
        }
    }

    .info {
        display: flex;
        flex-direction: column;
        grid-template-rows: auto 1fr 1fr;
        align-items: center;

        .app-icon {
            height: 157px;
            pointer-events: none;
            img {
                width: 150px;
                height: 100%;
            }
        }
        h1 {
            font-family: "2Peas";
            margin-bottom: 0;
        }

        small {
            opacity: 0.5;
        }

        .dev-info {
            height: 100%;
            display: flex;
            flex-direction: column;
            justify-content: flex-end;
            padding: 2em;
        }
    }

    .release-notes {
        border-top: 1px solid rgba(147, 147, 147, 0.336);
        .expanded > & {
            height: auto;
        }
        &-title {
            width: 100%;
            height: auto;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 5px;
            position: sticky;
            top: 0;
            background: rgb(74, 72, 77);
            z-index: 2;

            * {
                user-select: none;
            }

            .expanded {
                visibility: visible;
                transform: rotate(180deg);
            }

            &:hover {
                opacity: 0.7;
                cursor: default;
            }

            &:active {
                opacity: 0.5;
                cursor: default;
            }
        }
    }
</style>
