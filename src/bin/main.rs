use std::path::Path;

use query_the_github_api::extract::unpack;
use query_the_github_api::github::{create_github_client, query_latest_release};

use anyhow::anyhow;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let owner = "BurntSushi";
    let repo = "ripgrep";
    let tmpdir = Path::new("./output");
    let asset_download_filename = Path::new("ripgrep-12.1.1-x86_64-unknown-linux-musl.tar.gz");
    let src = Path::new("ripgrep-12.1.1-x86_64-unknown-linux-musl/rg");
    let dst = Path::new("./bin/rs");
    info!("owner: {:?}", owner);
    info!("repo: {:?}", repo);
    info!("tmpdir: {:?}", tmpdir);
    info!("asset_download_filename: {:?}", asset_download_filename);
    info!("src: {:?}", src);
    info!("dst: {:?}", dst);

    let client = create_github_client().await?;
    let latest_release = query_latest_release(&client, owner, repo).await?;
    // info!("latest_release: {:#?}", latest_release);

    let download_asset = latest_release
        .assets
        .iter()
        .find(|asset| {
            asset
                .download_filename()
                .map(|filename| filename == asset_download_filename)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow!(
                "{:?} is not exist in latest release of {}/{}",
                asset_download_filename,
                repo,
                owner
            )
        })?;
    let filepath = download_asset.download(&client, tmpdir).await?;
    unpack(filepath, tmpdir)?;

    Ok(())
}
