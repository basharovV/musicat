import { writeFile } from "@tauri-apps/plugin-fs";
import WBK from "wikibase-sdk";
import { getImageFormat } from "../../utils/FileUtils";
import { db } from "../../data/db";
import type { Album, Song } from "../../App";
import { convertFileSrc } from "@tauri-apps/api/core";
import md5 from "md5";
import { addOriginCountryStatus, userSettings } from "../../data/store";
import { path } from "@tauri-apps/api";
import { get } from "svelte/store";
import { fetch } from "@tauri-apps/plugin-http";

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
        const albumPath = song.path.replace(`/${song.file}`, "");
        album = await db.albums.get(md5(`${albumPath} - ${song.album}`.toLowerCase()));
    }
    
    let result;
    
    result = await fetchAlbumArtWithWikipedia(album)
    if (result.success) {
        return result
    }
    
    const settings = get(userSettings);
    
    if (settings.geniusApiKey) {
        result = await fetchAlbumArtWithGenius(album, settings.geniusApiKey)
        if (result.success) {
            return result
        }
    }
    
    if (settings.discogsApiKey) {
        result = await fetchAlbumArtWithDiscogs(album, settings.discogsApiKey)
        if (result.success) {
            return result
        }
    }
    
    result = await fetchAlbumArtWithMusicBrainz(album)
    if (result.success) {
        return result
    }
    
    return {
        error: "No artwork found!"
    };
}

