<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";

    import { isInfoPopupOpen } from "../../data/store";
    import { clickOutside } from "../../utils/ClickOutside";
    import ReleaseNotes from "./ReleaseNotes.svelte";

    let version = getVersion();
    export let onClickOutside;
</script>

<container>
    <div class="popup" use:clickOutside={onClickOutside}>
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
                    >built by <a
                        href="https://vyacheslavbasharov.com"
                        target="_blank">Slav</a
                    ></small
                >
                <!-- <br /> -->
                {#await version then versionValue}<small
                        >version {versionValue}</small
                    >{/await}
            </div>
        </section>
        <section class="release-notes">
            <ReleaseNotes />
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
        min-height: 400px;
        max-height: 500px;
        border-radius: 5px;
        position: relative;
        /* padding: 2em 0em 0; */
        background: rgb(55, 55, 55);
        color: white;
        border: 1px solid rgb(114, 114, 114);
        display: flex;
        flex-direction: row;
        flex: 10em;
    }

    .info {
        min-height: 100%;
        min-width: 300px;
        max-width: 350px;
        display: flex;
        flex-direction: column;
        grid-template-rows: auto 1fr 1fr;

        .app-icon {
            position: absolute;
            opacity: 0.08;
            width: auto;
            height: 100%;
            pointer-events: none;
            img {
                width: auto;
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
        border-left: 1px solid rgb(66, 65, 65);
    }
</style>
