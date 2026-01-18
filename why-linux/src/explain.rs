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

        _ =>
            "Sustained high resource usage usually means a process is busy, leaking memory, or stuck.
If this happens while idle, consider:
• checking which resources the process is using (`ps`, `top`, `smem`)
• restarting the process
• checking for known bugs or extensions
• if memory is full, consider adding swap or investigating memory leaks."
    }
}