async function fetchAlbumArtWithWikipedia(
    album: Album,
): Promise<{ success?: string; error?: string }> {
    try {
        const ecArtist = encodeURIComponent(album.artist);
        const dbpediaResult = await (
            await fetch(
                `https://dbpedia.org/sparql?default-graph-uri=http%3A%2F%2Fdbpedia.org&query=PREFIX+rdf%3A+%3Chttp%3A%2F%2Fwww.w3.org%2F1999%2F02%2F22-rdf-syntax-ns%23%3E%0D%0APREFIX+dbpedia2%3A+%3Chttp%3A%2F%2Fdbpedia.org%2Fproperty%2F%3E%0D%0APREFIX+owl%3A+%3Chttp%3A%2F%2Fdbpedia.org%2Fontology%2F%3E%0D%0APREFIX+rdfs%3A+%3Chttp%3A%2F%2Fwww.w3.org%2F2000%2F01%2Frdf-schema%23%3E%0D%0ASELECT+DISTINCT+%3Fname%2C+%3FcoverArtVar+WHERE+%7B%0D%0A%09%3Fsubject+dbpedia2%3Aname+%3Fname+.%0D%0A%09%3Fsubject+rdfs%3Alabel+%3Flabel+.%0D%0A%09%7B+%3Fsubject+dbpedia2%3Aartist+%3Fartist+%7D+UNION+%7B+%3Fsubject+owl%3Aartist+%3Fartist+%7D%0D%0A%09%7B+%3Fartist+rdfs%3Alabel+%22${ecArtist}%22%40en+%7D+UNION+%7B+%3Fartist+dbpedia2%3Aname+%22${ecArtist}%22%40en+%7D%0D%0A%09%3Fsubject+rdf%3Atype+%3Chttp%3A%2F%2Fdbpedia.org%2Fontology%2FAlbum%3E+.%0D%0A%09%3Fsubject+dbpedia2%3Acover+%3FcoverArtVar+.%0D%0A%7D%0D%0ALimit+30%0D%0A&format=application%2Fsparql-results%2Bjson&timeout=10000&signal_void=on&signal_unconnected=on`
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

        if (await fetchImage(album, imageUrl, imageExtension)) {
            return {
                success: "Artwork saved!"
            };
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

async function fetchAlbumArtWithGenius(
    album: Album,
    geniusApiKey: string
): Promise<{ success?: string; error?: string }> {
    try {
        const query = encodeURIComponent(`${album.displayTitle} ${album.artist}`);

        const result =
            await fetch(`https://api.genius.com/search?q=${query}`, {
                method: "GET",
                headers: {
                    "Accept": "application/json",
                    "Authorization": `Bearer ${geniusApiKey}`
                }
            });
        
        if (!result.ok) {
            throw new Error("Genius API: " + JSON.stringify(result));
        }
        
        const data = await result.json();
        console.log("genius: ", data);

        const artist = toComparableString(album.artist)
        const hit = data.response.hits.find(
            (h) =>
                h?.result.header_image_url &&
                !h.result.header_image_url.includes("default_cover_image") &&
                artist === toComparableString(h?.result?.artist_names)
        )

        if (hit) {
            const imageUrl = hit.result?.header_image_url;
            
            const imageExtension = imageUrl.split("?")[0].split(".")?.pop();

            if (await fetchImage(album, imageUrl, imageExtension)) {
                return {
                    success: "Artwork saved!"
                };
            }
        }

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

async function fetchAlbumArtWithDiscogs(
    album: Album,
    discogsApiKey: string
): Promise<{ success?: string; error?: string }> {
    try {
        const ecRelease = encodeURIComponent(album.displayTitle);
        
        const result = 
            await fetch(`https://api.discogs.com/database/search?release_title=${ecRelease}&per_page=5&page=1`, {
                method: "GET",
                headers: {
                    "Accept": "application/json",
                    "Authorization": `Discogs token=${discogsApiKey}`,
                    "User-Agent": "Musicat +https://github.com/basharovV/musicat",
                }
            });
        
        if (!result.ok) {
            throw new Error("Discogs API: " + JSON.stringify(result));
        }
        
        const data = await result.json();
        console.log("discogs: ", data);
        
        const artist = toComparableString(album.artist)
        const hit = data.results.find(
            (h) => toComparableString(h.title).startsWith(artist)
        )
        
        if (hit) {
            const imageUrl = hit.cover_image;
            const imageExtension = imageUrl.split("?")[0].split(".")?.pop();

            if (await fetchImage(album, imageUrl, imageExtension)) {
                return {
                    success: "Artwork saved!"
                };
            }
        }

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

async function fetchAlbumArtWithMusicBrainz(
    album: Album
): Promise<{ success?: string; error?: string }> {
    try {
        const ecRelease = encodeURIComponent(album.displayTitle);
        const ecArtist = encodeURIComponent(album.artist);
        
        const result = 
            await fetch(`https://musicbrainz.org/ws/2/release/?query=release:${ecRelease}%20AND%20artist:${ecArtist}`, {
                method: "GET",
                headers: {
                    "Accept": "application/json",
                }
            });
        if (!result.ok) {
            throw new Error("MusicBrainz API: " + JSON.stringify(result));
        }
        
        const data = await result.json();
        console.log("musicbrainz: ", data);
        
        const artist = toComparableString(album.artist)
        let imageUrl;
        for (const release of data.releases) {
            if (artist !== toComparableString(release["artist-credit"][0]?.name)) {
                continue;
            }
            
            const result = await (
                await fetch(`https://coverartarchive.org/release/${release.id}/front`, {
                    method: "GET",
                })
            );
            // console.log(result)
            
            if (result.ok) {
                imageUrl = result.url;
                
                break;
            }
        }

        if (imageUrl && await fetchImage(album, imageUrl, 'jpg')) {
            return {
                success: "Artwork saved!"
            };
        }
        
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

function toComparableString(str) {
    return str?.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
}

async function fetchImage(
    album: Album,
    imageUrl?: string,
    imageExtension?: string,
): Promise<boolean> {
    if (!imageUrl || !imageExtension) {
        return false;
    }

    // Fetch image
    const imageData = await fetch(imageUrl);
    let imageArray = await imageData.body.getReader().read();

    let imageBody = imageArray.value;

    console.log("imageData");

    if (imageBody) {
        const filePath = await path.join(album.path,  `cover.${imageExtension}`);
        console.log("filepath", filePath);
        // Write a binary file to the `$APPDATA/avatar.png` path
        await writeFile(filePath, imageBody);
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
        return true;
        // Double success! Artwork should now be visible in the library
    }

    return false
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
