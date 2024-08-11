use std::error::Error;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetHTMLRequest {
    url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetHTMLResponse {
    html: Option<String>,
}

#[tauri::command]
pub async fn get_wikipedia(event: GetHTMLRequest) -> GetHTMLResponse {
    let html = fetch_wikipedia(event.url.as_str()).await;
    return GetHTMLResponse {
        html: html.map_or(None, |l| Some(l)),
    };
}

async fn fetch_wikipedia(url: &str) -> Result<String, Box<dyn Error>> {
    // Make an HTTP GET request to the Wikipedia URL
    let response = reqwest::get(url).await?;

    // Check if the request was successful (status code 200)
    if !response.status().is_success() {
        return Err("Wiki page not found".into());
    }

    // Get the HTML content from the response
    let body = response.text().await?;
    // println!("{:?}", body);

    // Parse the HTML content using the scraper crate
    let document = Html::parse_document(&body);

    // Use a CSS selector to find the lyrics
    let lyrics_selector = Selector::parse("#mw-content-text").unwrap();

    let lyrics_element = document.select(&lyrics_selector).next();
    // println!("{:?}", lyrics_element);

    // Extract and return the lyrics
    match lyrics_element {
        Some(element) => Ok(element.html()),
        None => Err("Lyrics not found".into()),
    }
}
