use anyhow::anyhow;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;
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

pub async fn create_github_client() -> anyhow::Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static("reqwest_try"));

    let client = Client::builder().default_headers(headers).build()?;

    Ok(client)
}

pub async fn query_latest_release(
    client: &Client,
    owner: &str,
    repo: &str,
) -> anyhow::Result<Release> {
    let request_url = Url::parse(&format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/latest",
        owner = owner,
        repo = repo
    ))?;

    let response = client
        .get(request_url.as_str())
        .header(header::ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await?;

    let latest_release: Release = response.json().await?;

    Ok(latest_release)
}

impl Asset {
    pub async fn download<P: AsRef<Path>>(
        &self,
        client: &Client,
        dst: P,
    ) -> anyhow::Result<PathBuf> {
        fs::create_dir_all(&dst)?;

        let response = client.get(&self.browser_download_url).send().await?;
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .filter(|&x| x == HeaderValue::from_static("application/octet-stream"))
            .ok_or_else(|| anyhow!("content type is not application/octet-stream"))?;

        let content = response.bytes().await?;

        let filepath = self.download_filename()?;
        let filename = dst.as_ref().join(filepath);
        let mut dest = File::create(&filename)?;
        copy(&mut content.as_ref(), &mut dest)?;

        Ok(filename)
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
