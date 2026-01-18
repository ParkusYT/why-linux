mod cpu;
mod explain;
mod mem;

use cpu::detect_sustained_high_cpu;
use explain::explain_process;
use mem::detect_sustained_high_mem;

fn main() {
    println!("Monitoring CPU + memory usage...\n");

    // Detect sustained high CPU over time
    let cpu_result = detect_sustained_high_cpu(20.0, 5, 3);

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

    // Detect sustained high memory usage
    let mem_result = detect_sustained_high_mem(80.0, 5, 2);

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
}
