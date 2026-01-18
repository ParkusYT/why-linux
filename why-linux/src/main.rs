use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn get_top_cpu() -> Option<(String, u32, f32)> {
    let output = Command::new("ps")
        .args(["-eo", "pid,comm,%cpu", "--sort=-%cpu"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    lines.next();

    let line = lines.next()?;
    let mut parts = line.split_whitespace();

    let pid = parts.next()?.parse().ok()?;
    let name = parts.next()?.to_string();
    let cpu = parts.next()?.parse().ok()?;

    Some((name, pid, cpu))
}

fn main() {
    let mut high_count = 0;

    for _ in 0..5 {
        if let Some((name, pid, cpu)) = get_top_cpu() {
            if cpu > 20.0 {
                high_count += 1;
                println!("High CPU sample: {} (PID {}) - {:.1}%", name, pid, cpu);
            }
        }

        sleep(Duration::from_secs(1));
    }

    if high_count >= 3 {
        println!("\nSustained high CPU usage detected.");
        println!("This usually means an application is stuck or very busy.");
        println!("If this happens while idle, it may be a bug or runaway task.");
    } else {
        println!("\nCPU usage looks normal.");
    }
}