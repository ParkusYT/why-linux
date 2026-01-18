pub fn explain_process(name: &str) -> &'static str {
    match name {
        "firefox" =>
            "Firefox CPU usage is often caused by:
• heavy or broken tabs
• video playback or WebGL
• misbehaving extensions
• background service workers",

        "chromium" | "chrome" =>
            "Chromium-based browsers may use high CPU due to:
• many open tabs
• background extensions
• GPU acceleration issues",

        "kworker" =>
            "kworker is a kernel thread.
Sustained CPU usage here often indicates:
• driver bugs
• power management problems
• hardware issues",

        _ =>
            "Sustained high CPU usage usually means a process is busy or stuck.
If this happens while idle, it may indicate a bug."
    }
}
