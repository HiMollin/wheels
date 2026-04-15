mod database;
mod server;

use std::sync::LazyLock;

use anyhow::Context;
use config::{Config, FileFormat};
use serde::Deserialize;

pub use database::DatabaseConfig;
pub use server::ServerConfig;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
}

/// 全局单例
static CONFIG: LazyLock<AppConfig> =
    LazyLock::new(|| AppConfig::load().expect("Failed to initialize config"));

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Config::builder()
            // 读配置文件
            .add_source(
                config::File::with_name("config")
                    .format(FileFormat::Toml)
                    .required(true),
            )
            // 读环境变量
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(","),
            )
            .build()
            .with_context(|| anyhow::anyhow!("Failed to load config"))?
            // 上一步拿到 Config 类型，现在将他反序列化
            .try_deserialize()
            .with_context(|| anyhow::anyhow!("Failed to deserialize config"))
    }

    pub fn server(&self) -> &ServerConfig {
        &self.server
    }

    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }
}

pub fn get() -> &'static AppConfig {
    &CONFIG
}
