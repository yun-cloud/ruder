use std::fs;
use std::path::PathBuf;

use query_the_github_api::extract::unpack;
use query_the_github_api::github::{create_github_client, query_latest_release};
use query_the_github_api::Config;

use anyhow::anyhow;
use anyhow::Context;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let config: Config =
        toml::from_slice(&fs::read("binary.toml").with_context(|| "Fail to read 'binary.toml'")?)
            .with_context(|| "Fail to deserialize 'binary.toml'")?;
    // info!("binary_data: {:#?}", config);

    let tmp_dir = config.tmp_dir();
    let bin_dir = config.bin_dir();
    info!("tmp_dir: {:?}", tmp_dir);
    info!("bin_dir: {:?}", bin_dir);

    let client = create_github_client()
        .await
        .with_context(|| "Fail to create github client")?;
    for binary in config.binaries() {
        info!("binary: {:#?}", binary);
        let owner = &binary.owner;
        let repo = &binary.repo;
        let asset_download_filename = PathBuf::from(&binary.asset_download_filename);
        let src = tmp_dir.join(&binary.src);
        let dst = bin_dir.join(&binary.dst);
        info!("src: {:?}", src);
        info!("dst: {:?}", dst);

        let latest_release = query_latest_release(&client, &owner, &repo)
            .await
            .with_context(|| "Fail to query latest release")?;
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
        let filepath = download_asset
            .download(&client, &tmp_dir)
            .await
            .with_context(|| "Fail to download asset")?;
        unpack(filepath, &tmp_dir).with_context(|| "Fail to unpack")?;

        fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Fail to create all dir for {:?}", bin_dir))?;
        fs::rename(&src, &dst).with_context(|| format!("Fail to move {:?} to {:?}", src, dst))?;
    }

    Ok(())
}
