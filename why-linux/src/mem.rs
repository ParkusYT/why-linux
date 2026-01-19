use std::thread::sleep;
use std::time::Duration;
use std::fs;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MemSample {
    pub name: String,
    pub pid: u32,
    pub mem: f32,
    pub used_percent: f32,
}

fn get_system_mem_used_percent() -> Option<f32> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total: Option<f32> = None;
    let mut available: Option<f32> = None;

    for line in contents.lines() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            total = parts.get(1)?.parse::<f32>().ok();
        } else if line.starts_with("MemAvailable:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            available = parts.get(1)?.parse::<f32>().ok();
        }
    }

    let total = total?;
    let available = available?;

    // meminfo values are in kB
    let used = total - available;
    Some((used / total) * 100.0)
}

pub fn get_top_mem_excluding(exclude_pid: Option<u32>) -> Option<MemSample> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid,comm,%mem", "--sort=-%mem"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    lines.next(); // skip header

    for line in lines {
        let mut parts = line.split_whitespace();
        let pid: u32 = parts.next()?.parse().ok()?;
        let name = parts.next()?.to_string();
        let mem: f32 = parts.next()?.parse().ok()?;

        if exclude_pid.is_some_and(|p| p == pid) {
            continue;
        }

        // determine system usage too
        let used_percent = get_system_mem_used_percent().unwrap_or(mem);

        return Some(MemSample {
            name,
            pid,
            mem,
            used_percent,
        });
    }

    None
}

pub fn detect_sustained_high_mem(
    threshold: f32,
    samples: usize,
    min_hits: usize,
    interval_secs: u64,
    exclude_pid: Option<u32>,
) -> Option<MemSample> {
    let mut hits = 0;
    let mut last_sample = None;

    for _ in 0..samples {
        if let Some(sys_used) = get_system_mem_used_percent() {
            if sys_used > threshold {
                if let Some(sample) = get_top_mem_excluding(exclude_pid) {
                    hits += 1;
                    last_sample = Some(sample);
                }
            }
        }

        sleep(Duration::from_secs(interval_secs));
    }

    if hits >= min_hits {
        last_sample
    } else {
        None
    }
}
