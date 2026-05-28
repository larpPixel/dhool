# dhool

`dhool` is the Rust/CUDA production renderer for deterministic cinematic inferno backgrounds.

## Production preview

<video src="./final.mp4" controls autoplay loop muted playsinline width="100%"></video>

If your viewer doesn’t render the embed, open [`final.mp4`](./final.mp4) directly.

The first implementation gate is CUDA feasibility:

```bash
cargo run -p dhool -- device-info
cargo run -p dhool -- validate configs/inferno.production.yaml
cargo run -p dhool -- benchmark configs/inferno.production.yaml --output-dir outputs/cuda_probe --frames 1
```

Render commands:

```bash
cargo run -p dhool -- still configs/inferno.production.yaml
cargo run -p dhool -- preview configs/inferno.production.yaml --duration 3 --fps 30 --export pipe
cargo run -p dhool -- final configs/inferno.production.yaml --export both
```

Production rendering requires CUDA. The CPU reference backend is reserved for tests and deterministic setup validation; it is not a production fallback.

