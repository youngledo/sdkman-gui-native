/// SDKMAN API 端点
pub struct ApiEndpoints;

impl ApiEndpoints {
    const BASE_URL: &'static str = "https://api.sdkman.io/2";

    /// 获取候选列表: GET /candidates/list
    pub fn candidates_list() -> String {
        format!("{}/candidates/list", Self::BASE_URL)
    }

    /// 获取版本列表: GET /candidates/{candidate}/{platform}/versions/list
    pub fn versions_list(candidate: &str, platform: &str, installed: &str, current: &str) -> String {
        format!(
            "{}/candidates/{}/{}/versions/list?installed={}&current={}",
            Self::BASE_URL, candidate, platform, installed, current
        )
    }

    /// 下载SDK: GET /broker/download/{candidate}/{version}/{platform}
    pub fn download(candidate: &str, version: &str, platform: &str) -> String {
        format!(
            "{}/broker/download/{}/{}/{}",
            Self::BASE_URL, candidate, version, platform
        )
    }
}
