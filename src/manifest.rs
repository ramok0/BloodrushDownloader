
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mirror {
    pub name: String,
    pub url: String,
    pub note: Option<String>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub mirrors: Vec<Mirror>,
    pub current_exe_hash: u64,
    pub current_pak_hash: u64
}

pub const MANIFEST_URL:&'static str = "https://pastebin.com/raw/4ZErJnqs";

impl Manifest {
    pub async fn from_url(url:&str) -> Result<Self, Box<dyn std::error::Error>> {
        let response = reqwest::get(url).await?;
        let manifest:Manifest = response.json().await?;

        Ok(manifest)
    }
}