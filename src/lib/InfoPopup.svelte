<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";

    import { isInfoPopupOpen } from "../data/store";
    import ReleaseNotes from "./ReleaseNotes.svelte";

    let version = getVersion();

    function onPageClick() {
        $isInfoPopupOpen = false;
    }
</script>

<container on:click={onPageClick}>
    <div class="popup">
        <!-- <img src="images/cd6.gif" /> -->
        <section class="info">
            <h1>Musicat</h1>
            <p>🎹 for the love of music 🥁</p>
            <small>built by <a href="https://vyacheslavbasharov.com">Slav</a></small>
            <!-- <br /> -->
            {#await version then versionValue}<small
                    >version {versionValue}</small
                >{/await}
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
        min-width: 300px;
        display: flex;
        flex-direction: column;
        justify-content: flex-start;

        h1 {
            font-family: "2Peas";
            margin-bottom: 0;
        }

        small {
            opacity: 0.5;
        }
    }

    .release-notes {
        border-left: 1px solid rgb(66, 65, 65);
    }
</style>
