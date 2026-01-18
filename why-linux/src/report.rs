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
    table {{ border-collapse: collapse; width: 100%; margin-top: 8px; }}
    th, td {{ text-align: left; padding: 6px 8px; border-bottom: 1px solid #e0e0e0; }}
    th {{ background: #f2f4f7; }}
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
  <div id="summary-cards"></div>

  <h3>Top offenders</h3>
  <div id="offenders"></div>

  <h3>Raw JSON</h3>
  <pre id="summary"></pre>

  <script>
    const samples = {samples_json};
    const data = {summary_json};
    const summary = data.summary || Object();
    const offenders = data.offenders || Object();

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

    function fmt(v) {{
      return (typeof v === 'number' && isFinite(v)) ? v.toFixed(1) : '0.0';
    }}

    function renderSummary() {{
      const el = document.getElementById('summary-cards');
      if (!el || !summary) return;
      const cpu = summary.cpu || Object();
      const mem = summary.mem || Object();
      const disk = summary.disk || Object();
      el.innerHTML =
        '<div class="row"><strong>CPU</strong>: avg ' + fmt(cpu.avg) + '% | max ' + fmt(cpu.max) + '%</div>' +
        '<div class="row"><strong>Memory</strong>: avg ' + fmt(mem.avg) + '% | max ' + fmt(mem.max) + '% | system avg ' + fmt(mem.system_avg) + '% | system max ' + fmt(mem.system_max) + '%</div>' +
        '<div class="row"><strong>Disk</strong>: avg ' + fmt(disk.avg) + '% | max ' + fmt(disk.max) + '%</div>';
    }}

    function renderOffenders() {{
      const el = document.getElementById('offenders');
      if (!el) return;
      const cpu = offenders.cpu || [];
      const mem = offenders.mem || [];
      function rows(title, items) {{
        if (!items.length) return '<div class="row"><strong>' + title + '</strong>: none</div>';
        const list = items.map(i =>
          '<tr>' +
            '<td>' + i.name + '</td>' +
            '<td>' + i.pid + '</td>' +
            '<td>' + fmt(i.sum ?? 0) + '</td>' +
            '<td>' + fmt(i.avg ?? 0) + '</td>' +
            '<td>' + fmt(i.max ?? 0) + '</td>' +
          '</tr>'
        ).join('');
        return '<div class="row"><strong>' + title + '</strong>' +
          '<table>' +
            '<thead><tr><th>Name</th><th>PID</th><th>Sum</th><th>Avg</th><th>Max</th></tr></thead>' +
            '<tbody>' + list + '</tbody>' +
          '</table>' +
        '</div>';
      }}
      el.innerHTML = rows('CPU offenders', cpu) + rows('Memory offenders', mem);
    }}

    document.addEventListener('DOMContentLoaded', function() {{
      sparkline(cpuSeries, document.getElementById('cpu'));
      sparkline(memSeries, document.getElementById('mem'));
      sparkline(diskSeries, document.getElementById('disk'));
      renderSummary();
      renderOffenders();
      document.getElementById('summary').textContent = JSON.stringify(data, null, 2);
    }});
  </script>
</body>
</html>"##);

    f.write_all(html.as_bytes())?;
    Ok(())
}
