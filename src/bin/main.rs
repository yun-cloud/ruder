use log::info;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use url::Url;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let request_url = Url::parse(&format!(
        "https://api.github.com/repos/{owner}/{repo}/releases",
        owner = "BurntSushi",
        repo = "ripgrep"
    ))?;
    info!("request_url: {}", request_url);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static("reqwest_try"));
    let client = Client::builder().default_headers(headers).build()?;
    info!("client: {:?}", client);

    let response = client
        .get(request_url.as_str())
        .header(header::ACCEPT, "application/vnd.github.v3+json")
        .query(&[("per_page", "5")])
        .query(&[("page", "2")])
        .send()
        .await?;
    info!("response: {:?}", response);

    // let users: Vec<User> = response.json().await?;
    // info!("users.len(): {}", users.len());
    // info!("users: {:#?}", users);

    let body = response.text().await?;
    info!("body = {}", body);

    fs::write("body.json", &body)?;

    Ok(())
}
