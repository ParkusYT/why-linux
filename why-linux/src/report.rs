// Single clean implementation of the HTML report writer.
use crate::cpu::CpuSample;
use crate::mem::MemSample;
use crate::disk::DiskSample;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct TimelineSample {
    pub ts: u64,
    pub cpu: Option<CpuSample>,
    pub mem: Option<MemSample>,
    pub disk: Option<DiskSample>,
}

pub fn write_html_report(path: &str, samples: &[TimelineSample], summary_json: &str) -> std::io::Result<()> {
    let mut f = File::create(path)?;

    let samples_json = serde_json::to_string_pretty(samples).unwrap_or_else(|_| "[]".to_string());
    // assume summary_json is valid JSON
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let html = format!(r##"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>why-linux report</title>
  <style>
    body {{ font-family: system-ui, Arial, sans-serif; margin: 20px; }}
    .chart {{ width: 100%; height: 60px; }}
    .row {{ margin-bottom: 18px; }}
    .small {{ color: #666; font-size: 0.9em }}
    pre {{ background: #f6f8fa; padding: 12px; border-radius: 6px }}
  </style>
</head>
<body>
  <h1>why-linux report</h1>
  <p class="small">Generated at {ts}</p>

  <div class="row">
    <h3>CPU (top per-sample)</h3>
    <svg id="cpu" class="chart"></svg>
  </div>

  <div class="row">
    <h3>Memory (top per-sample)</h3>
    <svg id="mem" class="chart"></svg>
  </div>

  <div class="row">
    <h3>Disk usage (top per-sample)</h3>
    <svg id="disk" class="chart"></svg>
  </div>

  <h3>Summary</h3>
  <pre id="summary"></pre>

  <script>
    const samples = {samples_json};
    const summary = {summary_json};

    function sparkline(values, el) {{
      const rect = el.getBoundingClientRect();
      const w = Math.max(1, Math.floor(rect.width)) || 600;
      const h = 60;
      if (!values || values.length === 0) {{
        el.setAttribute('viewBox', `0 0 ${{w}} ${{h}}`);
        el.innerHTML = '';
        return;
      }}
      const maxv = Math.max(...values, 1);
      const minv = Math.min(...values);
      const step = w / Math.max(values.length - 1, 1);
      let path = '';
      values.forEach(function(v,i){{
        const x = i * step;
        const y = h - ((v - minv)/(maxv - minv || 1)) * (h - 4) - 2;
        path += (i==0 ? ('M ' + x + ' ' + y) : (' L ' + x + ' ' + y));
      }});
      el.setAttribute('viewBox', `0 0 ${{w}} ${{h}}`);
      el.innerHTML = '<path d="' + path + '" fill="none" stroke="#1976d2" stroke-width="2"/>';
    }}

    // prepare numeric series
    const cpuSeries = samples.map(s=> s.cpu ? s.cpu.cpu : 0);
    const memSeries = samples.map(s=> s.mem ? s.mem.mem : 0);
    const diskSeries = samples.map(s=> s.disk ? s.disk.used_percent : 0);

    document.addEventListener('DOMContentLoaded', function() {{
      sparkline(cpuSeries, document.getElementById('cpu'));
      sparkline(memSeries, document.getElementById('mem'));
      sparkline(diskSeries, document.getElementById('disk'));
      document.getElementById('summary').textContent = JSON.stringify(summary, null, 2);
    }});
  </script>
</body>
</html>"##);

    f.write_all(html.as_bytes())?;
    Ok(())
}
