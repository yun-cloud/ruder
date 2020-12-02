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

    latest_release.assets[5].download(&client, "./output").await?;

    Ok(())
}
