# dhool

`dhool` is the Rust/CUDA production renderer for deterministic cinematic inferno backgrounds.

## Preview
<video src="./final.mp4" controls autoplay loop muted playsinline width="100%"></video>

## How to Use

Render commands:

```bash
cargo run -p dhool -- still configs/inferno.production.yaml

cargo run -p dhool -- preview configs/inferno.production.yaml --duration 3 --fps 30 --export pipe

cargo run -p dhool -- final configs/inferno.production.yaml --export both
```

Production rendering requires CUDA. The CPU reference backend is reserved for tests and deterministic setup validation; it is not a production fallback.

