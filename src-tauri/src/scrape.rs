use std::error::Error;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use kuchiki::traits::*;
use kuchiki::parse_html;
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
    // Make an HTTP GET request to the Wikipedia URL
    let response = reqwest::get(url).await?;

    // Check if the request was successful (status code 200)
    if !response.status().is_success() {
        return Err("Wiki page not found".into());
    }

    // Get the HTML content from the response
    let body = response.text().await?;
    // info!("{:?}", body);

    // Parse the HTML content using the scraper crate
    let document = Html::parse_document(&body);

    // Use a CSS selector to find the lyrics
    let lyrics_selector = Selector::parse("#mw-content-text").unwrap();

    let lyrics_element = document.select(&lyrics_selector).next();
    // info!("{:?}", lyrics_element);

    // Extract and return the lyrics
    match lyrics_element {
        Some(element) => Ok(element.html()),
        None => Err("Lyrics not found".into()),
    }
}
