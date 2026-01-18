mod cpu;
mod explain;
mod mem;
mod disk;

use clap::Parser;
use serde_json::json;

use cpu::detect_sustained_high_cpu;
use explain::explain_process;
use mem::detect_sustained_high_mem;

#[derive(Parser, Debug)]
#[command(author, version, about = "Monitor sustained CPU and memory usage")]
struct Args {
    /// CPU threshold percentage to consider high
    #[arg(long, default_value_t = 20.0)]
    cpu_threshold: f32,

    /// Number of samples (seconds) to collect for CPU
    #[arg(long, default_value_t = 5)]
    cpu_samples: usize,

    /// Minimum high CPU hits to consider sustained
    #[arg(long, default_value_t = 3)]
    cpu_min_hits: usize,

    /// System memory used percent threshold
    #[arg(long, default_value_t = 80.0)]
    mem_threshold: f32,

    /// Number of samples (seconds) to collect for memory
    #[arg(long, default_value_t = 5)]
    mem_samples: usize,

    /// Minimum high memory hits to consider sustained
    #[arg(long, default_value_t = 2)]
    mem_min_hits: usize,

    /// Disk usage percent threshold to consider high
    #[arg(long, default_value_t = 90.0)]
    disk_threshold: f32,

    /// Number of samples (seconds) to collect for disk
    #[arg(long, default_value_t = 5)]
    disk_samples: usize,

    /// Minimum high disk hits to consider sustained
    #[arg(long, default_value_t = 2)]
    disk_min_hits: usize,

    /// Output machine-readable JSON
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let args = Args::parse();

    println!("Monitoring CPU + memory usage...\n");

    let cpu_result = detect_sustained_high_cpu(args.cpu_threshold, args.cpu_samples, args.cpu_min_hits);
    let mem_result = detect_sustained_high_mem(args.mem_threshold, args.mem_samples, args.mem_min_hits);
    let disk_result = disk::detect_sustained_high_disk(args.disk_threshold, args.disk_samples, args.disk_min_hits);

    if args.json {
        let mut out = json!({});

        if let Some(c) = cpu_result {
            if let serde_json::Value::Object(ref mut map) = out {
                map.insert("cpu".to_string(), serde_json::to_value(&c).unwrap());
            }
        }

        if let Some(m) = mem_result {
            if let serde_json::Value::Object(ref mut map) = out {
                map.insert("mem".to_string(), serde_json::to_value(&m).unwrap());
            }
        }

        if let Some(d) = disk_result {
            if let serde_json::Value::Object(ref mut map) = out {
                map.insert("disk".to_string(), serde_json::to_value(&d).unwrap());
            }
        }

        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return;
    }

    match cpu_result {
        Some(sample) => {
            println!(
                "Sustained high CPU usage detected:\n• {} (PID {}) – {:.1}% CPU\n",
                sample.name, sample.pid, sample.cpu
            );

            println!("Explanation:");
            println!("{}", explain_process(&sample.name));
        }
        None => {
            println!("CPU usage looks normal.");
        }
    }

    match mem_result {
        Some(sample) => {
            println!(
                "\nSustained high memory usage detected:\n• {} (PID {}) – {:.1}% mem (system {:.1}%)\n",
                sample.name, sample.pid, sample.mem, sample.used_percent
            );

            println!("Explanation:");
            println!("{}", explain_process(&sample.name));
        }
        None => {
            println!("Memory usage looks normal.");
        }
    }

    match disk_result {
        Some(sample) => {
            println!(
                "\nSustained high disk usage detected:\n• {} mounted on {} – {:.1}% used\n",
                sample.fs, sample.mount, sample.used_percent
            );

            println!("Explanation:");
            println!("{}", explain_process("disk"));
        }
        None => {
            println!("Disk usage looks normal.");
        }
    }
}
