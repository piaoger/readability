use crate::dom;
use crate::error::Error;
use crate::scorer;
use crate::scorer::Candidate;

use html5ever::tendril::stream::TendrilSink;
use html5ever::{parse_document, serialize};
use markup5ever_rcdom::RcDom;
use markup5ever_rcdom::SerializableHandle;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::default::Default;
use std::io::Read;
use std::path::Path;

use url::Url;

#[cfg(feature = "reqwest")]
use std::time::Duration;

#[cfg(feature = "reqwest")]
use reqwest;

#[derive(Debug)]
pub struct Product {
    pub title: String,
    pub content: String,
    pub text: String,
}

#[cfg(feature = "reqwest")]
pub fn scrape(url: &str) -> Result<Product, Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::new(30, 0))
        .build()?;
    let mut res = client.get(url).send()?;
    if res.status().is_success() {
        let url = Url::parse(url)?;
        extract(&mut res, &url)
    } else {
        Err(Error::Unexpected)
    }
}

/*

// let's support asnc later

#[cfg(feature = "reqwest")]
pub async fn scrape(url: &str) -> Result<Product, Error> {
    let client = Client::builder()
        .timeout(Duration::new(30, 0))
        .build()?;
    scrape_with_client(url, &client).await
}

#[cfg(feature = "reqwest")]
pub async fn scrape_with_client(url: &str, client: &Client) -> Result<Product, Error> {
    let res = client.get(url)
        .send()
        .await?;
    if res.status().is_success() {
        let url = Url::parse(url)?;
        extract(&mut res.text().await?.as_bytes(), &url)
    } else {
        Err(Error::Unexpected)
    }
}
*/

pub fn extract<R>(input: &mut R, url: &Url) -> Result<Product, Error>
where
    R: Read,
{
    let mut dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(input)
        .unwrap();
    let mut title = String::new();
    let mut candidates = BTreeMap::new();
    let mut nodes = BTreeMap::new();
    let handle = dom.document.clone();
    scorer::preprocess(&mut dom, handle.clone(), &mut title);
    scorer::find_candidates(
        &mut dom,
        Path::new("/"),
        handle.clone(),
        &mut candidates,
        &mut nodes,
    );
    let mut id: &str = "/";
    let mut top_candidate: &Candidate = &Candidate {
        node: handle.clone(),
        score: Cell::new(0.0),
    };
    for (i, c) in candidates.iter() {
        let score = c.score.get() * (1.0 - scorer::get_link_density(c.node.clone()));
        c.score.set(score);
        if score <= top_candidate.score.get() {
            continue;
        }
        id = i;
        top_candidate = c;
    }
    let mut bytes = vec![];

    let node = top_candidate.node.clone();
    scorer::clean(&mut dom, Path::new(id), node.clone(), url, &candidates);

    let serializable: SerializableHandle = node.clone().into();

    serialize(&mut bytes, &serializable, Default::default()).ok();

    // serialize(&mut bytes, &node, Default::default()).ok();
    let content = String::from_utf8(bytes).unwrap_or_default();

    let mut text: String = String::new();
    dom::extract_text(node, &mut text, true);
    Ok(Product {
        title,
        content,
        text,
    })
}
