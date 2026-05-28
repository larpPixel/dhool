use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let kernel = manifest_dir.join("../../shaders/cuda/dhool_kernels.cu");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let lib_path = out_dir.join("libdhoolcuda.a");

    println!("cargo:rerun-if-changed={}", kernel.display());
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");

    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".to_string());
    let status = Command::new(&nvcc)
        .arg("-std=c++17")
        .arg("-O3")
        .arg("--use_fast_math")
        .arg("--compiler-options")
        .arg("-fPIC")
        .arg("-lib")
        .arg(&kernel)
        .arg("-o")
        .arg(&lib_path)
        .status()
        .unwrap_or_else(|err| panic!("failed to launch nvcc ({nvcc}): {err}"));
    if !status.success() {
        panic!("nvcc failed while compiling {}", kernel.display());
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dhoolcuda");
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    if let Some(cuda_lib) = find_cuda_lib_dir() {
        println!("cargo:rustc-link-search=native={}", cuda_lib.display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", cuda_lib.display());
    }
}

fn find_cuda_lib_dir() -> Option<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(root) = env::var("CUDA_HOME").or_else(|_| env::var("CUDA_PATH")) {
        roots.push(PathBuf::from(root));
    }
    roots.push(PathBuf::from("/opt/cuda"));
    roots.push(PathBuf::from("/usr/local/cuda"));

    for root in roots {
        for suffix in ["lib64", "targets/x86_64-linux/lib", "lib"] {
            let candidate = root.join(suffix);
            if has_cudart(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn has_cudart(dir: &Path) -> bool {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("libcudart.so")
        })
}
