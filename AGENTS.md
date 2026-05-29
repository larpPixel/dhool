# AGENTS.md

## Project

`dhool` is a Rust/CUDA offline renderer for deterministic cinematic inferno-style animated backgrounds. It is production GPU-first renderer that can produce professional stills, previews, final videos, image sequences, benchmark reports, and reproducibility metadata.

## Purpose

The renderer exists to generate high-quality animated plasma, smoke, dust, embers, bloom, atmosphere, and cinematic color grading from explicit config and seed values. CUDA is the production backend. CPU paths may exist only for tests or deterministic setup validation and must not silently replace CUDA production rendering.

## Code Style

Write small functions with one clear responsibility. Keep boundaries honest between config parsing, scene setup, CUDA backend/device management, simulation fields, render/composite kernels, export, metadata, and benchmarks.

Prefer existing modules, types, config groups, helpers, and patterns before adding new ones. Add new abstractions only when they remove real complexity or prevent meaningful duplication.

Keep artistic controls and runtime constants in config when they affect output behavior. CUDA kernel constants must be config-derived uniforms or documented compile-time limits.

Do not preserve low-quality visual tricks. Do not call fake noise fields fluid simulation. Effects that visually interact must share field motion rather than being unrelated animated layers.

Use structured data parsing and serialization instead of ad hoc string manipulation. Keep reproducibility intact: config, seed, backend, CUDA device/driver, git state, timings, and output paths must be captured for render runs.

Add comments only when they clarify non-obvious logic.

## Guidelines
Do not duplicate config structs, kernels, export paths, metadata types, or shared math. If logic is copied, extract it to the right boundary.

## Performance

Treat live preview and CLI rendering as CUDA pipeline problems first, not quality-reduction problems. At 1x 1080p preview, optimize kernels, scheduling, readback, and upload before lowering resolution, field scale, particles, bloom, or artistic controls.

CUDA production paths must be truly parallel: use bounded host worker pools, per-thread CUDA streams/caches, asynchronous copies where practical, and delayed synchronization. Do not serialize production renders through a global render mutex, the default stream, or one-frame-at-a-time host loops unless the command is intrinsically single-frame.

Avoid redundant per-pixel work in kernels. Shared field motion, terrain/ridge terms, bloom pyramids, and other reusable data should be computed once at the right resolution and sampled/coalesced by later kernels. Do not recompute expensive noise, terrain, bloom, or flow math per output pixel when a lower-dimensional or lower-resolution prepass preserves the same output contract.

CLI movie/final/preview rendering should render frames in parallel but write outputs in frame order. FFmpeg pipe writes, image sequence paths, timings, metadata, seeds, and frame indices must remain deterministic and ordered even when rendering completes out of order.

When changing CUDA or render performance, add or run a CUDA smoke/benchmark command that reports GPU, readback, total frame time, and effective fps after warmup. Record meaningful 1080p 1x results when the change is intended to affect live or CLI performance.

Tests should match risk. Add focused unit tests for config, metadata, export planning, determinism helpers, and CLI behavior. Add integration or render smoke tests when changing command behavior, export behavior, CUDA initialization, or render output contracts.

Before finishing a change, run formatting and relevant tests. For renderer or CUDA changes, also run at least one CUDA device/render command that proves the changed path still works.

## Commits

Commit regularly in small, working chunks. Each commit message should be short, direct, and describe the actual change.

Do not add co-author trailers or similar attribution footers.

Do not commit build artifacts, render outputs, caches, or generated local state.
