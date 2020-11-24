use log::info;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    name: String,
    assets: Vec<Asset>,
    body: String,
    author: Author,
}

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

#[derive(Debug, Deserialize)]
struct Author {
    login: String,
}

async fn query_latest_release(owner: &str, repo: &str) -> anyhow::Result<Release> {
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
        .query(&[("per_page", "5")])
        .query(&[("page", "2")])
        .send()
        .await?;

    let latest_release: Release = response.json().await?;

    Ok(latest_release)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let latest_release = query_latest_release("BurntSushi", "ripgrep").await?;
    info!("latest_release: {:#?}", latest_release);

    Ok(())
}
