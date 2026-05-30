# Dhool Maintainability Audit

## Scope

This audit covers the incremental Rust refactor that split monolithic crate files into responsibility-based modules. The CUDA kernel source remains a known monolith in this pass because changing kernel layout has higher ABI, build, and visual-regression risk than moving Rust module boundaries.

## Completed Structural Cleanup

- `dhool-core` now separates device metadata, timings, git state, output helpers, deterministic RNG, name sanitization, and shared CUDA worker-count helpers.
- `dhool-config` now separates schema, defaults, overrides, validation, errors, and shared UI/validation constraints.
- `dhool-export` now separates output planning, PNG writing, image sequence writing, and FFmpeg piping.
- `dhool-cuda` now separates public render/device entrypoints, Rust FFI structs, parameter mapping, errors, and rendered-frame types. The FFI struct shapes and extern function names were not changed.
- `dhool` CLI now has a library entrypoint plus separate args, command handlers, effective config loading, ordered render loop, and sidecar metadata modules.
- `dhool-live` now separates app state, rendering submission/receive logic, playback, layout, preview image conversion, parameter panels, and render-worker queue management.

## Remaining Risks

### CUDA Kernel Monolith

`shaders/cuda/dhool_kernels.cu` is still the largest and riskiest file. It contains ABI structs, math helpers, field construction, terrain, HDR rendering, particles, bloom, final grading, cache/device buffer management, device info, and the exported render entrypoint. This is workable for the current pass, but it is still hard to review and risky to modify.

Recommended next split:

- `abi.cuh`: `DhoolCudaDeviceInfo`, `DhoolRenderReport`, `DhoolRenderParams`, ABI static assertions.
- `math.cuh`: vector helpers, hashing, noise, sampling helpers.
- `fields.cuh`: field flow, plasma mask, smoke layer, terrain ridge helpers.
- `kernels.cu`: CUDA kernels only.
- `cache.cuh` or `runtime.cu`: `RenderCache`, buffer/event setup, device info, exported render function.

The split should keep exported C ABI names and field order byte-for-byte compatible with `crates/dhool-cuda/src/ffi.rs`.

### Duplicated Rust/CUDA Parameter Structs

`RawRenderParams` in Rust mirrors `DhoolRenderParams` in CUDA manually. This is an ABI risk: adding, reordering, or changing a field in one language can silently break output. The current refactor intentionally keeps the duplication unchanged.

Recommended next step: add ABI size/offset checks. At minimum, add `static_assert` checks in CUDA and a Rust test that validates `size_of::<RawRenderParams>()` against an exported CUDA-reported size.

### NVCC Build Dependency

`crates/dhool-cuda/build.rs` compiles the CUDA source directly with `nvcc`. This is simple, but it means a normal workspace check depends on local CUDA tooling. Current behavior is acceptable for this GPU-first project, but CI should make that dependency explicit.

Recommended next step: document CUDA/nvcc requirements in `README.md` and add a CI job or script that reports CUDA toolkit, driver, device, and render smoke status separately from pure Rust tests.

### Reproducibility Metadata

The renderer records config, seed, backend, CUDA device info, git state, timings, and output paths. That contract should stay central when adding new commands or export modes. Parallel rendering must continue to render out of order but write frames, FFmpeg pipe input, timings, and metadata deterministically in frame order.

Recommended next step: add metadata snapshot tests around `build_metadata` with device info injection, or move metadata construction into a pure testable helper that does not query live CUDA state.

### Live Preview Behavior

The live app now has clearer module boundaries, but the UI still submits full config clones per preview request. That is acceptable for this pass. Future performance work should focus on CUDA pipeline reuse, queue cancellation, and readback scheduling before reducing quality defaults.

## Verification Expectations

For maintainability-only Rust refactors:

- Run `cargo fmt --check`.
- Run `cargo check --workspace`.
- Run focused Rust tests for changed crates.
- Run CLI tests that validate config loading and production CUDA backend rejection.

For CUDA/render-path changes:

- Also run `cargo run -p dhool -- device-info`.
- Run at least one small CUDA benchmark or render smoke and record GPU, readback, total frame time, and effective fps after warmup when the change is performance-related.
