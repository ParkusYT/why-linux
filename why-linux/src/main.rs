mod cpu;
mod explain;
mod mem;
mod disk;
mod io;
mod report;

use clap::Parser;
use serde_json::json;

use cpu::detect_sustained_high_cpu;
use explain::explain_process;
use mem::detect_sustained_high_mem;
use report::{TimelineSample, write_html_report};

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

    /// Read bytes/sec threshold to consider high (bytes/sec)
    #[arg(long, default_value_t = 5_000_000)]
    io_read_threshold: u64,

    /// Write bytes/sec threshold to consider high (bytes/sec)
    #[arg(long, default_value_t = 5_000_000)]
    io_write_threshold: u64,

    /// Number of samples (seconds) to collect for io
    #[arg(long, default_value_t = 5)]
    io_samples: usize,

    /// Minimum high io hits to consider sustained
    #[arg(long, default_value_t = 2)]
    io_min_hits: usize,

    /// Output machine-readable JSON
    #[arg(short, long)]
    json: bool,

    /// Write an HTML timeline report to the given path (optional)
    #[arg(long)]
    report: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("Monitoring CPU + memory usage...\n");

    // Extract needed args so we can move them into threads.
    let cpu_threshold = args.cpu_threshold;
    let cpu_samples = args.cpu_samples;
    let cpu_min_hits = args.cpu_min_hits;

    let mem_threshold = args.mem_threshold;
    let mem_samples = args.mem_samples;
    let mem_min_hits = args.mem_min_hits;

    let disk_threshold = args.disk_threshold;
    let disk_samples = args.disk_samples;
    let disk_min_hits = args.disk_min_hits;

    let io_read_threshold = args.io_read_threshold;
    let io_write_threshold = args.io_write_threshold;
    let io_samples = args.io_samples;
    let io_min_hits = args.io_min_hits;

    // Start parallel detectors (they still sample internally) and also collect per-second
    // timeline samples for the maximum of the configured sample windows so the report has data.
    let cpu_handle = std::thread::spawn(move || {
        detect_sustained_high_cpu(cpu_threshold, cpu_samples, cpu_min_hits)
    });

    let mem_handle = std::thread::spawn(move || {
        detect_sustained_high_mem(mem_threshold, mem_samples, mem_min_hits)
    });

    let disk_handle = std::thread::spawn(move || {
        disk::detect_sustained_high_disk(disk_threshold, disk_samples, disk_min_hits)
    });

    let io_handle = std::thread::spawn(move || {
        io::detect_sustained_high_io(io_read_threshold, io_write_threshold, io_samples, io_min_hits)
    });

    // collect per-second samples for timeline (duration = max configured samples)
    let duration = *[cpu_samples, mem_samples, disk_samples, io_samples].iter().max().unwrap_or(&1);
    let mut timeline: Vec<TimelineSample> = Vec::with_capacity(duration);
    for _ in 0..duration {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let cpu = cpu::get_top_cpu();
        let mem = mem::get_top_mem();
        let disk = disk::get_top_mount_usage();
        timeline.push(TimelineSample { ts, cpu, mem, disk });
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Join results (if a thread panics we treat as no result)
    let cpu_result = cpu_handle.join().ok().flatten();
    let mem_result = mem_handle.join().ok().flatten();
    let disk_result = disk_handle.join().ok().flatten();
    let io_result = io_handle.join().ok().flatten();

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

        if let Some(io) = io_result {
            if let serde_json::Value::Object(ref mut map) = out {
                map.insert("io".to_string(), serde_json::to_value(&io).unwrap());
            }
        }

        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        if let Some(path) = args.report.as_ref() {
            // include JSON findings in the report
            let summary_json = serde_json::to_string_pretty(&out).unwrap();
            let _ = write_html_report(path, &timeline, &summary_json);
        }

        return;
    }

    match cpu_result.as_ref() {
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

    match mem_result.as_ref() {
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

    match disk_result.as_ref() {
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

    match io_result.as_ref() {
        Some(sample) => {
            println!(
                "\nSustained high I/O detected:\n• {} (PID {}) – read {} B/s, write {} B/s\n",
                sample.name, sample.pid, sample.read_bps, sample.write_bps
            );

            println!("Explanation:");
            println!("{}", explain_process("io"));
        }
        None => {
            println!("I/O looks normal.");
        }
    }

    if let Some(path) = args.report.as_ref() {
        let mut out = json!({});
        if let Some(c) = cpu_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("cpu".to_string(), serde_json::to_value(&c).unwrap()); } }
        if let Some(m) = mem_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("mem".to_string(), serde_json::to_value(&m).unwrap()); } }
        if let Some(d) = disk_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("disk".to_string(), serde_json::to_value(&d).unwrap()); } }
        if let Some(i) = io_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("io".to_string(), serde_json::to_value(&i).unwrap()); } }

        let summary_json = serde_json::to_string_pretty(&out).unwrap();
        match write_html_report(path, &timeline, &summary_json) {
            Ok(()) => println!("Wrote HTML report to {}", path),
            Err(e) => eprintln!("Failed to write report: {}", e),
        }
    }
}
