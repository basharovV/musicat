import WBK from "wikibase-sdk";

export async function findCountryByArtist(artistName) {
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
