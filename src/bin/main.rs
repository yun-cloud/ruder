use std::fs;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use query_the_github_api::extract;
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
        // info!("latest_release: {:#?}", latest_release);
        info!("latest_release.tag_name: {:?}", latest_release.tag_name);
        info!("latest_release.name: {:?}", latest_release.name);

        let (download_asset, download_filename) = latest_release
            .assets
            .iter()
            .filter_map(|asset| asset.download_filename().ok().map(|name| (asset, name)))
            .find(|(_, name)| name == &asset_download_filename)
            .ok_or_else(|| {
                anyhow!(
                    "{:?} is not exist in latest release of {}/{}",
                    asset_download_filename,
                    repo,
                    owner
                )
            })?;

        let executable = {
            let data = download_asset.download(&client).await?;
            let filename = download_filename
                .to_str()
                .expect("download_filename failed to convert to &str");
            extract(Cursor::new(data), filename, &binary.src)?
        };

        fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Fail to create all dir for {:?}", bin_dir))?;
        let mut dst_f = File::create(&dst)?;

        io::copy(&mut Cursor::new(executable), &mut dst_f)?;

        // let filepath = download_asset
        //     .download(&client, &tmp_dir)
        //     .await
        //     .with_context(|| "Fail to download asset")?;
        // if let Err(_) = unpack(filepath, &tmp_dir) {
        //     warn!("Failed to unpack, assume this asset is a executable");
        // }

        // fs::create_dir_all(&bin_dir)
        //     .with_context(|| format!("Fail to create all dir for {:?}", bin_dir))?;
        // fs::rename(&src, &dst).with_context(|| format!("Fail to move {:?} to {:?}", src, dst))?;

        let mut perms = fs::metadata(&dst)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dst, perms)?;
    }

    Ok(())
}
