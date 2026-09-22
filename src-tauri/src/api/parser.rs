use crate::models::{Sdk, SdkVersion, JdkCategory};
use anyhow::Result;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// 正则表达式：解析候选列列表
    /// 格式: ---\n候选名称(版本)  网址\n\n描述...\n$ sdk install candidate_id\n
    static ref CANDIDATE_PATTERN: Regex = Regex::new(
        r"(?s)---\r*\n(.+?)\r*\n\r*\n(.*?)\$ sdk install(.*?)\r*\n"
    ).unwrap();

    /// 正则表达式：检测Java版本表格（表头以"Vendor |"开头）
    /// SDKMAN API返回过两种列格式，解析时需同时兼容：
    /// - 4列（现行）：Vendor | Use | Version | Identifier
    /// - 6列（旧版）：Vendor | Use | Version | Dist | Status | Identifier
    static ref JAVA_TABLE_HEADER_PATTERN: Regex = Regex::new(
        r"(?m)^\s*Vendor\s*\|"
    ).unwrap();
}

const VENDOR_HEADER_NAME: &str = "Vendor";

pub struct ResponseParser;

impl ResponseParser {
    /// 解析候选列表响应（表格格式）
    /// 使用正则表达式解析，参考 sdkman-ui 的实现
    /// 格式示例：
    /// ```
    /// --------------------------------------------------------------------------------
    /// Apache ActiveMQ (Classic) (5.17.1)                  https://activemq.apache.org/
    /// Apache ActiveMQ® is a popular open source...
    ///                                                   $ sdk install activemq
    /// --------------------------------------------------------------------------------
    /// ```
    pub fn parse_candidates(response: &str) -> Result<Vec<Sdk>> {
        let mut sdks = Vec::new();

        if response.trim().is_empty() {
            return Ok(sdks);
        }

        for cap in CANDIDATE_PATTERN.captures_iter(response) {
            // group 1: 候选名称（包含版本和网址的第一行）
            // group 2: 描述文本
            // group 3: candidate ID（安装命令中的标识符）
            let mut first_line = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let description = cap.get(2)
                .map(|m| m.as_str().trim().replace('\n', " "))
                .unwrap_or_default();
            let candidate_id = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("");

            let mut sdk = Sdk {
                candidate: candidate_id.to_string(),
                name: String::new(),
                description: description.clone(),
                website: None,
                latest_version: None,
                installed_version: None,
                installed: false,
                category: crate::models::Category::from_name(candidate_id, &description),
            };

            // 提取网址（最后一个 http 开头的部分）
            if let Some(http_index) = first_line.rfind("http") {
                sdk.website = Some(first_line[http_index..].trim().to_string());
                first_line = first_line[..http_index].trim();
            }

            // 提取版本（最后一对括号中的内容）
            if let Some(last_open_paren) = first_line.rfind('(') {
                if let Some(last_close_paren) = first_line.rfind(')') {
                    if last_close_paren > last_open_paren {
                        sdk.latest_version = Some(
                            first_line[last_open_paren + 1..last_close_paren]
                                .trim()
                                .to_string()
                        );
                        first_line = first_line[..last_open_paren].trim();
                    }
                }
            }

            // 剩下的就是名称
            sdk.name = first_line.to_string();

            sdks.push(sdk);
        }

        Ok(sdks)
    }

    /// 解析版本列表响应（表格格式）
    /// 使用正则表达式解析，参考 sdkman-ui 的实现
    pub fn parse_versions(response: &str, candidate: &str) -> Result<Vec<SdkVersion>> {
        if response.trim().is_empty() {
            return Ok(vec![]);
        }

        // 检测是Java格式还是其他格式（Java格式表头以"Vendor |"开头）
        if JAVA_TABLE_HEADER_PATTERN.is_match(response) {
            // Java格式：带|分隔符的表格
            Self::parse_java_versions(response, candidate)
        } else {
            // 其他SDK格式：空格分隔
            Self::parse_other_versions(response, candidate)
        }
    }

