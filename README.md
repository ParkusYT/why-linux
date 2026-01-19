# why-linux

`why-linux` is a small Linux CLI to spot sustained resource pressure and show the top offenders.

## Quick start

```bash
cd why-linux
cargo run --release
```

## Common usage

```bash
# sample for 10s at 1s intervals (default)
cargo run --release --

# custom duration + interval
cargo run --release -- --duration 20 --interval 2

# JSON output
cargo run --release -- --json

# HTML report
cargo run --release -- --report /tmp/why-linux-report.html
```

## Flags

- `--duration <u64>`: total sampling duration in seconds (default 10)
- `--interval <u64>`: sampling interval in seconds (default 1)
- `--cpu-threshold <f32>`: CPU percent considered high (default 20.0)
- `--mem-threshold <f32>`: system memory percent considered high (default 80.0)
- `--disk-threshold <f32>`: filesystem percent considered high (default 90.0)
- `--io-read-threshold <u64>`: read bytes/sec considered high (default 5_000_000)
- `--io-write-threshold <u64>`: write bytes/sec considered high (default 5_000_000)
- `-j, --json`: print machine-readable JSON
- `--report <path>`: write a self-contained HTML report