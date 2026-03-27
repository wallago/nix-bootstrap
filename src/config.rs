pub mod prelude {
    pub use super::Config;
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub cargo_pkg_name: String,
    pub project_name: String,
}

impl Config {
    pub fn from_env() -> color_eyre::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            cargo_pkg_name: std::env::var("CARGO_PKG_NAME")
                .map_err(|_| color_eyre::eyre::eyre!("CARGO_PKG_NAME env var is required"))?,
            project_name: std::env::var("CARGO_CRATE_NAME")
                .map_err(|_| color_eyre::eyre::eyre!("CARGO_CRATE_NAME env var is required"))?,
        })
    }
}
