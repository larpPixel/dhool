# AGENTS.md

## Project

`dhool` is a Rust/CUDA offline renderer for deterministic cinematic inferno-style animated backgrounds. It is production GPU-first renderer that can produce professional stills, previews, final videos, image sequences, benchmark reports, and reproducibility metadata.

## Purpose

The renderer exists to generate high-quality animated plasma, smoke, dust, embers, bloom, atmosphere, and cinematic color grading from explicit config and seed values. CUDA is the production backend. CPU paths may exist only for tests or deterministic setup validation and must not silently replace CUDA production rendering.

## Code Style

Write small functions with one clear responsibility. Keep boundaries honest between config parsing, scene setup, CUDA backend/device management, simulation fields, render/composite kernels, export, metadata, and benchmarks.

Prefer existing modules, types, config groups, helpers, and patterns before adding new ones. Add new abstractions only when they remove real complexity or prevent meaningful duplication.

Keep artistic controls and runtime constants in config when they affect output behavior. CUDA kernel constants must be config-derived uniforms or documented compile-time limits.

Do not port prototype code line by line. Do not preserve low-quality visual tricks. Do not call fake noise fields fluid simulation. Effects that visually interact must share field motion rather than being unrelated animated layers.

Use structured data parsing and serialization instead of ad hoc string manipulation. Keep reproducibility intact: config, seed, backend, CUDA device/driver, git state, timings, and output paths must be captured for render runs.

Use ASCII in source files unless the file already uses non-ASCII or there is a clear reason. Add comments only when they clarify non-obvious logic.

## Guidelines

CUDA is the production render path. Do not add OpenCL, Vulkan, wgpu, or CPU production fallbacks unless explicitly directed.

Do not make PNG the mandatory video path. Preserve direct FFmpeg pipe export and image sequence export.

Do not duplicate config structs, kernels, export paths, metadata types, or shared math. If logic is copied, extract it to the right boundary.

Tests should match risk. Add focused unit tests for config, metadata, export planning, determinism helpers, and CLI behavior. Add integration or render smoke tests when changing command behavior, export behavior, CUDA initialization, or render output contracts.

Before finishing a change, run formatting and relevant tests. For renderer or CUDA changes, also run at least one CUDA device/render command that proves the changed path still works.

## Commits

Commit regularly in small, working chunks. Each commit message should be short, direct, and describe the actual change.

Do not add co-author trailers or similar attribution footers.

Do not commit build artifacts, render outputs, caches, or generated local state.

Do not rewrite or revert user changes unless explicitly asked. If unrelated changes exist, leave them alone. If related user changes exist, work with them instead of discarding them.
