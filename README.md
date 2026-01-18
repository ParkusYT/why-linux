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

CLI flags:

- `--cpu-threshold <f32>` : CPU percent considered high (default 20.0)
- `--cpu-samples <usize>` : Number of 1s samples to collect for CPU (default 5)
- `--cpu-min-hits <usize>` : Minimum high CPU hits to consider sustained (default 3)
- `--mem-threshold <f32>` : System memory percent considered high (default 80.0)
- `--mem-samples <usize>` : Number of 1s samples to collect for memory (default 5)
- `--mem-min-hits <usize>` : Minimum high memory hits to consider sustained (default 2)
- `-j, --json` : Output findings as pretty JSON

Disk flags:

- `--disk-threshold <f32>` : Filesystem percent considered high (default 90.0)
- `--disk-samples <usize>` : Number of 1s samples to collect for disk (default 5)
- `--disk-min-hits <usize>` : Minimum high disk hits to consider sustained (default 2)

Example:

```bash
cargo run --release -- --cpu-threshold 25 --mem-threshold 75 --json
```

Example checking disk with JSON output:

```bash
cargo run --release -- --disk-threshold 85 --json
```

I/O flags:

- `--io-read-threshold <u64>` : Read bytes/sec considered high (default 5_000_000)
- `--io-write-threshold <u64>` : Write bytes/sec considered high (default 5_000_000)
- `--io-samples <usize>` : Number of 1s samples to collect for I/O (default 5)
- `--io-min-hits <usize>` : Minimum high I/O hits to consider sustained (default 2)

Example checking I/O with JSON output:

```bash
cargo run --release -- --io-read-threshold 2000000 --io-write-threshold 2000000 --json
```

CLI flags:

- `--cpu-threshold <f32>` : CPU percent considered high (default 20.0)
- `--cpu-samples <usize>` : Number of 1s samples to collect for CPU (default 5)
- `--cpu-min-hits <usize>` : Minimum high CPU hits to consider sustained (default 3)
- `--mem-threshold <f32>` : System memory percent considered high (default 80.0)
- `--mem-samples <usize>` : Number of 1s samples to collect for memory (default 5)
- `--mem-min-hits <usize>` : Minimum high memory hits to consider sustained (default 2)
- `-j, --json` : Output findings as pretty JSON

Example:

```bash
cargo run --release -- --cpu-threshold 25 --mem-threshold 75 --json
```