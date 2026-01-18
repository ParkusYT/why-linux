pub fn explain_process(name: &str) -> &'static str {
    match name {
        "firefox" =>
            "Firefox CPU usage is often caused by:
• heavy or broken tabs
• video playback or WebGL
• misbehaving extensions
• background service workers",

        "chromium" | "chrome" =>
            "Chromium-based browsers may use high CPU or memory due to:
• many open tabs (especially tabs with video/ads)
• background extensions or helper processes
• GPU acceleration issues
When memory is the problem, closing unused tabs or restarting the browser helps.",

        "kworker" =>
            "kworker is a kernel thread.
Sustained CPU usage here often indicates:
• driver bugs
• power management problems
• hardware issues",

        "disk" =>
            "High filesystem usage can cause system slowness and prevent writes.
Common causes:
• logs or caches filling the root or application partitions
• large backups or VM images stored on the same filesystem
• leftover build artifacts or package caches

Mitigation:
• free space by cleaning caches (e.g. package cache) or rotating logs
• move large files to another disk or expand the filesystem
• consider adding separate partitions for var/tmp or adding more disk space",

        "io" =>
            "High disk I/O (read/write) can make systems feel very slow, even with free space.
Common causes:
• running backups, rsync, or large file copies
• database or indexing workloads
• log-heavy applications or runaway processes writing continuously

Mitigation:
• identify the process with high I/O and throttle or reschedule it
• move heavy activity to off-peak times or faster storage
• add io-weighting via cgroups/ionice to deprioritize background jobs",

        _ =>
            "Sustained high resource usage usually means a process is busy, leaking memory, or stuck.
If this happens while idle, consider:
• checking which resources the process is using (`ps`, `top`, `smem`)
• restarting the process
• checking for known bugs or extensions
• if memory is full, consider adding swap or investigating memory leaks."
    }
}
