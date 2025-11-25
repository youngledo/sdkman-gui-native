use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::models::{SdkVersion, Sdk};

const JDK_VERSIONS_CACHE: &str = "jdk_versions.json";
const SDK_CANDIDATES_CACHE: &str = "sdk_candidates.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct SdkVersionsCache {
    pub candidate: String,
    pub versions: Vec<SdkVersion>,
    pub timestamp: u64,
}

/// 获取缓存目录路径（与配置文件相同的目录）
fn get_cache_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .context("Failed to get home directory")?;

    // 使用与配置文件相同的目录结构
    let cache_dir = home_dir.join(".config").join("sdkman-gui").join("cache");

    // 确保缓存目录存在
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;
    }

    Ok(cache_dir)
}

/// 获取当前时间戳（秒）
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 缓存JDK版本列表
pub fn cache_jdk_versions(versions: &[SdkVersion]) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(JDK_VERSIONS_CACHE);

    let cache_data = SdkVersionsCache {
        candidate: "java".to_string(),
        versions: versions.to_vec(),
        timestamp: get_timestamp(),
    };

    let json = serde_json::to_string_pretty(&cache_data)
        .context("Failed to serialize JDK versions cache")?;

    fs::write(&cache_file, json)
        .context("Failed to write JDK versions cache")?;

    Ok(())
}

/// 读取JDK版本缓存
pub fn read_jdk_versions_cache() -> Result<Option<Vec<SdkVersion>>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(JDK_VERSIONS_CACHE);

    if !cache_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&cache_file)
        .context("Failed to read JDK versions cache")?;

    let cache_data: SdkVersionsCache = serde_json::from_str(&content)
        .context("Failed to parse JDK versions cache")?;

    Ok(Some(cache_data.versions))
}

/// 缓存SDK候选者列表
pub fn cache_sdk_candidates(candidates: &[Sdk]) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(SDK_CANDIDATES_CACHE);

    let json = serde_json::to_string_pretty(candidates)
        .context("Failed to serialize SDK candidates cache")?;

    fs::write(&cache_file, json)
        .context("Failed to write SDK candidates cache")?;

    Ok(())
}

/// 读取SDK候选者缓存
pub fn read_sdk_candidates_cache() -> Result<Option<Vec<Sdk>>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(SDK_CANDIDATES_CACHE);

    if !cache_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&cache_file)
        .context("Failed to read SDK candidates cache")?;

    let candidates: Vec<Sdk> = serde_json::from_str(&content)
        .context("Failed to parse SDK candidates cache")?;

    Ok(Some(candidates))
}

/// 缓存SDK版本列表（通用，支持所有candidate）
pub fn cache_sdk_versions(candidate: &str, versions: &[SdkVersion]) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("{}_versions.json", candidate));

    let cache_data = SdkVersionsCache {
        candidate: candidate.to_string(),
        versions: versions.to_vec(),
        timestamp: get_timestamp(),
    };

    let json = serde_json::to_string_pretty(&cache_data)
        .context("Failed to serialize SDK versions cache")?;

    fs::write(&cache_file, json)
        .context(format!("Failed to write {} versions cache", candidate))?;

    Ok(())
}

/// 读取SDK版本缓存（通用，支持所有candidate）
pub fn read_sdk_versions_cache(candidate: &str) -> Result<Option<Vec<SdkVersion>>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("{}_versions.json", candidate));

    if !cache_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&cache_file)
        .context(format!("Failed to read {} versions cache", candidate))?;

    let cache_data: SdkVersionsCache = serde_json::from_str(&content)
        .context(format!("Failed to parse {} versions cache", candidate))?;

    Ok(Some(cache_data.versions))
}

/// 清除所有缓存
pub fn clear_all_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .context("Failed to remove cache directory")?;

        // 重新创建空目录
        fs::create_dir_all(&cache_dir)
            .context("Failed to recreate cache directory")?;
    }

    Ok(())
}
