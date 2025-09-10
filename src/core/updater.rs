#[allow(dead_code)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

#[allow(dead_code)]
pub async fn check_for_updates(_current: &str) -> anyhow::Result<Option<UpdateInfo>> {
    // TODO: реализовать проверку релизов на /в github
    Ok(None)
}