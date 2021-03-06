use std::env;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use bytes::Buf;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use url::Url;

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

pub async fn create_github_client() -> anyhow::Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static("reqwest_try"));

    let client = Client::builder().default_headers(headers).build()?;

    Ok(client)
}

pub async fn query_latest_release(client: &Client, repo: &str) -> anyhow::Result<Release> {
    let request_url = Url::parse(&format!(
        "https://api.github.com/repos/{repo}/releases/latest",
        repo = repo
    ))
    .with_context(|| "Fail to parse url")?;

    lazy_static! {
        static ref USERNAME: String = env::var("GITHUB_USERNAME").unwrap();
        static ref TOKEN: String = env::var("GITHUB_PERSONAL_ACCESS_TOKEN").unwrap();
    }

    let response = client
        .get(request_url.as_str())
        .basic_auth(&*USERNAME, Some(&*TOKEN))
        .header(header::ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await
        .with_context(|| "Fail to to send request")?;

    let latest_release: Release = response
        .json()
        .await
        .with_context(|| "Fail to deserialize json format of response into Release struct")?;

    Ok(latest_release)
}

impl Release {
    pub fn version(&self) -> anyhow::Result<Version> {
        lazy_static! {
            static ref VERSION_RE: Regex = Regex::new(r"(\d+).(\d+).(\d+)").unwrap();
        }
        let version = vec![&self.tag_name, &self.name]
            .into_iter()
            .filter_map(|name| VERSION_RE.find(&name))
            .map(|m| Version::parse(m.as_str()).unwrap())
            .next()
            .ok_or_else(|| anyhow!("Cannot find version from tag_name or name"))?;
        Ok(version)
    }
}

impl Asset {
    pub async fn download(&self, client: &Client) -> anyhow::Result<bytes::Bytes> {
        let response = client.get(&self.browser_download_url).send().await?;

        let content_type = response.headers().get(header::CONTENT_TYPE);
        if content_type != Some(&HeaderValue::from_static("application/octet-stream")) {
            return Err(anyhow!("content type is not application/octet-stream"));
        }

        Ok(response.bytes().await?.to_bytes())
    }

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
