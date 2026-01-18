use serde::Serialize;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct DiskSample {
    pub fs: String,
    pub mount: String,
    pub used_percent: f32,
}

pub fn get_top_mount_usage() -> Option<DiskSample> {
    let output = Command::new("df").arg("-P").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut best: Option<DiskSample> = None;

    for (i, line) in stdout.lines().enumerate() {
        if i == 0 {
            continue; // header
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        // Expect: filesystem 1024-blocks used available capacity Mounted_on
        if parts.len() < 6 {
            continue;
        }

        let fs = parts[0].to_string();
        let cap = parts[4];
        let mount = parts[5].to_string();

        let percent = cap.trim_end_matches('%').parse::<f32>().ok()?;

        let sample = DiskSample {
            fs,
            mount,
            used_percent: percent,
        };

        match &best {
            Some(b) if b.used_percent >= sample.used_percent => {}
            _ => best = Some(sample),
        }
    }

    best
}

pub fn detect_sustained_high_disk(threshold: f32, samples: usize, min_hits: usize) -> Option<DiskSample> {
    let mut hits = 0;
    let mut last = None;

    for _ in 0..samples {
        if let Some(sample) = get_top_mount_usage() {
            if sample.used_percent > threshold {
                hits += 1;
                last = Some(sample);
            }
        }

        sleep(Duration::from_secs(1));
    }

    if hits >= min_hits {
        last
    } else {
        None
    }
}
