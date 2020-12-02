use anyhow::anyhow;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

macro_rules! pub_fields {
    (
        $(#[$deri:meta])*
             struct $name:ident {
                 $($field:ident: $t:ty,)*
             }
    ) => {
        $(#[$deri])*
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_fields! {
    #[derive(Debug, Deserialize)]
    struct Release {
        tag_name: String,
        name: String,
        assets: Vec<Asset>,
        body: String,
        author: Author,
    }
}

pub_fields! {
    #[derive(Debug, Deserialize)]
    struct Asset {
        url: String,
        name: String,
        content_type: String,
        state: String,
        size: usize,
        created_at: String,
        updated_at: String,
        browser_download_url: String,
    }
}

pub_fields! {
    #[derive(Debug, Deserialize)]
    struct Author {
        login: String,
    }
}

pub async fn query_latest_release(owner: &str, repo: &str) -> anyhow::Result<Release> {
    let request_url = Url::parse(&format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/latest",
        owner = owner,
        repo = repo
    ))?;

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static("reqwest_try"));

    let client = Client::builder().default_headers(headers).build()?;

    let response = client
        .get(request_url.as_str())
        .header(header::ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await?;

    let latest_release: Release = response.json().await?;

    Ok(latest_release)
}

impl Asset {
    pub fn download_filename(&self) -> anyhow::Result<PathBuf> {
        let browser_download_url = &self.browser_download_url;

        let url = Url::parse(browser_download_url)?;
        let filename = url
            .path_segments()
            .ok_or_else(|| anyhow!("browser_download_url no path segments"))?
            .last()
            .unwrap();

        Ok(PathBuf::from(filename))
    }
}
