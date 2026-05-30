use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub backend: String,
    pub available: bool,
    pub name: String,
    pub driver_version: String,
    pub runtime_version: String,
    pub global_memory_bytes: u64,
    pub multiprocessor_count: i32,
    pub error: Option<String>,
}
