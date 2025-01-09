<script lang="ts">
    import type { Album, MapTooltipData } from "../../App";
    import { db } from "../../data/db";
    import md5 from "md5";

    export let data: MapTooltipData = null;
    let albums: Album[] = null;

    async function getAlbums() {
        console.log("albums to get", data);
        albums = await db.albums.bulkGet(
            data?.albums.map((a) =>
                md5(`${a.artist} - ${a.album}`.toLowerCase()),
            ),
        );
        albums = albums.filter((a) => a?.artwork).slice(0, 5);
    }
    $: {
        if (data) {
            getAlbums();
        }
    }
</script>

{#if data}
    <div class="tooltip">
        <p class="title">{data.emoji} {data.countryName}</p>
        {#if data.numberOfArtists}
            <p class="description">
                {data.numberOfArtists} artist{data.numberOfArtists === 1
                    ? ""
                    : "s"} including
                <span class="artists">
                    {data.artists.join(", ")}
                </span>
            </p>
            {#if albums?.length}
                <div class="artworks">
                    {#each albums as album (album.id)}
                        {#if album.artwork}
                            <img src={album.artwork.src} alt="album" />
                        {/if}
                    {/each}
                </div>
            {/if}
        {:else}
            <p class="description">No songs tagged</p>
        {/if}
    </div>
{/if}

<style lang="scss">
    .tooltip {
        position: relative;
        background-color: var(--mapview-tooltip-bg);
        border: 0.5px solid var(--mapview-tooltip-border);
        border-radius: 9px;
        backdrop-filter: blur(8px);
        padding: 0.5em 1em;
        text-overflow: ellipsis;
        display: flex;
        align-items: flex-start;
        justify-content: flex-start;
        flex-direction: column;
        color: var(--mapview-tooltip-text);

        p {
            display: inline-block;
            margin: 0;
            text-align: left;
            width: fit-content;
        }

        .title {
            font-weight: bold;
            width: max-content;
        }

        .description {
            font-size: smaller;
            line-height: 1.3em;
            color: var(--mapview-tooltip-text);
            max-width: 200px;

            .artists {
                font-style: italic;
                color: var(--mapview-tooltip-artist);
            }
        }

        .artworks {
            display: flex;
            flex-direction: row;
            gap: 5px;
            margin-top: 10px;
            margin-bottom: 5px;
            img {
                width: 30px;
                border-radius: 1px;
            }
        }
    }
</style>
