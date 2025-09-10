use anyhow::Result;

pub async fn warmup() -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("Launcher/0.1")
        .build()?;
    let _ = client.get("https://github.com/netherfall-minecraft/launcher").send().await?;
    Ok(())
}
