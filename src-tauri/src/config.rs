use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 代理类型: none, auto, manual
    pub proxy_type: String,

    /// 代理主机
    pub proxy_host: Option<String>,

    /// 代理端口
    pub proxy_port: Option<String>,

    /// 主题: light, dark, auto
    pub theme: String,

    /// 语言: en, zh, auto (auto = 根据系统语言自动检测)
    pub language: String,

    /// SDKMAN路径
    pub sdkman_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            proxy_type: "none".to_string(),
            proxy_host: None,
            proxy_port: None,
            theme: "auto".to_string(),
            language: "auto".to_string(), // 默认使用 auto，由前端根据系统语言自动检测
            sdkman_path: "~/.sdkman".to_string(),
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;

        let config_dir = home.join(".config").join("sdkman-gui");

        // 确保配置目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }

        Ok(config_dir.join("settings.json"))
    }

    /// 加载配置
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            // 配置文件不存在，返回默认配置
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read config file")?;

        let config: AppConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// 获取代理URL（如果配置了代理）
    pub fn get_proxy_url(&self) -> Option<String> {
        match self.proxy_type.as_str() {
            "manual" => {
                if let (Some(host), Some(port)) = (&self.proxy_host, &self.proxy_port) {
                    Some(format!("http://{}:{}", host, port))
                } else {
                    None
                }
            }
            "auto" => {
                // 从环境变量读取代理设置
                std::env::var("HTTP_PROXY").ok()
                    .or_else(|| std::env::var("http_proxy").ok())
            }
            _ => None
        }
    }
}