    /// 解析Java版本（表格格式，|分隔）
    /// 兼容两种列格式：
    /// - 4列（现行）：Vendor | Use | Version | Identifier
    /// - 6列（旧版）：Vendor | Use | Version | Dist | Status | Identifier
    ///
    /// Use列标记：`>` 使用中、`*` 已安装、`+` 仅本地（已安装但远端已下架）
    fn parse_java_versions(response: &str, candidate: &str) -> Result<Vec<SdkVersion>> {
        let mut versions = Vec::new();
        let mut last_vendor: Option<String> = None;

        for line in response.lines() {
            // 只处理带|分隔符的数据行，分隔线/标题/图例行自然被跳过
            let raw_cols: Vec<&str> = line.split('|').collect();
            if raw_cols.len() < 4 {
                continue;
            }

            let cols: Vec<String> = raw_cols.iter().map(|c| c.trim().to_string()).collect();

            let vendor_col = &cols[0];
            let use_col = &cols[1];
            let version_col = &cols[2];
            let identifier_col = cols.last().unwrap();
            // 6列旧格式中Status位于第5列，4列格式状态全在Use列
            let status_col: &str = if cols.len() >= 6 { cols[4].as_str() } else { "" };

            // 跳过表头
            if vendor_col == VENDOR_HEADER_NAME {
                continue;
            }

            // 处理vendor（可能为空，使用上一行的vendor）
            let vendor = if !vendor_col.is_empty() {
                last_vendor = Some(vendor_col.clone());
                vendor_col.clone()
            } else if let Some(ref lv) = last_vendor {
                lv.clone()
            } else {
                String::new()
            };

            // identifier 必须非空
            if identifier_col.is_empty() {
                continue;
            }

            // 解析状态：Use列中 > 使用中、* 已安装、+ 仅本地；旧6列格式额外检查Status列
            let is_in_use = use_col.contains('>');
            let is_installed = is_in_use
                || use_col.contains('*')
                || use_col.contains('+')
                || status_col.contains("installed");

            let sdk_version = SdkVersion {
                version: version_col.clone(),
                identifier: identifier_col.clone(),
                vendor,
                categories: JdkCategory::from_identifier(identifier_col),
                candidate: candidate.to_string(),
                installed: is_installed,
                is_default: is_in_use,
                in_use: is_in_use,
                installing: None,
                install_progress: None,
            };

            versions.push(sdk_version);
        }

        Ok(versions)
    }

    /// 解析其他SDK版本（空格分隔格式）
    /// 参考 sdkman-ui 的实现逻辑
    fn parse_other_versions(response: &str, candidate: &str) -> Result<Vec<SdkVersion>> {
        let mut versions = Vec::new();
        let lines: Vec<&str> = response.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // 跳过表头（前3行）
            if line_num < 3 {
                continue;
            }

            // 跳过分隔符行
            if line.trim().starts_with("=====") {
                break;
            }

            // 跳过空行
            if line.trim().is_empty() {
                continue;
            }

            // 每行开头的5个字符是标记区域，格式如: " > * " 或 "   * " 或 "     "
            let marker_area = if line.len() >= 5 {
                &line[0..5]
            } else {
                line
            };

            let is_in_use = marker_area.contains('>');
            let is_installed = marker_area.contains('*');

            // 提取版本区域（从第5个字符开始），按空格分割
            let versions_area = if line.len() > 5 { &line[5..] } else { "" };

            // 按多个空格分割，提取所有版本��
            let version_parts: Vec<&str> = versions_area
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .collect();

            // 只有第一个版本带标记，其他版本不带标记
            for (idx, version_text) in version_parts.iter().enumerate() {
                let is_first = idx == 0;

                let sdk_version = SdkVersion {
                    version: version_text.to_string(),
                    identifier: version_text.to_string(),
                    vendor: String::new(),
                    categories: Vec::new(),
                    candidate: candidate.to_string(),
                    installed: if is_first { is_installed || is_in_use } else { false },
                    is_default: if is_first { is_in_use } else { false },
                    in_use: if is_first { is_in_use } else { false },
                    installing: None,
                    install_progress: None,
                };

                versions.push(sdk_version);
            }
        }

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_FOUR_COLUMN_RESPONSE: &str = "\
================================================================================
Available Java Versions for macOS ARM 64bit
================================================================================
 Vendor         | Use | Version            | Identifier
--------------------------------------------------------------------------------
 Corretto       |     | 27.0.0             | 27.0.0-amzn
                |     | 26.0.2             | 26.0.2-amzn
 GraalVM CE     | > * | 25.3.4+1.r25       | 25.3.4+1.r25-graalce
 Temurin        |   + | 26.0.1             | 26.0.1-tem
 Zulu           |   * | 27.0.0-fx+35       | 27.0.0-fx+35-zulu
================================================================================
 > in use   * installed   + local only
--------------------------------------------------------------------------------
 $ sdk install java <Identifier>    install a specific version
================================================================================
";

