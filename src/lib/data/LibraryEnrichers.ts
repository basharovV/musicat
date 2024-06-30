import { writeBinaryFile } from "@tauri-apps/api/fs";
import WBK from "wikibase-sdk";
import { getImageFormat } from "../../utils/FileUtils";
import { db } from "../../data/db";
import type { Album, Song } from "../../App";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import md5 from "md5";
import { addOriginCountryStatus } from "../../data/store";

export async function findCountryByArtist(artistName) {
    if (!artistName?.length) return null;

    const validStatements = ["P27", "P495"];

    try {
        const wdk = WBK({
            instance: "https://www.wikidata.org",
            sparqlEndpoint: "https://query.wikidata.org/sparql"
        });
        let url = wdk.searchEntities({ search: artistName });

        const response = await fetch(url);
        const json = await response.json();
        const firstItem = json.search && json.search[0];

        if (!firstItem) return null;

        url = wdk.getEntities({
            ids: [firstItem.id]
        });
        const { entities } = await fetch(url).then((res) => res.json());
        console.log(entities);

        let foundCountry = null;
        for (let i = 0; i < validStatements.length; i++) {
            const countryEntity =
                Object.values(entities)[0]["claims"][validStatements[i]];
            if (!countryEntity) continue;
            const countryId = countryEntity[0]["mainsnak"].datavalue.value.id;
            url = wdk.getEntities({ ids: [countryId] });
            const country = await fetch(url).then((res) => res.json());
            const result = Object.values(country.entities)[0]["labels"]["en"]
                ?.value;
            if (result) {
                foundCountry = result;
                break;
            }
        }

        return foundCountry;
    } catch (error) {
        console.error("Error fetching data from Wikipedia API:", error);
        return null;
    }
}

export async function fetchAlbumArt(
    album: Album = null,
    song: Song = null
): Promise<{ success?: string; error?: string }> {
    if (!album) {
        album = await db.albums.get(md5(`${song.artist} - ${song.album}`.toLowerCase()));
    }
    try {
        const dbpediaResult = await (
            await fetch(
                `https://dbpedia.org/sparql?default-graph-uri=http%3A%2F%2Fdbpedia.org&query=PREFIX+rdf%3A+%3Chttp%3A%2F%2Fwww.w3.org%2F1999%2F02%2F22-rdf-syntax-ns%23%3E%0D%0APREFIX+dbpedia2%3A+%3Chttp%3A%2F%2Fdbpedia.org%2Fproperty%2F%3E%0D%0APREFIX+owl%3A+%3Chttp%3A%2F%2Fdbpedia.org%2Fontology%2F%3E%0D%0APREFIX+rdfs%3A+%3Chttp%3A%2F%2Fwww.w3.org%2F2000%2F01%2Frdf-schema%23%3E%0D%0ASELECT+DISTINCT+%3Fname%2C+%3FcoverArtVar+WHERE+%7B%0D%0A%09%3Fsubject+dbpedia2%3Aname+%3Fname+.%0D%0A%09%3Fsubject+rdfs%3Alabel+%3Flabel+.%0D%0A%09%7B+%3Fsubject+dbpedia2%3Aartist+%3Fartist+%7D+UNION+%7B+%3Fsubject+owl%3Aartist+%3Fartist+%7D%0D%0A%09%7B+%3Fartist+rdfs%3Alabel+%22${album.artist}%22%40en+%7D+UNION+%7B+%3Fartist+dbpedia2%3Aname+%22${album.artist}%22%40en+%7D%0D%0A%09%3Fsubject+rdf%3Atype+%3Chttp%3A%2F%2Fdbpedia.org%2Fontology%2FAlbum%3E+.%0D%0A%09%3Fsubject+dbpedia2%3Acover+%3FcoverArtVar+.%0D%0A%7D%0D%0ALimit+30%0D%0A&format=application%2Fsparql-results%2Bjson&timeout=10000&signal_void=on&signal_unconnected=on`
            )
        ).json();
        console.log("dbpedia", dbpediaResult);
        let coverArtFile;
        // Check if album is in here
        for (let item of dbpediaResult?.results?.bindings) {
            if (item.name.value.toLowerCase() === album.title.toLowerCase()) {
                console.log("found it");
                if (item.coverArtVar?.value) {
                    coverArtFile = item.coverArtVar.value;
                    break;
                }
            }
        }

        // Then grab it from Wikipedia
        const wikiResult = await (
            await fetch(
                `https://en.wikipedia.org/w/api.php?format=json&&origin=*&action=query&prop=imageinfo&iiprop=url|size&titles=File:${coverArtFile}`,
                {
                    headers: {
                        "Accept": "application/json"
                    }
                }
            )
        ).json();
        let imageUrl;
        let imageExtension;
        const pages = wikiResult?.query?.pages;
        if (Object.values(pages)?.length === 1) {
            const imageInfo = Object.values(pages)[0]?.imageinfo;
            if (imageInfo?.length) {
                imageUrl = imageInfo[0]?.url;
                imageExtension = imageUrl.split(".")?.pop();
                console.log("url", imageUrl);
            }
        }

        if (imageUrl && imageExtension) {
            // Fetch image
            const imageData = await fetch(imageUrl);
            let imageArray = await imageData.body.getReader().read();

            let imageBody = imageArray.value;

            console.log("imageData");

            if (imageBody) {
                const filePath = album.path + "/cover." + imageExtension;
                console.log("filepath", filePath);
                // Write a binary file to the `$APPDATA/avatar.png` path
                await writeBinaryFile(filePath, imageBody);
                let format = getImageFormat(imageExtension);
                // Success! Let's write the artwork to the album

                // TODO: Option to write to track directly?

                await db.albums.update(album, {
                    artwork: {
                        src: convertFileSrc(filePath) + `?${Date.now()}`,
                        format: format,
                        size: {
                            width: 200,
                            height: 200
                        }
                    }
                });
                return {
                    success: "Artwork saved!"
                };
                // Double success! Artwork should now be visible in the library
            }
        }

        console.log("got art", wikiResult);

        return {
            error: "No artwork found!"
        };
    } catch (err) {
        console.error("Error fetching artwork", err);

        return {
            error: "Error fetching artwork." + err
        };
    }
}

export async function addCountryDataAllSongs() {
    const allArtists = await db.songs.orderBy("artist").uniqueKeys();
    let count = 1;
    let delay = 0;
    const delayIncrement = 1000;

    await allArtists.map((artist, idx) => {
        delay += delayIncrement;
        return new Promise((resolve) => setTimeout(resolve, delay)).then(
            async () => {
                await enrichArtistCountry(artist);
                if (idx === allArtists.length - 1) {
                    addOriginCountryStatus.set(null);
                } else {
                    addOriginCountryStatus.set({
                        percent: Math.ceil(
                            ((idx + 1) / allArtists.length) * 100
                        )
                    });
                }
            }
        );
    });
}

async function enrichArtistCountry(artist) {
    const country = await findCountryByArtist(artist);
    console.log("country", country);
    if (country) {
        // Find all songs with this artist
        const artistSongs = await db.songs
            .where("artist")
            .equals(artist)
            .toArray();

        db.songs.bulkPut(
            artistSongs.map((s) => {
                s.originCountry = country;
                return s;
            })
        );
    }
}
