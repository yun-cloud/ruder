use log::info;
use query_the_github_api::github::{create_github_client, query_latest_release};
use skip_error::skip_error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let client = create_github_client().await?;

    let latest_release = query_latest_release(&client, "BurntSushi", "ripgrep").await?;
    // info!("latest_release: {:#?}", latest_release);

    for asset in &latest_release.assets {
        let filepath = skip_error!(asset.download_filename());
        info!(
            "filepath: {:?}, {:?}, {:?}",
            filepath,
            filepath.file_stem(),
            filepath.extension()
        );
    }

    /*
     *     let asset = &latest_release.assets[5];
     *     let url = &asset.browser_download_url;
     *     let filepath = asset.download_filename()?;
     *     info!("url: {:?}", url);
     *
     *     let mut headers = header::HeaderMap::new();
     *     headers.insert(header::USER_AGENT, HeaderValue::from_static("reqwest_try"));
     *     let client = Client::builder()
     *         .redirect(Policy::limited(100))
     *         .default_headers(headers)
     *         .build()?;
     *
     *     let response = client
     *         .get(url)
     *         .header(header::ACCEPT, "application/vnd.github.v3+json")
     *         .send()
     *         .await?;
     *     info!("response: {:#?}", response);
     *     let content = response.text().await?;
     *
     *     let mut dest = {
     *         let filename = filepath.file_name().unwrap();
     *         File::create(filename)?
     *     };
     *     copy(&mut content.as_bytes(), &mut dest)?;
     */

    Ok(())
}
