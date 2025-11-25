use anyhow::Result;
use reqwest::Client;
use crate::models::{Sdk, SdkVersion};
use crate::local::scanner::LocalScanner;
use super::{endpoints::ApiEndpoints, parser::ResponseParser};

pub struct SdkmanApiClient {
    http_client: Client,
}

impl SdkmanApiClient {
    pub fn new() -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("SDKMAN-GUI/1.0 (Tauri)")
            .build()?;

        Ok(Self { http_client })
    }

    /// 获取候选列表
    pub async fn list_candidates(&self) -> Result<Vec<Sdk>> {
        let url = ApiEndpoints::candidates_list();
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch candidates: HTTP {}", response.status());
        }

        let body = response.text().await?;
        ResponseParser::parse_candidates(&body)
    }

    /// 获取版本列表
    pub async fn list_versions(&self, candidate: &str) -> Result<Vec<SdkVersion>> {
        let platform = detect_platform();

        // 从本地文件系统读取已安装版本和当前版本
        let installed_versions = LocalScanner::scan_installed_versions(candidate)
            .unwrap_or_default();
        let installed = installed_versions.join(",");

        let current = LocalScanner::get_current_version(candidate)
            .unwrap_or(None)
            .unwrap_or_default();

        let url = ApiEndpoints::versions_list(candidate, &platform, &installed, &current);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch versions for {}: HTTP {}", candidate, response.status());
        }

        let body = response.text().await?;
        ResponseParser::parse_versions(&body, candidate)
    }
}

/// 检测当前平台（SDKMAN格式）
///
/// 返回值示例:
/// - `darwinarm64` - macOS on Apple Silicon
/// - `darwinx64` - macOS on Intel
/// - `linuxarm64` - Linux ARM 64-bit
/// - `linuxx64` - Linux x86_64
/// - `windowsx64` - Windows x86_64
pub fn detect_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let os_prefix = match os {
        "windows" => "windows",
        "macos" => "darwin",
        "linux" => "linux",
        _ => "universal",
    };

    let arch_suffix = match arch {
        "aarch64" => "arm64",
        "arm" => "arm32hf",
        "x86_64" => "x64",
        "x86" => "x32",
        _ => "x64", // 默认
    };

    format!("{}{}", os_prefix, arch_suffix)
}
