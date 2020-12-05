use std::path::Path;

use query_the_github_api::extract::unpack;
use query_the_github_api::github::{create_github_client, query_latest_release};

use anyhow::anyhow;
use log::info;
use skip_error::skip_error_and_log;

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

    let mut found = false;
    for asset in &latest_release.assets {
        let download_filename = skip_error_and_log!(asset.download_filename(), log::Level::Info);
        if download_filename != asset_download_filename {
            continue;
        }
        found = true;
        let filepath = asset.download(&client, tmpdir).await?;
        unpack(filepath, tmpdir)?;
        break;
    }

    if found == false {
        return Err(anyhow!(
            "{:?} is not exist in latest release of {}/{}",
            asset_download_filename,
            repo,
            owner
        ));
    }

    Ok(())
}
