mod cpu;
mod explain;

use cpu::detect_sustained_high_cpu;
use explain::explain_process;

fn main() {
    println!("Monitoring CPU usage...\n");

    // Detect sustained high CPU over time
    let result = detect_sustained_high_cpu(
        20.0, // CPU threshold (%)
        5,    // total samples (seconds)
        3,    // minimum high samples to consider sustained
    );

    match result {
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
}
