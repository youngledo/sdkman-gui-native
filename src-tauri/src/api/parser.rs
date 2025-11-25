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

    /// 正则表达式：解析Java版本表格（带|分隔符）
    static ref JAVA_VERSION_PATTERN: Regex = Regex::new(
        r"(.*?)\|(.*?)\|(.*?)\|(.*?)\|(.*?)\|(.*)"
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

        // 检测是Java格式还是其他格式
        if JAVA_VERSION_PATTERN.is_match(response) && response.contains(VENDOR_HEADER_NAME) {
            // Java格式：带|分隔符的表格
            Self::parse_java_versions(response, candidate)
        } else {
            // 其他SDK格式：空格分隔
            Self::parse_other_versions(response, candidate)
        }
    }

    /// 解析Java版本（表格格式，|分隔）
    /// 格式: Vendor | Use | Version | Dist | Status | Identifier
    fn parse_java_versions(response: &str, candidate: &str) -> Result<Vec<SdkVersion>> {
        let mut versions = Vec::new();
        let mut last_vendor: Option<String> = None;

        for cap in JAVA_VERSION_PATTERN.captures_iter(response) {
            let vendor_col = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let use_col = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let version_col = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("");
            let status_col = cap.get(5).map(|m| m.as_str().trim()).unwrap_or("");
            let identifier_col = cap.get(6).map(|m| m.as_str().trim()).unwrap_or("");

            // 跳过表头
            if vendor_col == VENDOR_HEADER_NAME {
                continue;
            }

            // 处理vendor（可能为空，使用上一行的vendor）
            let vendor = if !vendor_col.is_empty() {
                last_vendor = Some(vendor_col.to_string());
                vendor_col.to_string()
            } else if let Some(ref lv) = last_vendor {
                lv.clone()
            } else {
                String::new()
            };

            // identifier 必须非��
            if identifier_col.is_empty() {
                continue;
            }

            // 解析状态
            let is_installed = status_col.contains("installed") || use_col.contains('*');
            let is_in_use = use_col.contains('>');

            let sdk_version = SdkVersion {
                version: version_col.to_string(),
                identifier: identifier_col.to_string(),
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
