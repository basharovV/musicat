export async function getWikipediaUrlForArtist(artistName) {
    if (!artistName?.length) return null;

    const validStatements = ["P27", "P495"];

    try {
        const dbpediaResult = await (
            await fetch(
                `https://dbpedia.org/sparql?default-graph-uri=http%3A%2F%2Fdbpedia.org&query=SELECT+%3Fartist+%3Flink%0D%0AWHERE+%7B%0D%0A++%7B+%3Fartist+a+dbo%3ABand+%7D%0D%0A++UNION%0D%0A++%7B+%3Fartist+a+dbo%3AMusicalArtist+%7D%0D%0A++%3Fartist+foaf%3Aname+%22${encodeURIComponent(artistName)}%22%40en+.%0D%0A++%3Fartist+foaf%3AisPrimaryTopicOf+%3Flink+.%0D%0A%7D%0D%0ALIMIT+1%0D%0A&format=application%2Fsparql-results%2Bjson&timeout=10000&signal_void=on&signal_unconnected=on`
            )
        ).json();
        console.log("dbpedia", dbpediaResult);
        let siteLink = false;
        for (let item of dbpediaResult?.results?.bindings) {
            if (item?.link?.value) {
                siteLink = item.link.value;
            }
        }

        if (!siteLink) return null;
        return `${siteLink}?useskin=vector-2022`;
    } catch (error) {
        console.error("Error fetching data from Wikipedia API:", error);
        return null;
    }
}
