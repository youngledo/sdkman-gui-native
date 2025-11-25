use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// SDK版本信息（对应 JavaFX 的 SdkVersion.java）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkVersion {
    /// 版本号
    pub version: String,

    /// 唯一标识符（如 21.0.9-amzn）
    pub identifier: String,

    /// 供应商
    pub vendor: String,

    /// JDK分类集合 - 使用Vec以便正确序列化为JSON数组
    #[serde(default)]
    pub categories: Vec<JdkCategory>,

    /// SDK候选名称（如 java、gradle、maven）
    pub candidate: String,

    /// 是否已安装
    pub installed: bool,

    /// 是否为默认版本
    #[serde(rename = "isDefault")]
    pub is_default: bool,

    /// 是否正在使用
    #[serde(rename = "inUse")]
    pub in_use: bool,

    /// 是否正在安装（前端状态）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installing: Option<bool>,

    /// 安装进度文本（前端状态）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_progress: Option<String>,
}

/// JDK分类枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JdkCategory {
    /// 普通JDK
    #[serde(rename = "JDK")]
    Jdk,
    /// 带JavaFX支持的JDK
    #[serde(rename = "JAVAFX")]
    JavaFx,
    /// Native Image Kit (NIK)
    #[serde(rename = "NIK")]
    Nik,
}

impl JdkCategory {
    /// 从标识符推断分类（对应 JavaFX 的 JdkCategory.fromIdentifier）
    /// 一个JDK可能同时属于多个分类(例如: 既支持JavaFX又是NIK)
    pub fn from_identifier(identifier: &str) -> Vec<JdkCategory> {
        let mut categories = HashSet::new();

        if identifier.is_empty() {
            categories.insert(JdkCategory::Jdk);
            return categories.into_iter().collect();
        }

        let lower = identifier.to_lowercase();

        // 检查是否包含JavaFX（如 21.0.9.fx-librca, 25.0.1.fx-nik）
        if lower.contains(".fx") {
            categories.insert(JdkCategory::JavaFx);
        }

        // 检查是否为NIK（如 25.0.1-nik, 25.0.1.fx-nik）
        if lower.contains("-nik") {
            categories.insert(JdkCategory::Nik);
        }

        // 如果没有特殊分类，则归为普通JDK
        if categories.is_empty() {
            categories.insert(JdkCategory::Jdk);
        }

        categories.into_iter().collect()
    }
}
