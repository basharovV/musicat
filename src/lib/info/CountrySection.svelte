<script lang="ts">
    import tippy from "svelte-tippy";
    import list from "../../data/countries.json";
    import { db } from "../../data/db";
    import { rightClickedTracks, userSettings } from "../../data/store";
    import LL from "../../i18n/i18n-svelte";
    import { findCountryByArtist } from "../data/LibraryEnrichers";
    import ButtonWithIcon from "../ui/ButtonWithIcon.svelte";
    import Icon from "../ui/Icon.svelte";
    import InputDropdown from "../ui/InputDropdown.svelte";
    import { countries, getFlagEmoji } from "../data/CountryCodes";

    let isFetchingOriginCountry = false;
    let originCountry = $rightClickedTracks[0]?.originCountry || null;
    let originCountryEdited = { label: originCountry, value: originCountry };

    async function fetchFromWikipedia() {
        if (isFetchingOriginCountry) {
            return;
        }

        isFetchingOriginCountry = true;
        originCountryEdited = null;

        const country = await findCountryByArtist(
            $rightClickedTracks[0].artist,
        );

        console.log("country", country);
        if (country) {
            originCountryEdited = country;
        }
        isFetchingOriginCountry = false;
    }

    async function saveTrack() {
        $rightClickedTracks[0].originCountry = originCountryEdited.value;

        // Find all songs with this artist
        const artistSongs = await db.songs
            .where("artist")
            .equals($rightClickedTracks[0].artist)
            .toArray();

        console.log("artistSongs", artistSongs);
        artistSongs.forEach((s) => {
            db.songs.update(s.id, { originCountry: originCountryEdited.value });
        });

        originCountry = originCountryEdited.value;
    }
</script>

<section class="enrichment-section boxed">
    <div class="label">
        <h4>{$LL.trackInfo.enrichment.country.title()}</h4>
        <div
            use:tippy={{
                content: $LL.trackInfo.enrichment.country.infoTooltip(),
                placement: "right",
            }}
        >
            <Icon icon="mdi:information" />
        </div>
    </div>
    {#if $userSettings.beetsDbLocation}
        <small>⚠️ {$LL.trackInfo.enrichment.country.disabled()}</small>
    {/if}
    <div class="country">
        <InputDropdown
            options={Object.entries(countries).map((c) => ({
                value: c[1],
                label: `${getFlagEmoji(c[0])} ${c[1]}`,
            }))}
            bind:selected={originCountryEdited}
            onSelect={(s) => {
                console.log("on select", s);
                originCountryEdited = {
                    value: s,
                    label: s,
                };
            }}
            placeholder={$LL.common.noResults()}
        />
        <ButtonWithIcon
            onClick={saveTrack}
            text={$LL.trackInfo.enrichment.country.saveButton.title()}
            icon="material-symbols:save-outline"
            theme="translucent"
            disabled={originCountry === originCountryEdited.value ||
                !!$userSettings.beetsDbLocation}
        />
        <ButtonWithIcon
            onClick={fetchFromWikipedia}
            isLoading={isFetchingOriginCountry}
            text={$LL.trackInfo.enrichment.country.fetchButton.title()}
            icon={isFetchingOriginCountry
                ? "line-md:loading-loop"
                : "tabler:world-download"}
            theme="transparent"
            tooltip={{
                content: $LL.trackInfo.enrichment.country.fetchButton.tooltip(),
            }}
            disabled={!!$userSettings.beetsDbLocation}
        />
    </div>
</section>

<style lang="scss">
    .enrichment-section {
        border-radius: 5px;
        padding: 1em;
        grid-column: 1 / 3;
        position: relative;
        text-align: left;

        small {
            display: block;
            margin: 0.5em 0;
        }

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
