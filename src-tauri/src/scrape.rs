use std::error::Error;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use kuchiki::parse_html;
use kuchiki::traits::*;
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetHTMLRequest {
    url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetHTMLResponse {
    html: Option<String>,
}

fn edit_anchors(html: String, prefix: String) -> String {
    let document = parse_html().one(html);
    let prefix_url = Url::parse(prefix.as_str()).unwrap();

    for css_match in document.select("a").unwrap() {
        let as_node = css_match.as_node();
        if let Some(element) = as_node.as_element() {
            let mut attrs = element.attributes.borrow_mut();

            // add target="_blank"
            attrs.insert("target".to_string(), "_blank".to_string());

            // add "https://en.wikipedia.org" to the links
            if let Some(href) = attrs.get("href") {
                let mut new_url = prefix_url.join(href).unwrap();

                if prefix_url.host_str() != new_url.host_str() {
                    new_url.set_scheme(prefix_url.scheme()).unwrap();
                    new_url.set_host(prefix_url.host_str()).unwrap();

                    if let Some(port) = prefix_url.port() {
                        new_url.set_port(Some(port)).unwrap();
                    }
                }

                attrs.insert("href".to_string(), new_url.to_string());
            }
        }
    }

    let mut bytes = vec![];
    document.serialize(&mut bytes).unwrap();

    String::from_utf8(bytes).unwrap()
}

#[tauri::command]
pub async fn get_wikipedia(event: GetHTMLRequest) -> GetHTMLResponse {
    let html = fetch_wikipedia(event.url.as_str()).await;
    return GetHTMLResponse {
        html: html.map_or(None, |l| Some(edit_anchors(l, event.url))),
    };
}

async fn fetch_wikipedia(url: &str) -> Result<String, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            // Pick a modern browser UA string — don't use the default reqwest UA
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
         AppleWebKit/537.36 (KHTML, like Gecko) \
         Chrome/117.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.google.com/"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client.get(url).send().await?;
    // Check if the request was successful (status code 200)
    if !response.status().is_success() {
        return Err("Wiki page not found".into());
    }

    // Get the HTML content from the response
    let body = response.text().await?;
    info!("{:?}", body);

    // Parse the HTML content using the scraper crate
    let document = Html::parse_document(&body);

    // Use a CSS selector to find the lyrics
    let lyrics_selector = Selector::parse("#mw-content-text").unwrap();

    let lyrics_element = document.select(&lyrics_selector).next();
    info!("{:?}", lyrics_element);

    // Extract and return the lyrics
    match lyrics_element {
        Some(element) => Ok(element.html()),
        None => Err("Lyrics not found".into()),
    }
}
