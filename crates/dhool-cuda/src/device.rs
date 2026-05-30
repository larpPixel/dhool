use std::mem::MaybeUninit;

use dhool_core::DeviceInfo;

use crate::ffi::{RawDeviceInfo, c_string, cuda_version, dhool_cuda_device_info};

pub fn device_info() -> DeviceInfo {
    let mut raw = MaybeUninit::<RawDeviceInfo>::zeroed();
    let status = unsafe { dhool_cuda_device_info(raw.as_mut_ptr()) };
    let raw = unsafe { raw.assume_init() };
    let error = c_string(&raw.error_message);
    DeviceInfo {
        backend: "cuda".to_string(),
        available: status != 0 && raw.available != 0,
        name: c_string(&raw.name),
        driver_version: cuda_version(raw.driver_version),
        runtime_version: cuda_version(raw.runtime_version),
        global_memory_bytes: raw.global_memory_bytes,
        multiprocessor_count: raw.multiprocessor_count,
        error: (!error.is_empty()).then_some(error),
    }
}
