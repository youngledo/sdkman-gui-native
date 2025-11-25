use tauri::{State, AppHandle};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::api::SdkmanApiClient;
use crate::models::{SdkVersion, Sdk, Statistics};
use crate::local::{LocalScanner, Downloader, Installer, SymlinkManager};
use crate::config::AppConfig;
use crate::cache;
use std::path::Path;

#[tauri::command]
pub async fn list_jdk_versions(
    force_refresh: bool,
    client: State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<SdkVersion>, String> {
    // 1. 如果不强制刷新，先尝试读取缓存
    let mut versions = if !force_refresh {
        if let Ok(Some(cached)) = cache::read_jdk_versions_cache() {
            println!("Using cached JDK versions ({} items)", cached.len());
            cached
        } else {
            println!("No cache found, fetching from API");
            fetch_and_cache_jdk_versions(&client).await?
        }
    } else {
        println!("Force refresh, fetching from API");
        fetch_and_cache_jdk_versions(&client).await?
    };

    // 2. 扫描本地已安装版本
    let installed_versions = LocalScanner::scan_installed_versions("java")
        .map_err(|e| e.to_string())?;

    // 3. 获取当前版本
    let current_version = LocalScanner::get_current_version("java")
        .map_err(|e| e.to_string())?;

    // 4. 标记已安装和当前版本
    for version in &mut versions {
        version.installed = installed_versions.contains(&version.identifier);
        version.in_use = current_version.as_ref() == Some(&version.identifier);
        version.is_default = version.in_use;
    }

    Ok(versions)
}

/// 从API获取JDK版本并缓存
async fn fetch_and_cache_jdk_versions(
    client: &State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<SdkVersion>, String> {
    let client_guard = client.lock().await;
    let versions = client_guard.list_versions("java")
        .await
        .map_err(|e| e.to_string())?;
    drop(client_guard);

    // 缓存结果
    if let Err(e) = cache::cache_jdk_versions(&versions) {
        eprintln!("Warning: Failed to cache JDK versions: {}", e);
    }

    Ok(versions)
}

#[tauri::command]
pub async fn list_sdk_candidates(
    force_refresh: bool,
    client: State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<Sdk>, String> {
    // 如果不强制刷新，先尝试读取缓存
    if !force_refresh {
        if let Ok(Some(cached)) = cache::read_sdk_candidates_cache() {
            println!("Using cached SDK candidates ({} items)", cached.len());
            return Ok(cached);
        }
    }

    println!("Fetching SDK candidates from API");
    let client_guard = client.lock().await;
    let candidates = client_guard.list_candidates()
        .await
        .map_err(|e| e.to_string())?;
    drop(client_guard);

    // 缓存结果
    if let Err(e) = cache::cache_sdk_candidates(&candidates) {
        eprintln!("Warning: Failed to cache SDK candidates: {}", e);
    }

    Ok(candidates)
}

/// 获取指定SDK的版本列表（通用版本，支持所有candidate）
#[tauri::command]
pub async fn list_sdk_versions(
    candidate: String,
    force_refresh: bool,
    client: State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<SdkVersion>, String> {
    // 1. 如果不强制刷新，先尝试读取缓存
    let mut versions = if !force_refresh {
        if let Ok(Some(cached)) = cache::read_sdk_versions_cache(&candidate) {
            println!("Using cached {} versions ({} items)", candidate, cached.len());
            cached
        } else {
            println!("No cache found for {}, fetching from API", candidate);
            fetch_and_cache_sdk_versions(&candidate, &client).await?
        }
    } else {
        println!("Force refresh for {}, fetching from API", candidate);
        fetch_and_cache_sdk_versions(&candidate, &client).await?
    };

    // 2. 扫描本地已安装版本
    let installed_versions = LocalScanner::scan_installed_versions(&candidate)
        .map_err(|e| e.to_string())?;

    // 3. 获取当前版本
    let current_version = LocalScanner::get_current_version(&candidate)
        .map_err(|e| e.to_string())?;

    // 4. 标记已安装和当前版本
    for version in &mut versions {
        version.installed = installed_versions.contains(&version.identifier);
        version.in_use = current_version.as_ref() == Some(&version.identifier);
        version.is_default = version.in_use;
    }

    Ok(versions)
}

/// 从API获取SDK版本并缓存
async fn fetch_and_cache_sdk_versions(
    candidate: &str,
    client: &State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Vec<SdkVersion>, String> {
    let client_guard = client.lock().await;
    let versions = client_guard.list_versions(candidate)
        .await
        .map_err(|e| e.to_string())?;
    drop(client_guard);

    // 缓存结果
    if let Err(e) = cache::cache_sdk_versions(candidate, &versions) {
        eprintln!("Warning: Failed to cache {} versions: {}", candidate, e);
    }

    Ok(versions)
}

#[tauri::command]
pub async fn get_statistics(
    client: State<'_, Arc<Mutex<SdkmanApiClient>>>
) -> Result<Statistics, String> {
    // 1. 获取已安装的JDK数量
    let jdk_installed = LocalScanner::scan_installed_versions("java")
        .map_err(|e| e.to_string())?
        .len();

    // 2. 获取可用的JDK数量
    let client_guard = client.lock().await;
    let jdk_versions = client_guard.list_versions("java")
        .await
        .map_err(|e| e.to_string())?;
    let jdk_available = jdk_versions.len();

    // 3. 获取所有候选者
    let candidates = client_guard.list_candidates()
        .await
        .map_err(|e| e.to_string())?;

    // 释放锁，以便后续操作可以使用
    drop(client_guard);

    let sdk_available = candidates.len();

    // 4. 获取已安装的SDK总数（所有候选者的已安装版本数）
    let mut sdk_installed = 0usize;
    for candidate in &candidates {
        if let Ok(versions) = LocalScanner::scan_installed_versions(&candidate.candidate) {
            sdk_installed += versions.len();
        }
    }

    Ok(Statistics {
        jdk_installed,
        jdk_available,
        sdk_installed,
        sdk_available,
    })
}

/// 扫描指定候选者的已安装版本
#[tauri::command]
pub async fn scan_installed_sdks(candidate: String) -> Result<Vec<String>, String> {
    LocalScanner::scan_installed_versions(&candidate)
        .map_err(|e| e.to_string())
}

/// 获取当前默认版本
#[tauri::command]
pub async fn get_current_sdk_version(candidate: String) -> Result<Option<String>, String> {
    LocalScanner::get_current_version(&candidate)
        .map_err(|e| e.to_string())
}

/// 获取所有已安装的候选者列表
#[tauri::command]
pub async fn list_installed_candidates() -> Result<Vec<String>, String> {
    LocalScanner::list_installed_candidates()
        .map_err(|e| e.to_string())
}

/// 检查指定版本是否已安装
#[tauri::command]
pub async fn is_sdk_installed(candidate: String, version: String) -> Result<bool, String> {
    LocalScanner::is_version_installed(&candidate, &version)
        .map_err(|e| e.to_string())
}

/// 下载SDK（带进度报告）
#[tauri::command]
pub async fn download_sdk(
    candidate: String,
    version: String,
    app: AppHandle,
) -> Result<String, String> {
    let downloader = Downloader::new()
        .map_err(|e| e.to_string())?;

    let temp_path = downloader.download_sdk(&candidate, &version, app)
        .await
        .map_err(|e| e.to_string())?;

    Ok(temp_path.to_string_lossy().to_string())
}

/// 下载SDK（简化版，不报告进度）
#[tauri::command]
pub async fn download_sdk_simple(
    candidate: String,
    version: String,
) -> Result<String, String> {
    let downloader = Downloader::new()
        .map_err(|e| e.to_string())?;

    let temp_path = downloader.download_sdk_simple(&candidate, &version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(temp_path.to_string_lossy().to_string())
}

/// 从ZIP文件安装SDK
#[tauri::command]
pub async fn install_sdk(
    zip_path: String,
    candidate: String,
    version: String,
    app: AppHandle,
) -> Result<String, String> {
    let zip_path = Path::new(&zip_path);

    let install_path = Installer::install_from_zip(zip_path, &candidate, &version, app)
        .map_err(|e| e.to_string())?;

    // 如果这是该候选者的唯一已安装版本，自动设置为默认版本
    let installed_versions = LocalScanner::scan_installed_versions(&candidate)
        .map_err(|e| e.to_string())?;

    println!("Installed versions for {}: {:?} (count: {})", candidate, installed_versions, installed_versions.len());

    if installed_versions.len() == 1 {
        println!("Only one version installed for {}, setting {} as default", candidate, version);
        if let Err(e) = SymlinkManager::set_default_version(&candidate, &version) {
            eprintln!("Warning: Failed to set default version: {}", e);
        } else {
            println!("Successfully set {} {} as default version", candidate, version);
        }
    } else {
        println!("Multiple versions ({}) installed, not setting default automatically", installed_versions.len());
    }

    Ok(install_path.to_string_lossy().to_string())
}

/// 卸载SDK
#[tauri::command]
pub async fn uninstall_sdk(
    candidate: String,
    version: String,
    app: AppHandle,
) -> Result<(), String> {
    Installer::uninstall_sdk(&candidate, &version, app)
        .map_err(|e| e.to_string())
}

/// 验证SDK安装
#[tauri::command]
pub async fn verify_sdk_installation(
    candidate: String,
    version: String,
) -> Result<bool, String> {
    Installer::verify_installation(&candidate, &version)
        .map_err(|e| e.to_string())
}

/// 下载并安装SDK（组合命令）
#[tauri::command]
pub async fn download_and_install_sdk(
    candidate: String,
    version: String,
    app: AppHandle,
) -> Result<String, String> {
    // 1. 下载SDK
    let downloader = Downloader::new()
        .map_err(|e| e.to_string())?;

    let temp_path = downloader.download_sdk(&candidate, &version, app.clone())
        .await
        .map_err(|e| e.to_string())?;

    // 2. 安装SDK
    let install_path = Installer::install_from_zip(&temp_path, &candidate, &version, app)
        .map_err(|e| e.to_string())?;

    // 3. 清理临时文件
    if let Err(e) = std::fs::remove_file(&temp_path) {
        eprintln!("Warning: Failed to remove temporary file {:?}: {}", temp_path, e);
    }

    // 4. 如果这是该候选者的唯一已安装版本，自动设置为默认版本
    let installed_versions = LocalScanner::scan_installed_versions(&candidate)
        .map_err(|e| e.to_string())?;

    println!("Installed versions for {}: {:?} (count: {})", candidate, installed_versions, installed_versions.len());

    if installed_versions.len() == 1 {
        println!("Only one version installed for {}, setting {} as default", candidate, version);
        if let Err(e) = SymlinkManager::set_default_version(&candidate, &version) {
            eprintln!("Warning: Failed to set default version: {}", e);
        } else {
            println!("Successfully set {} {} as default version", candidate, version);
        }
    } else {
        println!("Multiple versions ({}) installed, not setting default automatically", installed_versions.len());
    }

    Ok(install_path.to_string_lossy().to_string())
}

/// 设置默认SDK版本
#[tauri::command]
pub async fn set_default_sdk_version(
    candidate: String,
    version: String,
) -> Result<(), String> {
    SymlinkManager::set_default_version(&candidate, &version)
        .map_err(|e| e.to_string())
}

/// 取消默认SDK版本设置
#[tauri::command]
pub async fn unset_default_sdk_version(
    candidate: String,
) -> Result<(), String> {
    SymlinkManager::unset_default_version(&candidate)
        .map_err(|e| e.to_string())
}

/// 加载配置
#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    AppConfig::load()
        .map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    config.save()
        .map_err(|e| e.to_string())
}

/// 测试代理配置
#[tauri::command]
pub async fn test_proxy() -> Result<String, String> {
    let config = AppConfig::load()
        .map_err(|e| e.to_string())?;

    let mut result = String::new();
    result.push_str(&format!("Proxy Type: {}\n", config.proxy_type));
    result.push_str(&format!("Proxy Host: {:?}\n", config.proxy_host));
    result.push_str(&format!("Proxy Port: {:?}\n", config.proxy_port));

    if let Some(proxy_url) = config.get_proxy_url() {
        result.push_str(&format!("Proxy URL: {}\n", proxy_url));
        result.push_str("Status: Proxy will be used for downloads\n");
    } else {
        result.push_str("Status: No proxy configured (direct connection)\n");
    }

    Ok(result)
}

/// 获取SDKMAN目录路径
#[tauri::command]
pub async fn get_sdkman_path() -> Result<String, String> {
    let home = dirs::home_dir()
        .ok_or("Failed to get home directory")?;
    let sdkman_path = home.join(".sdkman");
    Ok(sdkman_path.to_string_lossy().to_string())
}
