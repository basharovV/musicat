<script lang="ts">
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import { db } from "../../data/db";
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import Icon from "../ui/Icon.svelte";
    import LL from "../../i18n/i18n-svelte";
    import { rightClickedTrack, rightClickedTracks } from "../../data/store";
    import Svelecte from "svelecte";
    import tippy from "svelte-tippy";
    import list from "../../data/countries.json";

    let isFetchingOriginCountry = false;
    let originCountry =
        ($rightClickedTrack || $rightClickedTracks[0])?.originCountry || null;
    let originCountryEdited = originCountry;

    async function fetchFromWikipedia() {
        originCountryEdited = null;
        isFetchingOriginCountry = true;
        const country = await findCountryByArtist(
            ($rightClickedTrack || $rightClickedTracks[0]).artist,
        );
        console.log("country", country);
        if (country) {
            originCountryEdited = country;
        }
        isFetchingOriginCountry = false;
    }

    async function saveTrack() {
        ($rightClickedTrack || $rightClickedTracks[0]).originCountry =
            originCountryEdited;

        // Find all songs with this artist
        const artistSongs = await db.songs
            .where("artist")
            .equals(($rightClickedTrack || $rightClickedTracks[0]).artist)
            .toArray();

        artistSongs.forEach((s) => {
            db.songs.update(s.id, { originCountry: originCountryEdited });
        });

        originCountry = originCountryEdited;
    }
</script>

<section class="enrichment-section boxed">
    <h5 class="section-title">
        <Icon icon="iconoir:atom" size={34} />{$LL.trackInfo.enrichmentCenter()}
    </h5>
    <div class="label">
        <h4>{$LL.trackInfo.countryOfOrigin()}</h4>
        <div
            use:tippy={{
                content: $LL.trackInfo.countryOfOriginTooltip(),
                placement: "right",
            }}
        >
            <Icon icon="mdi:information" />
        </div>
    </div>
    <div class="country">
        <Svelecte
            options={list}
            max={1}
            clearable={true}
            bind:value={originCountryEdited}
            placeholder={isFetchingOriginCountry
                ? $LL.trackInfo.fetchingOriginCountry()
                : ""}
        />
        <ButtonWithIcon
            onClick={saveTrack}
            text={$LL.trackInfo.save()}
            icon="material-symbols:save-outline"
            theme="translucent"
            disabled={originCountry === originCountryEdited}
        />
        <ButtonWithIcon
            onClick={fetchFromWikipedia}
            isLoading={isFetchingOriginCountry}
            text={$LL.trackInfo.fetchFromWikipedia()}
            icon="tabler:world-download"
            theme="transparent"
        />
    </div>
</section>

<style lang="scss">
    .enrichment-section {
        margin-top: 1.5em;
        border: 1px solid
            color-mix(in srgb, var(--background) 70%, var(--inverse));
        border-radius: 5px;
        padding: 2em 1em 1em 1em;
        grid-column: 1 / 3;
        background-color: color-mix(
            in srgb,
            var(--overlay-bg) 80%,
            var(--inverse)
        );
        position: relative;

        .label {
            display: flex;
            flex-direction: row;
            gap: 5px;
            align-items: center;
            margin: 0 0 5px 0;

            h4 {
                margin: 0;
                color: var(--text);
                text-align: left;
                text-transform: uppercase;
                letter-spacing: 0.5px;
                font-size: 0.9em;
            }

            p {
                margin: 0;
                color: rgb(from var(--text) r g b / 0.768);
            }
        }
        .country {
            display: flex;
            justify-content: space-between;
            align-items: center;
            width: fit-content;
            gap: 5px;
            width: 100%;

            p {
                margin-right: 1em;
            }
        }
    }
</style>
