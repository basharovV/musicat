import type { ArtistLinkItem } from "src/App";

async function getYouTubeTitle(videoId) {
    let title;
    await fetch(
        `https://www.youtube.com/oembed?url=https://www.youtube.com/watch?v=${videoId}&feature=emb_logo&format=json`
    )
        .then((response) => response.json())
        .then((response) => {
            console.log("response", response);
            if (response) {
                title = response.title;
            }
        });
    return title;
}
/**
 * Adds thumbnail URL, proper title from OpenGraph to link item and returns it
 * Services supported: YouTube
 */
export async function getLinkItemWithData(url): Promise<ArtistLinkItem> {
    // Populate image / title
    let imageUrl;
    let title = url; // by default it's the URL itself

    if (url.includes("youtube")) {
        const videoId = url.split("v=")[1].split("&")[0];
        if (videoId) {
            imageUrl = `https://img.youtube.com/vi/${videoId}/sddefault.jpg`;
            title = await getYouTubeTitle(videoId);
            if (!title) title = url;
        }
    }

    return {
        url,
        name: title,
        imageUrl,
        tags: [],
        type: "link"
    };
}
