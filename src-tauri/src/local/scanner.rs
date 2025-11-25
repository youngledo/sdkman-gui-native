use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

/// 本地 SDK 扫描器
pub struct LocalScanner;

impl LocalScanner {
    /// 获取 SDKMAN 根目录路径
    fn get_sdkman_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;
        Ok(home.join(".sdkman"))
    }

    /// 获取候选者目录路径
    fn get_candidate_dir(candidate: &str) -> Result<PathBuf> {
        let sdkman_dir = Self::get_sdkman_dir()?;
        Ok(sdkman_dir.join("candidates").join(candidate))
    }

    /// 扫描指定 candidate 的已安装版本
    ///
    /// # Arguments
    /// * `candidate` - SDK 候选者名称（如 "java", "maven"）
    ///
    /// # Returns
    /// 返回已安装版本的列表（不包括 "current" 符号链接）
    pub fn scan_installed_versions(candidate: &str) -> Result<Vec<String>> {
        let candidate_dir = Self::get_candidate_dir(candidate)?;

        // 如果目录不存在，返回空列表
        if !candidate_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();

        // 读取目录中的所有条目
        let entries = fs::read_dir(&candidate_dir)
            .context(format!("Failed to read directory: {:?}", candidate_dir))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 只处理目录（排除文件）
            if !path.is_dir() {
                continue;
            }

            // 获取目录名
            if let Some(version_name) = path.file_name() {
                let version_str = version_name.to_string_lossy().to_string();

                // 排除 "current" 符号链接
                if version_str != "current" {
                    versions.push(version_str);
                }
            }
        }

        // 对版本进行排序
        versions.sort();

        Ok(versions)
    }

    /// 获取当前默认版本
    ///
    /// # Arguments
    /// * `candidate` - SDK 候选者名称（如 "java", "maven"）
    ///
    /// # Returns
    /// 返回当前默认版本，如果未设置则返回 None
    pub fn get_current_version(candidate: &str) -> Result<Option<String>> {
        let candidate_dir = Self::get_candidate_dir(candidate)?;
        let current_link = candidate_dir.join("current");

        // 如果 current 符号链接不存在，返回 None
        if !current_link.exists() {
            return Ok(None);
        }

        // 读取符号链接的目标
        let target = fs::read_link(&current_link)
            .context(format!("Failed to read symlink: {:?}", current_link))?;

        // 获取目标路径的最后一个部分（版本号）
        if let Some(version_name) = target.file_name() {
            let version = version_name.to_string_lossy().to_string();
            Ok(Some(version))
        } else {
            Ok(None)
        }
    }

    /// 检查指定版本是否已安装
    ///
    /// # Arguments
    /// * `candidate` - SDK 候选者名称
    /// * `version` - 版本号
    ///
    /// # Returns
    /// 如果版本已安装返回 true，否则返回 false
    pub fn is_version_installed(candidate: &str, version: &str) -> Result<bool> {
        let candidate_dir = Self::get_candidate_dir(candidate)?;
        let version_dir = candidate_dir.join(version);
        Ok(version_dir.exists() && version_dir.is_dir())
    }

    /// 获取所有已安装的候选者列表
    ///
    /// # Returns
    /// 返回已安装候选者的列表（至少有一个版本安装的候选者）
    pub fn list_installed_candidates() -> Result<Vec<String>> {
        let sdkman_dir = Self::get_sdkman_dir()?;
        let candidates_dir = sdkman_dir.join("candidates");

        // 如果 candidates 目录不存在，返回空列表
        if !candidates_dir.exists() {
            return Ok(Vec::new());
        }

        let mut candidates = Vec::new();

        // 读取所有候选者目录
        let entries = fs::read_dir(&candidates_dir)
            .context(format!("Failed to read directory: {:?}", candidates_dir))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 只处理目录
            if !path.is_dir() {
                continue;
            }

            if let Some(candidate_name) = path.file_name() {
                let candidate_str = candidate_name.to_string_lossy().to_string();

                // 检查是否有安装的版本
                let installed_versions = Self::scan_installed_versions(&candidate_str)?;
                if !installed_versions.is_empty() {
                    candidates.push(candidate_str);
                }
            }
        }

        // 排序
        candidates.sort();

        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sdkman_dir() {
        let result = LocalScanner::get_sdkman_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".sdkman"));
    }

    #[test]
    fn test_scan_installed_versions() {
        // 这个测试需要实际的 SDKMAN 安装
        // 如果没有安装，会返回空列表
        let result = LocalScanner::scan_installed_versions("java");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_current_version() {
        let result = LocalScanner::get_current_version("java");
        assert!(result.is_ok());
    }
}
