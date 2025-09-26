<script lang="ts">
    import tippy from "svelte-tippy";
    import list from "../../data/countries.json";
    import { db } from "../../data/db";
    import { rightClickedTrack, rightClickedTracks } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import InputDropdown from "../ui/InputDropdown.svelte";

    let isFetchingOriginCountry = false;
    let originCountry =
        ($rightClickedTrack || $rightClickedTracks[0])?.originCountry || null;
    let originCountryEdited = { label: originCountry, value: originCountry };

    async function fetchFromWikipedia() {
        if (isFetchingOriginCountry) {
            return;
        }

        isFetchingOriginCountry = true;
        originCountryEdited = null;

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
            originCountryEdited.value;

        // Find all songs with this artist
        const artistSongs = await db.songs
            .where("artist")
            .equals(($rightClickedTrack || $rightClickedTracks[0]).artist)
            .toArray();

        artistSongs.forEach((s) => {
            db.songs.update(s.id, { originCountry: originCountryEdited.value });
        });

        originCountry = originCountryEdited.value;
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
        <InputDropdown
            options={list.map((c) => ({ value: c, label: c }))}
            bind:selected={originCountryEdited}
        />
        <ButtonWithIcon
            onClick={saveTrack}
            text={$LL.trackInfo.save()}
            icon="material-symbols:save-outline"
            theme="translucent"
            disabled={originCountry === originCountryEdited.value}
        />
        <ButtonWithIcon
            onClick={fetchFromWikipedia}
            isLoading={isFetchingOriginCountry}
            text={$LL.trackInfo.fetchFromWikipedia()}
            icon={isFetchingOriginCountry
                ? "line-md:loading-loop"
                : "tabler:world-download"}
            theme="transparent"
        />
    </div>
</section>

<style lang="scss">
    .enrichment-section {
        margin-top: 1.5em;
        border-radius: 5px;
        padding: 2em 1em 1em 1em;
        grid-column: 1 / 3;
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
            justify-content: flex-start;
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
