use std::thread::sleep;
use std::time::Duration;
use std::fs;

#[derive(Debug)]
pub struct CpuSample {
    pub name: String,
    pub pid: u32,
    pub cpu: f32,
}

pub fn get_top_cpu() -> Option<CpuSample> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid,comm,%cpu", "--sort=-%cpu"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    lines.next(); // skip header

    let line = lines.next()?;
    let mut parts = line.split_whitespace();

    let pid: u32 = parts.next()?.parse().ok()?;
    let name = parts.next()?.to_string();
    let cpu: f32 = parts.next()?.parse().ok()?;

    let mut sample = CpuSample { name, pid, cpu };

    // Check if this is a known browser child process
    if sample.name == "Web" || sample.name == "GPU" {
        if let Some(parent) = get_parent_process(sample.pid) {
            sample = parent;
        }
    }

    Some(sample)
}

fn get_parent_process(pid: u32) -> Option<CpuSample> {
    let stat_path = format!("/proc/{}/stat", pid);
    let contents = fs::read_to_string(stat_path).ok()?;
    let parts: Vec<&str> = contents.split_whitespace().collect();

    // stat format: pid (comm) state ppid ...
    let ppid: u32 = parts.get(3)?.parse().ok()?;

    // get CPU usage for parent via ps
    let output = std::process::Command::new("ps")
        .args(["-p", &ppid.to_string(), "-o", "pid,comm,%cpu"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    lines.next()?; // skip header
    let line = lines.next()?;
    let mut parts = line.split_whitespace();

    let pid: u32 = parts.next()?.parse().ok()?;
    let name = parts.next()?.to_string();
    let cpu: f32 = parts.next()?.parse().ok()?;

    Some(CpuSample { pid, name, cpu })
}

pub fn detect_sustained_high_cpu(
    threshold: f32,
    samples: usize,
    min_hits: usize,
) -> Option<CpuSample> {
    let mut hits = 0;
    let mut last_sample = None;

    for _ in 0..samples {
        if let Some(sample) = get_top_cpu() {
            if sample.cpu > threshold {
                hits += 1;
                last_sample = Some(sample);
            }
        }

        sleep(Duration::from_secs(1));
    }

    if hits >= min_hits {
        last_sample
    } else {
        None
    }
}
