use log::info;
use query_the_github_api::github::query_latest_release;
use skip_error::skip_error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let latest_release = query_latest_release("BurntSushi", "ripgrep").await?;
    info!("latest_release: {:#?}", latest_release);

    for asset in latest_release.assets {
        let filepath = skip_error!(asset.download_filename());
        info!(
            "filepath: {:?}, {:?}, {:?}",
            filepath,
            filepath.file_stem(),
            filepath.extension()
        );
    }

    Ok(())
}