    #[test]
    fn test_parse_current_four_column_java_format() {
        let versions =
            ResponseParser::parse_versions(CURRENT_FOUR_COLUMN_RESPONSE, "java").unwrap();

        assert_eq!(versions.len(), 5);

        // 第一行：带vendor的普通版本
        let corretto = &versions[0];
        assert_eq!(corretto.vendor, "Corretto");
        assert_eq!(corretto.identifier, "27.0.0-amzn");
        assert!(!corretto.installed);
        assert!(!corretto.in_use);

        // 第二行：vendor为空，应继承上一行的vendor
        assert_eq!(versions[1].vendor, "Corretto", "空vendor应继承上一行的vendor");
        assert_eq!(versions[1].identifier, "26.0.2-amzn");

        // > * 使用中且已安装，graalce标识符应识别为NIK分类
        let graalce = &versions[2];
        assert_eq!(graalce.vendor, "GraalVM CE");
        assert!(graalce.in_use);
        assert!(graalce.installed);
        assert!(graalce.categories.contains(&JdkCategory::Nik));

        // + 仅本地，也应视为已安装
        assert!(versions[3].installed, "+ local only应视为已安装");
        assert!(!versions[3].in_use);

        // * 已安装，-fx标识符应识别为JavaFX分类
        assert!(versions[4].installed);
        assert!(versions[4].categories.contains(&JdkCategory::JavaFx));
    }

    #[test]
    fn test_parse_legacy_six_column_java_format() {
        let response = "\
================================================================================
 Vendor      | Use | Version | Dist | Status   | Identifier
--------------------------------------------------------------------------------
 Temurin     |     | 21.0.2  | tem  |          | 21.0.2-tem
             | > * | 17.0.9  | tem  | installed | 17.0.9-tem
 Amazon      |     | 21.0.3  | amzn | installed | 21.0.3-amzn
================================================================================
";
        let versions = ResponseParser::parse_versions(response, "java").unwrap();

        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].vendor, "Temurin");
        assert!(!versions[0].installed);

        assert!(versions[1].in_use);
        assert!(versions[1].installed, "旧格式Status列的installed标记应生效");

        assert_eq!(versions[2].vendor, "Amazon");
        assert!(versions[2].installed);
        assert!(!versions[2].in_use);
    }

    #[test]
    fn test_parse_other_sdk_format() {
        // 空格分隔格式（无|分隔符）不应被误判为Java格式
        let response = "\
================================================================================
Available Maven Versions for macOS ARM 64bit
================================================================================
 > * 3.9.9              3.9.8              3.9.7
     3.9.6              3.9.5
================================================================================
";
        let versions = ResponseParser::parse_versions(response, "maven").unwrap();

        assert_eq!(versions.len(), 5);
        assert_eq!(versions[0].version, "3.9.9");
        assert!(versions[0].in_use);
        assert!(versions[0].installed);

        // 后续版本不应继承行首标记
        assert!(!versions[1].installed);
        assert!(!versions[1].in_use);
    }

    #[test]
    fn test_parse_empty_response() {
        assert!(ResponseParser::parse_versions("", "java").unwrap().is_empty());
        assert!(ResponseParser::parse_versions("   ", "java").unwrap().is_empty());
    }

    /// 基于真实API完整响应（2026-09抓取）的回归测试
    /// installed=21.0.10-tem,26.0.1-tem,25.3.4+1.r25-graalce,27.0.0-fx+35-zulu&current=25.3.4+1.r25-graalce
    #[test]
    fn test_parse_real_api_response() {
        let response = include_str!("testdata/java-versions-api-response.txt");
        let versions = ResponseParser::parse_versions(response, "java").unwrap();

        assert_eq!(versions.len(), 89, "数据行总数");

        // 15个供应商，且所有版本的vendor都应非空
        let vendors: std::collections::HashSet<&str> =
            versions.iter().map(|v| v.vendor.as_str()).collect();
        assert_eq!(vendors.len(), 15, "供应商数量");
        assert!(versions.iter().all(|v| !v.vendor.is_empty()));

        let installed: std::collections::HashSet<&str> = versions
            .iter()
            .filter(|v| v.installed)
            .map(|v| v.identifier.as_str())
            .collect();
        assert_eq!(
            installed,
            ["21.0.10-tem", "26.0.1-tem", "25.3.4+1.r25-graalce", "27.0.0-fx+35-zulu"]
                .into_iter()
                .collect()
        );

        let in_use: Vec<&str> = versions
            .iter()
            .filter(|v| v.in_use)
            .map(|v| v.identifier.as_str())
            .collect();
        assert_eq!(in_use, vec!["25.3.4+1.r25-graalce"]);

        assert_eq!(
            versions.iter().filter(|v| v.categories.contains(&JdkCategory::JavaFx)).count(),
            16,
            "JavaFX分类数量（含.fx与-fx两种标识符形式）"
        );
        assert_eq!(
            versions.iter().filter(|v| v.categories.contains(&JdkCategory::Nik)).count(),
            11,
            "NIK分类数量"
        );

        assert!(
            versions.iter().all(|v| !v.version.contains('|')),
            "不应有包含|的垃圾条目"
        );
    }
}
