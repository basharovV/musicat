export async function getWikipediaUrlForArtist(artistName) {
    if (!artistName?.length) return null;

    // 'hastemplate' is a powerful hidden search keyword in MediaWiki
    // It filters results to ONLY those containing the musical artist infobox.
    const searchQuery = `"${artistName}" hastemplate:"Infobox musical artist"`;

    const params = new URLSearchParams({
        action: "query",
        generator: "search",
        gsrsearch: searchQuery,
        gsrlimit: "5",
        prop: "info",
        inprop: "url",
        format: "json",
        origin: "*",
    });

    try {
        const response = await fetch(
            `https://en.wikipedia.org/w/api.php?${params.toString()}`,
        );
        const data = await response.json();

        if (!data.query?.pages) {
            // FALLBACK: If "hastemplate" was too strict, try a broader search
            // but look for "(band)" in the title.
            return await fallbackSearch(artistName);
        }

        const pages = Object.values(data.query.pages).sort(
            (a, b) => a.index - b.index,
        );

        // Return the first match that looks like a high-quality link
        return `${pages[0].fullurl}?useskin=vector-2022`;
    } catch (error) {
        console.error("Error fetching Wikipedia artist:", error);
        return null;
    }
}

async function fallbackSearch(artistName) {
    // Second attempt: Search for "Artist Name (band)" specifically
    const params = new URLSearchParams({
        action: "query",
        generator: "search",
        gsrsearch: `${artistName} (band)`,
        gsrlimit: "1",
        prop: "info",
        inprop: "url",
        format: "json",
        origin: "*",
    });

    const response = await fetch(
        `https://en.wikipedia.org/w/api.php?${params.toString()}`,
    );
    const data = await response.json();
    const pages = data.query?.pages;

    if (pages) {
        const page = Object.values(pages)[0];
        return `${page.fullurl}?useskin=vector-2022`;
    }

    return null;
}
