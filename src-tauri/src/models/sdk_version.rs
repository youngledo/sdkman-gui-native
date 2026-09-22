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

        // 检查是否包含JavaFX
        // SDKMAN的JavaFX标识符有两种形式：旧版 `.fx`（如21.0.9.fx-zulu）和新版 `-fx`（如27.0.0-fx+35-zulu）
        if lower.contains(".fx") || lower.contains("-fx") {
            categories.insert(JdkCategory::JavaFx);
        }

        // 检查是否为NIK（如 25.0.1-nik, 25.0.1.fx-nik）
        // 也包含 GraalVM 相关产品：
        // - Liberica NIK: 标识符以 -nik 结尾（已包含在 contains("-nik") 中）
        // - Mandrel: 标识符包含 -mandrel
        // - GraalVM: 标识符包含 -graal 或 -graalvm
        if lower.contains("-nik")
            || lower.contains("-mandrel")
            || lower.contains("-graal")
            || lower.contains("-graalvm")
        {
            categories.insert(JdkCategory::Nik);
        }

        // 如果没有特殊分类，则归为普通JDK
        if categories.is_empty() {
            categories.insert(JdkCategory::Jdk);
        }

        categories.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nik_identification() {
        // 标准 NIK
        let categories = JdkCategory::from_identifier("25.0.1-nik");
        assert!(categories.contains(&JdkCategory::Nik));

        // JavaFX + NIK
        let categories = JdkCategory::from_identifier("25.0.1.fx-nik");
        assert!(categories.contains(&JdkCategory::Nik));
        assert!(categories.contains(&JdkCategory::JavaFx));

        // Liberica NIK（应该以 -nik 结尾）
        let categories = JdkCategory::from_identifier("25.0.1-nik");
        assert!(categories.contains(&JdkCategory::Nik));

        // 普通 Liberica（不应该是 NIK）
        let categories = JdkCategory::from_identifier("25.0.1-librca");
        assert!(!categories.contains(&JdkCategory::Nik));
        assert!(categories.contains(&JdkCategory::Jdk));

        // Mandrel
        let categories = JdkCategory::from_identifier("25.0.1-mandrel");
        assert!(categories.contains(&JdkCategory::Nik));

        // GraalVM CE
        let categories = JdkCategory::from_identifier("25.0.1-graal");
        assert!(categories.contains(&JdkCategory::Nik));

        // GraalVM Oracle
        let categories = JdkCategory::from_identifier("25.0.1-graalvm");
        assert!(categories.contains(&JdkCategory::Nik));

        // 普通 JDK（不应该是 NIK）
        let categories = JdkCategory::from_identifier("25.0.1-tem");
        assert!(!categories.contains(&JdkCategory::Nik));
        assert!(categories.contains(&JdkCategory::Jdk));

        // JavaFX + Liberica（不应该是 NIK）
        let categories = JdkCategory::from_identifier("21.0.9.fx-librca");
        assert!(categories.contains(&JdkCategory::JavaFx));
        assert!(!categories.contains(&JdkCategory::Nik));

        // 新版 -fx 形式的JavaFX标识符（如 27.0.0-fx+35-zulu）
        let categories = JdkCategory::from_identifier("27.0.0-fx+35-zulu");
        assert!(categories.contains(&JdkCategory::JavaFx), "-fx标识符应识别为JavaFX分类");
        assert!(!categories.contains(&JdkCategory::Nik));

        // 新版Liberica的JavaFX标识符（如 26.0.2-fx+1.1-librca）
        let categories = JdkCategory::from_identifier("26.0.2-fx+1.1-librca");
        assert!(categories.contains(&JdkCategory::JavaFx));
        assert!(!categories.contains(&JdkCategory::Nik));

        // 新版NIK的JavaFX标识符（如 25.0.4-fx+1.1.r25-nik）
        let categories = JdkCategory::from_identifier("25.0.4-fx+1.1.r25-nik");
        assert!(categories.contains(&JdkCategory::JavaFx));
        assert!(categories.contains(&JdkCategory::Nik));

        // -crac标识符不应被误判为JavaFX（如 27.0.0-crac+35-zulu）
        let categories = JdkCategory::from_identifier("27.0.0-crac+35-zulu");
        assert!(!categories.contains(&JdkCategory::JavaFx), "-crac不应识别为JavaFX分类");
        assert!(categories.contains(&JdkCategory::Jdk));
    }

    #[test]
    fn test_empty_identifier() {
        let categories = JdkCategory::from_identifier("");
        assert_eq!(categories.len(), 1);
        assert!(categories.contains(&JdkCategory::Jdk));
    }
}
