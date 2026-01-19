mod cpu;
mod explain;
mod mem;
mod disk;
mod io;
mod report;

use clap::Parser;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

use cpu::detect_sustained_high_cpu;
use explain::explain_process;
use mem::detect_sustained_high_mem;
use report::{TimelineSample, write_html_report};

#[derive(Clone)]
struct OffenderStats {
    name: String,
    pid: u32,
    sum: f32,
    max: f32,
    samples: u32,
}

#[derive(Serialize)]
struct OffenderRow {
    name: String,
    pid: u32,
    sum: f32,
    avg: f32,
    max: f32,
}

fn update_offender(
    map: &mut HashMap<u32, OffenderStats>,
    pid: u32,
    name: &str,
    value: f32,
) {
    let entry = map.entry(pid).or_insert_with(|| OffenderStats {
        name: name.to_string(),
        pid,
        sum: 0.0,
        max: 0.0,
        samples: 0,
    });
    entry.name = name.to_string();
    entry.sum += value;
    entry.max = entry.max.max(value);
    entry.samples += 1;
}

fn avg_of(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn max_of(values: &[f32]) -> f32 {
    values.iter().cloned().fold(0.0, f32::max)
}

fn top_offenders(map: &HashMap<u32, OffenderStats>, limit: usize) -> Vec<OffenderRow> {
    let mut rows: Vec<OffenderRow> = map
        .values()
        .map(|o| OffenderRow {
            name: o.name.clone(),
            pid: o.pid,
            sum: o.sum,
            avg: if o.samples == 0 { 0.0 } else { o.sum / o.samples as f32 },
            max: o.max,
        })
        .collect();

    rows.sort_by(|a, b| b.sum.partial_cmp(&a.sum).unwrap_or(std::cmp::Ordering::Equal));
    rows.truncate(limit);
    rows
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Monitor sustained CPU and memory usage")]
struct Args {
    /// Total duration to sample (seconds)
    #[arg(long, default_value_t = 10)]
    duration: u64,

    /// Sampling interval (seconds)
    #[arg(long, default_value_t = 1)]
    interval: u64,

    /// CPU threshold percentage to consider high
    #[arg(long, default_value_t = 20.0)]
    cpu_threshold: f32,

    /// System memory used percent threshold
    #[arg(long, default_value_t = 80.0)]
    mem_threshold: f32,


    /// Disk usage percent threshold to consider high
    #[arg(long, default_value_t = 90.0)]
    disk_threshold: f32,


    /// Read bytes/sec threshold to consider high (bytes/sec)
    #[arg(long, default_value_t = 5_000_000)]
    io_read_threshold: u64,

    /// Write bytes/sec threshold to consider high (bytes/sec)
    #[arg(long, default_value_t = 5_000_000)]
    io_write_threshold: u64,


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
    let self_pid = std::process::id();

    // Extract needed args so we can move them into threads.
    let duration = args.duration.max(1);
    let interval = args.interval.max(1);
    let samples = (duration / interval).max(1) as usize;
    let min_hits = (samples / 2).max(1);

    let cpu_threshold = args.cpu_threshold;
    let mem_threshold = args.mem_threshold;
    let disk_threshold = args.disk_threshold;
    let io_read_threshold = args.io_read_threshold;
    let io_write_threshold = args.io_write_threshold;

    // Start parallel detectors (they still sample internally) and also collect per-second
    // timeline samples for the maximum of the configured sample windows so the report has data.
    let cpu_handle = std::thread::spawn(move || {
        detect_sustained_high_cpu(cpu_threshold, samples, min_hits, interval, Some(self_pid))
    });

    let mem_handle = std::thread::spawn(move || {
        detect_sustained_high_mem(mem_threshold, samples, min_hits, interval, Some(self_pid))
    });

    let disk_handle = std::thread::spawn(move || {
        disk::detect_sustained_high_disk(disk_threshold, samples, min_hits, interval)
    });

    let io_handle = std::thread::spawn(move || {
        io::detect_sustained_high_io(io_read_threshold, io_write_threshold, samples, min_hits, interval)
    });

    // collect per-second samples for timeline (duration = max configured samples)
    let mut timeline: Vec<TimelineSample> = Vec::with_capacity(samples);
    let mut cpu_values: Vec<f32> = Vec::with_capacity(samples);
    let mut mem_values: Vec<f32> = Vec::with_capacity(samples);
    let mut mem_used_values: Vec<f32> = Vec::with_capacity(samples);
    let mut disk_values: Vec<f32> = Vec::with_capacity(samples);
    let mut cpu_offenders: HashMap<u32, OffenderStats> = HashMap::new();
    let mut mem_offenders: HashMap<u32, OffenderStats> = HashMap::new();

    for _ in 0..samples {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cpu = cpu::get_top_cpu_excluding(Some(self_pid));
        let mem = mem::get_top_mem_excluding(Some(self_pid));
        let disk = disk::get_top_mount_usage();

        if let Some(ref sample) = cpu {
            cpu_values.push(sample.cpu);
            update_offender(&mut cpu_offenders, sample.pid, &sample.name, sample.cpu);
        } else {
            cpu_values.push(0.0);
        }

        if let Some(ref sample) = mem {
            mem_values.push(sample.mem);
            mem_used_values.push(sample.used_percent);
            update_offender(&mut mem_offenders, sample.pid, &sample.name, sample.mem);
        } else {
            mem_values.push(0.0);
            mem_used_values.push(0.0);
        }

        if let Some(ref sample) = disk {
            disk_values.push(sample.used_percent);
        } else {
            disk_values.push(0.0);
        }

        timeline.push(TimelineSample { ts, cpu, mem, disk });
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }

    let summary = json!({
        "cpu": {
            "avg": avg_of(&cpu_values),
            "max": max_of(&cpu_values),
        },
        "mem": {
            "avg": avg_of(&mem_values),
            "max": max_of(&mem_values),
            "system_avg": avg_of(&mem_used_values),
            "system_max": max_of(&mem_used_values),
        },
        "disk": {
            "avg": avg_of(&disk_values),
            "max": max_of(&disk_values),
        }
    });

    let offenders = json!({
        "cpu": top_offenders(&cpu_offenders, 5),
        "mem": top_offenders(&mem_offenders, 5),
    });

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

        if let serde_json::Value::Object(ref mut map) = out {
            map.insert("summary".to_string(), summary.clone());
            map.insert("offenders".to_string(), offenders.clone());
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

    println!("\nSummary ({}s):", duration);
    println!(
        "CPU avg {:.1}% | max {:.1}%",
        summary["cpu"]["avg"].as_f64().unwrap_or(0.0),
        summary["cpu"]["max"].as_f64().unwrap_or(0.0)
    );
    println!(
        "Mem avg {:.1}% | max {:.1}% | system avg {:.1}% | system max {:.1}%",
        summary["mem"]["avg"].as_f64().unwrap_or(0.0),
        summary["mem"]["max"].as_f64().unwrap_or(0.0),
        summary["mem"]["system_avg"].as_f64().unwrap_or(0.0),
        summary["mem"]["system_max"].as_f64().unwrap_or(0.0)
    );
    println!(
        "Disk avg {:.1}% | max {:.1}%",
        summary["disk"]["avg"].as_f64().unwrap_or(0.0),
        summary["disk"]["max"].as_f64().unwrap_or(0.0)
    );

    let cpu_top = top_offenders(&cpu_offenders, 3);
    if !cpu_top.is_empty() {
        println!("\nTop CPU offenders:");
        for row in cpu_top {
            println!(
                "• {} (PID {}) – sum {:.1} | avg {:.1} | max {:.1}",
                row.name, row.pid, row.sum, row.avg, row.max
            );
        }
    }

    let mem_top = top_offenders(&mem_offenders, 3);
    if !mem_top.is_empty() {
        println!("\nTop memory offenders:");
        for row in mem_top {
            println!(
                "• {} (PID {}) – sum {:.1} | avg {:.1} | max {:.1}",
                row.name, row.pid, row.sum, row.avg, row.max
            );
        }
    }

    if let Some(path) = args.report.as_ref() {
        let mut out = json!({});
        if let Some(c) = cpu_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("cpu".to_string(), serde_json::to_value(&c).unwrap()); } }
        if let Some(m) = mem_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("mem".to_string(), serde_json::to_value(&m).unwrap()); } }
        if let Some(d) = disk_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("disk".to_string(), serde_json::to_value(&d).unwrap()); } }
        if let Some(i) = io_result { if let serde_json::Value::Object(ref mut map) = out { map.insert("io".to_string(), serde_json::to_value(&i).unwrap()); } }

        if let serde_json::Value::Object(ref mut map) = out {
            map.insert("summary".to_string(), summary);
            map.insert("offenders".to_string(), offenders);
        }

        let summary_json = serde_json::to_string_pretty(&out).unwrap();
        match write_html_report(path, &timeline, &summary_json) {
            Ok(()) => println!("Wrote HTML report to {}", path),
            Err(e) => eprintln!("Failed to write report: {}", e),
        }
    }
}
