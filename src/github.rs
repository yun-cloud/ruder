use std::fs;
use std::fs::File;
use std::io;
use std::io::copy;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use bytes::IntoBuf;
use flate2::read::GzDecoder;
use log::info;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::Deserialize;
use tar::Archive;
use url::Url;
use zip::ZipArchive;

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

        let filepath = self.download_filename()?;
        info!("downloading {:?}...", filepath);
        let filename = dst.as_ref().join(filepath);
        let mut dest = File::create(&filename)?;
        let content = response.bytes().await?;
        copy(&mut content.as_ref(), &mut dest)?;

        Ok(filename)
    }

    pub async fn download_to<W: Write>(
        &self,
        client: &Client,
        mut dst: &mut W,
        src: &str,
    ) -> anyhow::Result<u64> {
        let response = client.get(&self.browser_download_url).send().await?;

        if response.headers().get(header::CONTENT_TYPE)
            != Some(&HeaderValue::from_static("application/octet-stream"))
        {
            return Err(anyhow!("content type is not application/octet-stream"));
        }

        let content = response.bytes().await?;
        let mut buf = content.into_buf();

        let mut size = 0u64;
        let filename = {
            let name = self.download_filename()?;
            String::from(name.to_str().expect("pathbuf to &str failed"))
        };
        if filename.ends_with(".tar.gz") {
            let mut ar = Archive::new(GzDecoder::new(buf));
            for entry in ar.entries()? {
                let mut f = entry?;
                if f.path()? == Path::new(&src) {
                    size = io::copy(&mut f, &mut dst)?;
                }
            }
        } else if filename.ends_with(".tar") {
            let mut ar = Archive::new(buf);
            for entry in ar.entries()? {
                let mut f = entry?;
                if f.path()? == Path::new(&src) {
                    size = io::copy(&mut f, &mut dst)?;
                }
            }
        } else if filename.ends_with(".zip") {
            let mut ar = ZipArchive::new(buf)?;
            let mut f = ar.by_name(src)?;
            size = io::copy(&mut f, &mut dst)?;
        } else {
            size = io::copy(&mut buf, &mut dst)?;
        }

        Ok(size)
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
