use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, AppHandle};
use zip::ZipArchive;
use flate2::read::GzDecoder;
use tar::Archive;

/// 安装完成事件
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InstallComplete {
    pub candidate: String,
    pub version: String,
    pub path: String,
    pub success: bool,
    pub message: Option<String>,
}

/// 安装进度事件
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InstallProgress {
    pub candidate: String,
    pub version: String,
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
}

/// 卸载完成事件
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UninstallComplete {
    pub candidate: String,
    pub version: String,
    pub success: bool,
    pub message: Option<String>,
}

/// SDK 安装器
pub struct Installer;

impl Installer {
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

    /// 从归档文件安装SDK（自动检测ZIP或tar.gz）
    ///
    /// # Arguments
    /// * `archive_path` - 归档文件路径（ZIP或tar.gz）
    /// * `candidate` - SDK候选者名称
    /// * `version` - 版本号
    /// * `app` - Tauri应用句柄（用于发送进度事件）
    ///
    /// # Returns
    /// 返回安装路径
    pub fn install_from_archive(
        archive_path: &Path,
        candidate: &str,
        version: &str,
        app: AppHandle,
    ) -> Result<PathBuf> {
        println!("Installing {} {} from {:?}", candidate, version, archive_path);

        // 1. 创建目标目录
        let install_dir = Self::get_candidate_dir(candidate)?
            .join(version);

        // 如果目标目录已存在，先删除
        if install_dir.exists() {
            println!("Removing existing installation at {:?}", install_dir);
            fs::remove_dir_all(&install_dir)
                .context("Failed to remove existing installation")?;
        }

        // 创建目标目录
        fs::create_dir_all(&install_dir)
            .context("Failed to create installation directory")?;

        // 2. 根据文件扩展名选择解压方法
        let file_name = archive_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            Self::extract_tar_gz(archive_path, &install_dir, candidate, version, app.clone())?;
        } else if file_name.ends_with(".zip") {
            Self::extract_zip(archive_path, &install_dir, candidate, version, app.clone())?;
        } else {
            anyhow::bail!("Unsupported archive format: {}", file_name);
        }

        // 3. 设置权限（Unix系统）
        #[cfg(unix)]
        Self::set_executable_permissions(&install_dir)?;

        println!("Installation completed at {:?}", install_dir);

        // 4. 发送完成事件
        let complete = InstallComplete {
            candidate: candidate.to_string(),
            version: version.to_string(),
            path: install_dir.to_string_lossy().to_string(),
            success: true,
            message: Some("Installation completed successfully".to_string()),
        };

        let _ = app.emit("install-complete", &complete);

