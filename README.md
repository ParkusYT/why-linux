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

The defaults check CPU >20% and memory >80% over a short sampling window.

CLI flags (simplified):

- `--duration <u64>` : Total sampling duration in seconds (default 10)
- `--interval <u64>` : Sampling interval in seconds (default 1)
- `--cpu-threshold <f32>` : CPU percent considered high (default 20.0)
- `--mem-threshold <f32>` : System memory percent considered high (default 80.0)
- `--disk-threshold <f32>` : Filesystem percent considered high (default 90.0)
- `--io-read-threshold <u64>` : Read bytes/sec considered high (default 5_000_000)
- `--io-write-threshold <u64>` : Write bytes/sec considered high (default 5_000_000)
- `-j, --json` : Output findings as pretty JSON
- `--report <path>` : Write HTML report to path

Example:

```bash
cargo run --release -- --cpu-threshold 25 --mem-threshold 75 --json
```

Example checking disk with JSON output:

```bash
cargo run --release -- --disk-threshold 85 --json
```

Example checking I/O with JSON output:

```bash
cargo run --release -- --io-read-threshold 2000000 --io-write-threshold 2000000 --json
```

Example with custom duration/interval:

```bash
cargo run --release -- --duration 20 --interval 2 --json
```