import WBK from "wikibase-sdk";

export async function getWikipediaUrlForArtist(artistName) {
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
        console.log("Wiki response", json);
        const firstItem =
            json.search && json.search.find((s) => s.match.type === "label");

        if (!firstItem) return null;

        url = wdk.getEntities({
            ids: [firstItem.id]
        });
        const { entities } = await fetch(url).then((res) => res.json());
        console.log(entities);
        const firstEntity = Object.values(entities)[0];
        if (!firstEntity) return null;
        const siteLink = firstEntity?.["sitelinks"]?.enwiki?.title;

        if (!siteLink) return null;
        return `https://en.wikipedia.org/wiki/${siteLink}?useskin=vector-2022`;
    } catch (error) {
        console.error("Error fetching data from Wikipedia API:", error);
        return null;
    }
}
