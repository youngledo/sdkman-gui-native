use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub jdk_installed: usize,
    pub jdk_available: usize,
    pub sdk_installed: usize,
    pub sdk_available: usize,
}
