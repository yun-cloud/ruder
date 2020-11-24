use log::info;
use query_the_github_api::github::query_latest_release;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let latest_release = query_latest_release("BurntSushi", "ripgrep").await?;
    info!("latest_release: {:#?}", latest_release);

    Ok(())
}
