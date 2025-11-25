use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use reqwest::Client;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, AppHandle};
use crate::api::{endpoints::ApiEndpoints, detect_platform};
use crate::config::AppConfig;

/// 下载进度事件
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DownloadProgress {
    pub candidate: String,
    pub version: String,
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
}

/// SDK 下载器
pub struct Downloader {
    client: Client,
}

impl Downloader {
    /// 创建新的下载器实例
    pub fn new() -> Result<Self> {
        // 加载配置以获取代理设置
        let config = AppConfig::load().unwrap_or_default();

        println!("=== Downloader Configuration ===");
        println!("Proxy type: {}", config.proxy_type);
        println!("Proxy host: {:?}", config.proxy_host);
        println!("Proxy port: {:?}", config.proxy_port);

        let mut client_builder = Client::builder()
            .timeout(std::time::Duration::from_secs(300))  // 5分钟总超时
            .connect_timeout(std::time::Duration::from_secs(30))  // 30秒连接超时
            .redirect(reqwest::redirect::Policy::limited(10));  // 允许最多10次重定向

        // 如果配置了代理，应用代理设置
        if let Some(proxy_url) = config.get_proxy_url() {
            println!("✓ Using proxy: {}", proxy_url);
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
                println!("✓ Proxy configured successfully");
            } else {
                println!("✗ Failed to configure proxy");
            }
        } else {
            println!("✗ No proxy configured (direct connection)");
        }

        let client = client_builder
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// 下载SDK并返回临时文件路径
    ///
    /// # Arguments
    /// * `candidate` - SDK 候选者名称
    /// * `version` - 版本号
    /// * `app` - Tauri 应用句柄，用于发送进度事件
    ///
    /// # Returns
    /// 返回下载的临时文件路径
    pub async fn download_sdk(
        &self,
        candidate: &str,
        version: &str,
        app: AppHandle,
    ) -> Result<PathBuf> {
        // 1. 获取下载URL
        let platform = detect_platform();
        let url = ApiEndpoints::download(candidate, version, &platform);

        println!("=== Download Information ===");
        println!("Candidate: {}", candidate);
        println!("Version: {}", version);
        println!("Platform: {}", platform);
        println!("Initial URL: {}", url);

        // 2. 发送HTTP请求
        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to send download request")?;

        // 检查响应状态
        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        // 记录最终的URL（重定向后）
        let final_url = response.url().clone();
        println!("Final URL (after redirects): {}", final_url);

        // 获取文件总大小
        let total_size = response.content_length().unwrap_or(0);

        println!("Total size: {} bytes", total_size);

        // 3. 检测文件类型并创建临时文件
        // 优先使用 SDKMAN 的 X-Sdkman-ArchiveType 响应头
        let archive_type = response.headers()
            .get("x-sdkman-archivetype")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // 检查 Content-Disposition 头中的文件名
        let content_disposition = response.headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // 根据多种因素确定文件扩展名
        let file_extension = if !archive_type.is_empty() {
            // 优先使用 SDKMAN 提供的类型
            if archive_type == "zip" {
                "zip"
            } else if archive_type == "tar.gz" || archive_type == "tgz" {
                "tar.gz"
            } else {
                archive_type  // 直接使用
            }
        } else if content_disposition.contains(".zip") {
            "zip"
        } else if content_disposition.contains(".tar.gz") || content_disposition.contains(".tgz") {
            "tar.gz"
        } else if content_type.contains("zip") || content_type.contains("application/x-zip") {
            "zip"
        } else if content_type.contains("gzip") || content_type.contains("x-gzip") {
            "tar.gz"
        } else {
            // 某些 SDK（如 Gradle）默认为 zip
            // 根据 candidate 类型判断
            match candidate {
                "gradle" | "maven" | "ant" => "zip",
                _ => "tar.gz",  // JDK 等默认 tar.gz
            }
        };

        println!("X-Sdkman-ArchiveType: {}", archive_type);
        println!("Content-Type: {}", content_type);
        println!("Content-Disposition: {}", content_disposition);
        println!("Using file extension: {}", file_extension);

        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("{}-{}.{}", candidate, version, file_extension));
        let mut temp_file = File::create(&temp_file_path)
            .context("Failed to create temporary file")?;

        // 4. 下载文件，报告进度
        let mut downloaded: u64 = 0;
        let mut last_reported_downloaded: u64 = 0;
        let mut last_reported_percentage: f64 = 0.0;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;

        // 节流配置：每下载1MB或进度增加1%时发送事件
        const REPORT_INTERVAL_BYTES: u64 = 1024 * 1024; // 1MB
        const REPORT_INTERVAL_PERCENTAGE: f64 = 1.0; // 1%

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            temp_file.write_all(&chunk)
                .context("Failed to write to temporary file")?;

            downloaded += chunk.len() as u64;

            // 计算百分比
            let percentage = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            // 节流：只在以下情况发送事件
            // 1. 下载了超过1MB数据
            // 2. 进度增加了超过1%
            // 3. 下载完成（100%）
            let should_report =
                downloaded - last_reported_downloaded >= REPORT_INTERVAL_BYTES
                || percentage - last_reported_percentage >= REPORT_INTERVAL_PERCENTAGE
                || downloaded == total_size;

            if should_report {
                // 发送进度事件
                let progress = DownloadProgress {
                    candidate: candidate.to_string(),
                    version: version.to_string(),
                    downloaded,
                    total: total_size,
                    percentage,
                };

                // 忽略发送错误（前端可能未监听）
                let _ = app.emit("download-progress", &progress);

                // 更新上次报告的值
                last_reported_downloaded = downloaded;
                last_reported_percentage = percentage;

                // 打印进度
                println!("Downloaded: {} / {} bytes ({:.2}%)",
                    downloaded, total_size, percentage);
            }
        }

        // 5. 确保文件写入完成
        temp_file.flush()
            .context("Failed to flush temporary file")?;

        // 显式关闭文件句柄
        drop(temp_file);

        // 验证下载的文件
        let file_size = std::fs::metadata(&temp_file_path)
            .context("Failed to get file metadata")?
            .len();

        println!("Download completed: {:?}", temp_file_path);
        println!("Downloaded file size: {} bytes (expected: {} bytes)", file_size, total_size);

        if file_size != downloaded {
            anyhow::bail!("File size mismatch: downloaded {} bytes but file is {} bytes", downloaded, file_size);
        }

        Ok(temp_file_path)
    }

    /// 下载SDK（简化版，不报告进度）
    ///
    /// # Arguments
    /// * `candidate` - SDK 候选者名称
    /// * `version` - 版本号
    ///
    /// # Returns
    /// 返回下载的临时文件路径
    pub async fn download_sdk_simple(
        &self,
        candidate: &str,
        version: &str,
    ) -> Result<PathBuf> {
        // 1. 获取下载URL
        let platform = detect_platform();
        let url = ApiEndpoints::download(candidate, version, &platform);

        println!("Downloading SDK from: {}", url);

        // 2. 发送HTTP请求并下载
        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to send download request")?;

        // 检查响应状态
        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        // 3. 创建临时文件
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("{}-{}.zip", candidate, version));

        // 4. 下载文件
        let bytes = response.bytes()
            .await
            .context("Failed to download file")?;

        // 5. 写入文件
        let mut temp_file = File::create(&temp_file_path)
            .context("Failed to create temporary file")?;

        temp_file.write_all(&bytes)
            .context("Failed to write to temporary file")?;

        println!("Download completed: {:?}", temp_file_path);

        Ok(temp_file_path)
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create Downloader")
    }
}