        Ok(install_dir)
    }

    /// 从ZIP文件安装SDK（保留用于兼容性）
    ///
    /// # Arguments
    /// * `zip_path` - ZIP文件路径
    /// * `candidate` - SDK候选者名称
    /// * `version` - 版本号
    /// * `app` - Tauri应用句柄（用于发送进度事件）
    ///
    /// # Returns
    /// 返回安装路径
    pub fn install_from_zip(
        zip_path: &Path,
        candidate: &str,
        version: &str,
        app: AppHandle,
    ) -> Result<PathBuf> {
        // 直接调用新的通用方法
        Self::install_from_archive(zip_path, candidate, version, app)
    }

    /// 解压tar.gz文件
    fn extract_tar_gz(
        tar_gz_path: &Path,
        target_dir: &Path,
        candidate: &str,
        version: &str,
        app: AppHandle,
    ) -> Result<()> {
        // 验证文件是否存在
        if !tar_gz_path.exists() {
            anyhow::bail!("tar.gz file does not exist: {:?}", tar_gz_path);
        }

        // 获取文件大小
        let file_size = fs::metadata(tar_gz_path)
            .context("Failed to get tar.gz file metadata")?
            .len();

        println!("Opening tar.gz file: {:?} (size: {} bytes)", tar_gz_path, file_size);

        if file_size == 0 {
            anyhow::bail!("tar.gz file is empty: {:?}", tar_gz_path);
        }

        let file = fs::File::open(tar_gz_path)
            .context(format!("Failed to open tar.gz file: {:?}", tar_gz_path))?;

        // 验证 gzip 文件头（前两个字节应该是 0x1f 0x8b）
        use std::io::Read;
        let mut file_for_check = fs::File::open(tar_gz_path)?;
        let mut magic_bytes = [0u8; 2];
        file_for_check.read_exact(&mut magic_bytes)
            .context("Failed to read file header")?;

        if magic_bytes != [0x1f, 0x8b] {
            anyhow::bail!(
                "File is not a valid gzip archive (magic bytes: {:02x} {:02x}). \
                This might be a ZIP file or corrupted download. Try redownloading.",
                magic_bytes[0], magic_bytes[1]
            );
        }

        println!("Verified gzip format (magic bytes: 1f 8b)");

        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        // 解压所有文件
        println!("Extracting tar.gz archive...");

        // tar 解压时，通常会有一个顶层目录（如 jdk-25.0.1+8）
        // 我们需要扁平化这个结构，直接解压到目标目录
        let entries = archive.entries()
            .context("Failed to read tar.gz entries. The file may be corrupted or not a valid tar archive.")?;

        let mut file_count = 0;
        for (i, entry_result) in entries.enumerate() {
            let mut entry = entry_result
                .context(format!("Failed to read tar entry at index {}. The archive may be corrupted.", i))?;

            let path = entry.path()
                .context("Failed to get entry path")?;

            // 跳过顶层目录，只提取内容
            let components: Vec<_> = path.components().collect();
            if components.len() <= 1 {
                continue; // 跳过顶层目录本身
            }

            // 移除顶层目录，构建目标路径
            let relative_path = components[1..].iter()
                .collect::<PathBuf>();
            let outpath = target_dir.join(relative_path);

            // 创建父目录
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            // 解压文件
            entry.unpack(&outpath)
                .context(format!("Failed to extract file: {:?}", outpath))?;

            file_count += 1;

            // 每100个文件发送一次进度（tar.gz没有总数，所以只报告已处理数量）
            if file_count % 100 == 0 {
                let progress = InstallProgress {
                    candidate: candidate.to_string(),
                    version: version.to_string(),
                    current: file_count,
                    total: 0, // tar.gz无法提前知道总数
                    percentage: 0.0,
                };
                let _ = app.emit("install-progress", &progress);
            }
        }

        println!("Extracted {} files", file_count);
        Ok(())
    }

    /// 解压ZIP文件
    fn extract_zip(
        zip_path: &Path,
        target_dir: &Path,
        candidate: &str,
        version: &str,
        app: AppHandle,
    ) -> Result<()> {
        // 验证ZIP文件是否存在
        if !zip_path.exists() {
            anyhow::bail!("ZIP file does not exist: {:?}", zip_path);
        }

        // 获取文件大小
        let file_size = fs::metadata(zip_path)
            .context("Failed to get ZIP file metadata")?
            .len();

        println!("Opening ZIP file: {:?} (size: {} bytes)", zip_path, file_size);

        if file_size == 0 {
            anyhow::bail!("ZIP file is empty: {:?}", zip_path);
        }

        let file = fs::File::open(zip_path)
            .context(format!("Failed to open ZIP file: {:?}", zip_path))?;

        let mut archive = ZipArchive::new(file)
            .context(format!("Failed to read ZIP archive: {:?}. The file may be corrupted or not a valid ZIP file.", zip_path))?;

        let total_files = archive.len();
        println!("Extracting {} files...", total_files);

        for i in 0..total_files {
            let mut file = archive.by_index(i)
                .context(format!("Failed to read file at index {}", i))?;

            let path = match file.enclosed_name() {
                Some(path) => path.to_path_buf(),
                None => continue,
            };

            // 跳过顶层目录，只提取内容（与tar.gz逻辑一致）
            let components: Vec<_> = path.components().collect();
            if components.len() <= 1 {
                continue; // 跳过顶层目录本身
            }

            // 移除顶层目录，构建目标路径
            let relative_path: PathBuf = components[1..].iter().collect();
            let outpath = target_dir.join(relative_path);

            if file.name().ends_with('/') {
                // 这是一个目录
                fs::create_dir_all(&outpath)
                    .context(format!("Failed to create directory: {:?}", outpath))?;
            } else {
                // 这是一个文件
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .context(format!("Failed to create parent directory: {:?}", parent))?;
                    }
                }

                let mut outfile = fs::File::create(&outpath)
                    .context(format!("Failed to create file: {:?}", outpath))?;

                std::io::copy(&mut file, &mut outfile)
                    .context(format!("Failed to write file: {:?}", outpath))?;
            }

            // 发送进度事件
            if i % 10 == 0 || i == total_files - 1 {
                let percentage = ((i + 1) as f64 / total_files as f64) * 100.0;
                let progress = InstallProgress {
                    candidate: candidate.to_string(),
                    version: version.to_string(),
                    current: i + 1,
                    total: total_files,
                    percentage,
                };

                let _ = app.emit("install-progress", &progress);

                if i % 100 == 0 || i == total_files - 1 {
                    println!("Extracted {} / {} files ({:.2}%)", i + 1, total_files, percentage);
                }
            }
        }

        Ok(())
    }

    /// 设置可执行权限（Unix系统）
    #[cfg(unix)]
    fn set_executable_permissions(install_dir: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        println!("Setting executable permissions...");

        // 遍历 bin 目录（如果存在）
        let bin_dir = install_dir.join("bin");
        if bin_dir.exists() && bin_dir.is_dir() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    // 读取当前权限
                    let metadata = fs::metadata(&path)?;
                    let mut permissions = metadata.permissions();

                    // 添加可执行权限 (755)
                    permissions.set_mode(0o755);

                    // 设置权限
                    fs::set_permissions(&path, permissions)?;

                    println!("Set executable: {:?}", path);
                }
            }
        }

        Ok(())
    }

    /// 卸载SDK
    ///
    /// # Arguments
    /// * `candidate` - SDK候选者名称
    /// * `version` - 版本号
    /// * `app` - Tauri应用句柄（用于发送完成事件）
    pub fn uninstall_sdk(candidate: &str, version: &str, app: AppHandle) -> Result<()> {
        println!("Uninstalling {} {}", candidate, version);

        // 1. 获取安装目录
        let install_dir = Self::get_candidate_dir(candidate)?
            .join(version);

        // 检查目录是否存在
        if !install_dir.exists() {
            anyhow::bail!("Version {} of {} is not installed", version, candidate);
        }

        // 2. 删除目录
        fs::remove_dir_all(&install_dir)
            .context(format!("Failed to remove installation directory: {:?}", install_dir))?;

        println!("Removed installation directory: {:?}", install_dir);

        // 3. 检查是否为当前版本
        let current_link = Self::get_candidate_dir(candidate)?
            .join("current");

        // 使用 symlink_metadata 检查符号链接本身是否存在（包括损坏的符号链接）
        if current_link.symlink_metadata().is_ok() {
            // 读取符号链接目标
            if let Ok(target) = fs::read_link(&current_link) {
                // 规范化路径进行比较
                let target_canonical = if target.is_absolute() {
                    target
                } else {
                    Self::get_candidate_dir(candidate)?.join(&target)
                };

                // 检查目标是否指向被删除的版本，或者目标已不存在
                let should_remove = target_canonical == install_dir || !target_canonical.exists();

                if should_remove {
                    println!("Removing 'current' symlink (target: {:?}, install_dir: {:?}, exists: {})",
                        target_canonical, install_dir, target_canonical.exists());
                    fs::remove_file(&current_link)
                        .context("Failed to remove 'current' symlink")?;
                }
            }
        }

        println!("Uninstallation completed");

        // 4. 发送卸载完成事件
        let complete = UninstallComplete {
            candidate: candidate.to_string(),
            version: version.to_string(),
            success: true,
            message: Some("Uninstallation completed successfully".to_string()),
        };

        let _ = app.emit("uninstall-complete", &complete);

        Ok(())
    }

    /// 验证安装
    ///
    /// # Arguments
    /// * `candidate` - SDK候选者名称
    /// * `version` - 版本号
    ///
    /// # Returns
    /// 如果安装有效返回 true，否则返回 false
    pub fn verify_installation(candidate: &str, version: &str) -> Result<bool> {
        let install_dir = Self::get_candidate_dir(candidate)?
            .join(version);

        // 检查目录是否存在
        if !install_dir.exists() || !install_dir.is_dir() {
            return Ok(false);
        }

        // 检查目录是否为空
        let entries = fs::read_dir(&install_dir)?;
        let has_files = entries.count() > 0;

        Ok(has_files)
    }
}
