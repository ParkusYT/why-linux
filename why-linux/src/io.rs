use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct IoSample {
    pub pid: u32,
    pub name: String,
    pub read_bps: u64,
    pub write_bps: u64,
}

fn read_proc_io(pid: u32) -> Option<(u64, u64)> {
    let path = format!("/proc/{}/io", pid);
    let contents = fs::read_to_string(path).ok()?;
    let mut read_bytes: Option<u64> = None;
    let mut write_bytes: Option<u64> = None;

    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("read_bytes:") {
            read_bytes = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("write_bytes:") {
            write_bytes = rest.trim().parse().ok();
        }
    }

    Some((read_bytes?, write_bytes?))
}

fn all_pids() -> Vec<u32> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for e in entries.flatten() {
            if let Ok(name) = e.file_name().into_string() {
                if let Ok(pid) = name.parse::<u32>() {
                    v.push(pid);
                }
            }
        }
    }
    v
}

fn read_name(pid: u32) -> Option<String> {
    let path = format!("/proc/{}/comm", pid);
    let s = fs::read_to_string(path).ok()?;
    Some(s.trim().to_string())
}

pub fn detect_sustained_high_io(
    read_threshold: u64,
    write_threshold: u64,
    samples: usize,
    min_hits: usize,
) -> Option<IoSample> {
    let mut hits: HashMap<u32, usize> = HashMap::new();
    let mut last_values: HashMap<u32, (u64, u64)> = HashMap::new();
    let mut last_seen: HashMap<u32, (u64, u64, String)> = HashMap::new();

    for _ in 0..samples {
        // snapshot t0
        let pids = all_pids();
        last_values.clear();
        for pid in &pids {
            if let Some((r, w)) = read_proc_io(*pid) {
                last_values.insert(*pid, (r, w));
            }
        }

        sleep(Duration::from_secs(1));

        // snapshot t1 and compute deltas
        for pid in pids {
                if let (Some((r0, w0)), Some((r1, w1))) = (last_values.get(&pid), read_proc_io(pid)) {
                let read_delta = r1.saturating_sub(*r0);
                let write_delta = w1.saturating_sub(*w0);

                if read_delta > 0 || write_delta > 0 {
                    let name = read_name(pid).unwrap_or_else(|| "?".to_string());
                    last_seen.insert(pid, (read_delta, write_delta, name.clone()));

                    if read_delta >= read_threshold || write_delta >= write_threshold {
                        *hits.entry(pid).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // pick the pid with hits >= min_hits and highest combined bps
    let mut best: Option<(u32, u64, u64, String)> = None;

    for (pid, &count) in &hits {
        if count >= min_hits {
            if let Some((r, w, name)) = last_seen.get(pid) {
                match &best {
                    Some((_, br, bw, _)) if (br + bw) >= (r + w) => {}
                    _ => best = Some((*pid, *r, *w, name.clone())),
                }
            }
        }
    }

    best.map(|(pid, r, w, name)| IoSample {
        pid,
        name,
        read_bps: r,
        write_bps: w,
    })
}
