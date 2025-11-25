use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

/// 符号链接管理器
pub struct SymlinkManager;

impl SymlinkManager {
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

    /// 设置默认版本（创建符号链接）
    ///
    /// # Arguments
    /// * `candidate` - SDK候选者名称
    /// * `version` - 版本号
    ///
    /// # Returns
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn set_default_version(candidate: &str, version: &str) -> Result<()> {
        println!("Setting default version for {} to {}", candidate, version);

        let candidates_dir = Self::get_candidate_dir(candidate)?;
        let current_link = candidates_dir.join("current");
        let target = candidates_dir.join(version);

        // 检查目标版本是否存在
        if !target.exists() {
            anyhow::bail!("Version {} of {} is not installed", version, candidate);
        }

        // 删除旧的符号链接（如果存在）
        // 注意：使用 symlink_metadata 而不是 exists，因为 exists 会检查目标是否存在
        // 对于损坏的符号链接，exists() 会返回 false，但我们仍然需要删除它
        if current_link.symlink_metadata().is_ok() {
            println!("Removing existing 'current' symlink");
            fs::remove_file(&current_link)
                .context("Failed to remove existing 'current' symlink")?;
        }

        // 创建新的符号链接
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &current_link)
                .context(format!("Failed to create symlink from {:?} to {:?}", current_link, target))?;
        }

        #[cfg(windows)]
        {
            // Windows需要使用不同的方法
            if target.is_dir() {
                std::os::windows::fs::symlink_dir(&target, &current_link)
                    .context(format!("Failed to create directory symlink from {:?} to {:?}", current_link, target))?;
            } else {
                std::os::windows::fs::symlink_file(&target, &current_link)
                    .context(format!("Failed to create file symlink from {:?} to {:?}", current_link, target))?;
            }
        }

        println!("Successfully set default version: {:?} -> {:?}", current_link, target);

        Ok(())
    }

    /// 取消默认版本设置（删除符号链接）
    ///
    /// # Arguments
    /// * `candidate` - SDK候选者名称
    pub fn unset_default_version(candidate: &str) -> Result<()> {
        println!("Unsetting default version for {}", candidate);

        let candidates_dir = Self::get_candidate_dir(candidate)?;
        let current_link = candidates_dir.join("current");

        // 如果符号链接不存在，直接返回成功
        if !current_link.exists() {
            println!("No default version set for {}", candidate);
            return Ok(());
        }

        // 删除符号链接
        fs::remove_file(&current_link)
            .context("Failed to remove 'current' symlink")?;

        println!("Successfully unset default version for {}", candidate);

        Ok(())
    }

    /// 获取符号链接目标
    ///
    /// # Arguments
    /// * `candidate` - SDK候选者名称
    ///
    /// # Returns
    /// 返回符号链接指向的版本号，如果不存在返回 None
    pub fn get_symlink_target(candidate: &str) -> Result<Option<String>> {
        let candidates_dir = Self::get_candidate_dir(candidate)?;
        let current_link = candidates_dir.join("current");

        // 如果符号链接不存在，返回 None
        if !current_link.exists() {
            return Ok(None);
        }

        // 读取符号链接目标
        let target = fs::read_link(&current_link)
            .context("Failed to read 'current' symlink")?;

        // 获取版本号（路径的最后一部分）
        if let Some(version_name) = target.file_name() {
            let version = version_name.to_string_lossy().to_string();
            Ok(Some(version))
        } else {
            Ok(None)
        }
    }

    /// 验证符号链接是否有效
    ///
    /// # Arguments
    /// * `candidate` - SDK候选者名称
    ///
    /// # Returns
    /// 如果符号链接存在且指向有效的安装目录，返回 true
    pub fn is_symlink_valid(candidate: &str) -> Result<bool> {
        let candidates_dir = Self::get_candidate_dir(candidate)?;
        let current_link = candidates_dir.join("current");

        // 检查符号链接是否存在
        if !current_link.exists() {
            return Ok(false);
        }

        // 读取符号链接目标
        let target = fs::read_link(&current_link)?;

        // 检查目标是否存在且是目录
        Ok(target.exists() && target.is_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_symlink_target() {
        // 测试Java的当前版本
        let result = SymlinkManager::get_symlink_target("java");
        assert!(result.is_ok());
        // 如果安装了Java，应该有一个当前版本
    }

    #[test]
    fn test_is_symlink_valid() {
        let result = SymlinkManager::is_symlink_valid("java");
        assert!(result.is_ok());
    }
}
