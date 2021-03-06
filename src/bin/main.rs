use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use ruder::Archive;
use ruder::BinaryTable;
use ruder::Config;

use anyhow::anyhow;
use anyhow::Context;
use log::info;
use reqwest::Client;
use ruder::github::{create_github_client, query_latest_release};
use tokio::fs::{self, File};
use tokio::io;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let config: Config = {
        let content = fs::read("binary.toml")
            .await
            .with_context(|| "Fail to read 'binary.toml'")?;
        toml::from_slice(&content).with_context(|| "Fail to deserialize 'binary.toml'")?
    };

    let client = create_github_client()
        .await
        .with_context(|| "Fail to create github client")?;

    for binary in config.binaries() {
        if let Err(err) = run_on_binary(&client, &config, &binary).await {
            eprintln!("run_on_binary - err: {:?}", err);
            continue;
        }
    }

    Ok(())
}

async fn run_on_binary(
    client: &Client,
    config: &Config,
    binary: &BinaryTable,
) -> anyhow::Result<()> {
    info!("===========================================================================");
    log::info!("binary.repo(): {:?}", binary.repo());

    let bin_dir = config.bin_dir();
    // info!("bin_dir: {:?}", bin_dir);
    fs::create_dir_all(&bin_dir)
        .await
        .with_context(|| format!("Fail to create all dir for {:?}", bin_dir))?;

    let latest_release = query_latest_release(&client, binary.repo())
        .await
        .with_context(|| "Fail to query latest release")?;
    // info!("latest_release: {:#?}", latest_release);

    let version = latest_release.version()?;
    log::info!("version of release: {}", version);

    if !config.need_to_upgrade(&binary, &version) {
        return Ok(());
    }

    let src = binary.src(&version);
    let dst = bin_dir.join(binary.dst());
    let asset_download_filename = PathBuf::from(&binary.asset_download_filename(&version));

    let (download_asset, download_filename) = latest_release
        .assets
        .iter()
        .filter_map(|asset| asset.download_filename().ok().map(|name| (asset, name)))
        .find(|(_, name)| name == &asset_download_filename)
        .ok_or_else(|| {
            anyhow!(
                "{:?} is not exist in latest release of {}",
                asset_download_filename,
                binary.repo(),
            )
        })?;
    let data = download_asset
        .download(&client)
        .await
        .with_context(|| "Failed to download asset")?;

    let filename = download_filename
        .to_str()
        .expect("download_filename fail to convert to &str");
    let mut ar = Archive::new(data, filename).with_context(|| "Fail to create archive")?;
    let mut executable = ar
        .extract(&src)
        .with_context(|| "Fail to extract archive")?;

    let mut dst_f = File::create(&dst)
        .await
        .with_context(|| "Fail to create destination file")?;
    io::copy(&mut executable, &mut dst_f)
        .await
        .with_context(|| "fail to copy download executable to destination file")?;

    let mut perms = fs::metadata(&dst)
        .await
        .with_context(|| "Fail to get metadata")?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dst, perms)
        .await
        .with_context(|| "Fail to set permissions")?;

    Ok(())
}
