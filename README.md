# why-linux

`why-linux` is a tiny CLI tool to help spot sustained system resource problems on Linux.

Features added:
- Detect sustained high CPU usage and show the top offending process.
- Detect sustained high memory usage and show the top memory-consuming process.

Quick run:

```bash
cd why-linux
cargo run --release
```

The defaults check CPU >20% and memory >80% over short sample windows; tweak thresholds in `src/main.rs` as needed.