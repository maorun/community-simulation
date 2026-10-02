//! Interactive web dashboard for exploring simulation results.
//!
//! This module renders a self-contained HTML page (using [Chart.js](https://www.chartjs.org/)
//! loaded from a CDN, no build step required) that visualizes key metrics from a
//! [`SimulationResult`]: skill price history over time, the wealth distribution
//! histogram, and social class mobility. It also provides a tiny local HTTP server
//! (built on `std::net`, no additional dependencies) that serves the generated page
//! so results can be explored interactively in a browser, as a richer alternative to
//! the terminal ASCII charts.
//!
//! # Example
//!
//! ```no_run
//! use community_simulation::result::SimulationResult;
//! use community_simulation::dashboard;
//!
//! # fn example(result: &SimulationResult) -> std::io::Result<()> {
//! let html = dashboard::generate_dashboard_html(result);
//! dashboard::serve(html, 8080)
//! # }
//! ```

use crate::result::SimulationResult;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// The subset of a [`SimulationResult`] that is embedded into the dashboard page as JSON
/// so client-side JavaScript can render charts without any additional server round-trips.
#[derive(Serialize)]
struct DashboardData<'a> {
    total_steps: usize,
    active_persons: usize,
    average_money: f64,
    average_money_trend: &'static str,
    median_money: f64,
    median_money_trend: &'static str,
    gini_coefficient: f64,
    gini_coefficient_trend: &'static str,
    skill_price_history: &'a HashMap<String, Vec<f64>>,
    final_money_distribution: &'a [f64],
    social_class_labels: Vec<&'static str>,
    social_class_counts: Vec<usize>,
    upward_mobility_rate: f64,
    downward_mobility_rate: f64,
    transition_matrix: &'a [Vec<usize>],
}

/// Return the visual trend for a metric's change since the preceding simulation step.
///
/// A missing prior value and an unchanged or non-finite comparison are rendered as a
/// neutral arrow so every metric has a consistent indicator.
fn trend_symbol(current: f64, previous: Option<f64>) -> &'static str {
    match previous.and_then(|previous| current.partial_cmp(&previous)) {
        Some(std::cmp::Ordering::Greater) => "↑",
        Some(std::cmp::Ordering::Less) => "↓",
        _ => "→",
    }
}

/// Render a self-contained interactive HTML dashboard for the given simulation result.
///
/// The returned string is a complete HTML document (including an inline `<script>`
/// that loads Chart.js from a CDN) that can be written to a file or served directly
/// over HTTP via [`serve`].
pub fn generate_dashboard_html(result: &SimulationResult) -> String {
    let previous_snapshot = result
        .wealth_stats_history
        .len()
        .checked_sub(2)
        .and_then(|index| result.wealth_stats_history.get(index));
    let data = DashboardData {
        total_steps: result.total_steps,
        active_persons: result.active_persons,
        average_money: result.money_statistics.average,
        average_money_trend: trend_symbol(
            result.money_statistics.average,
            previous_snapshot.map(|snapshot| snapshot.average),
        ),
        median_money: result.money_statistics.median,
        median_money_trend: trend_symbol(
            result.money_statistics.median,
            previous_snapshot.map(|snapshot| snapshot.median),
        ),
        gini_coefficient: result.money_statistics.gini_coefficient,
        gini_coefficient_trend: trend_symbol(
            result.money_statistics.gini_coefficient,
            previous_snapshot.map(|snapshot| snapshot.gini_coefficient),
        ),
        skill_price_history: &result.skill_price_history,
        final_money_distribution: &result.final_money_distribution,
        social_class_labels: vec!["Lower", "Middle", "Upper", "Elite"],
        social_class_counts: vec![
            result.social_class_statistics.lower_class_count,
            result.social_class_statistics.middle_class_count,
            result.social_class_statistics.upper_class_count,
            result.social_class_statistics.elite_class_count,
        ],
        upward_mobility_rate: result.social_class_statistics.upward_mobility_rate,
        downward_mobility_rate: result.social_class_statistics.downward_mobility_rate,
        transition_matrix: &result.social_class_statistics.transition_matrix,
    };

    let data_json = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());

    // Chart.js is pinned to a fixed version (rather than "latest") so the dashboard's
    // appearance and behavior stay stable across runs and are not affected by upstream
    // CDN changes.
    const CHART_JS_VERSION: &str = "4.4.4";
    // Maximum number of skill price-history lines to show by default; datasets beyond
    // this count are still available for toggling in the legend but start hidden so the
    // chart stays readable for simulations with many skills.
    const MAX_VISIBLE_PRICE_SERIES: usize = 10;
    // Maximum number of buckets used for the wealth distribution histogram, keeping the
    // chart readable regardless of population size.
    const MAX_WEALTH_BUCKETS: usize = 20;

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Community Simulation Dashboard</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@{chart_js_version}/dist/chart.umd.min.js" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
<style>
  body {{ font-family: sans-serif; margin: 2rem; background: #111; color: #eee; }}
  h1 {{ font-size: 1.4rem; }}
  .summary {{ display: flex; gap: 2rem; margin-bottom: 2rem; flex-wrap: wrap; }}
  .summary div {{ background: #222; padding: 0.75rem 1rem; border-radius: 6px; }}
  .trend {{ margin-left: 0.25rem; font-weight: bold; }}
  .chart-container {{ background: #1b1b1b; padding: 1rem; border-radius: 8px; margin-bottom: 2rem; }}
  canvas {{ max-height: 400px; }}
</style>
</head>
<body>
<h1>Community Simulation Dashboard</h1>
<div class="summary">
  <div>Total steps: <strong id="total-steps"></strong></div>
  <div>Active persons: <strong id="active-persons"></strong></div>
  <div>Average wealth: <strong id="average-money"></strong><span class="trend" id="average-money-trend"></span></div>
  <div>Median wealth: <strong id="median-money"></strong><span class="trend" id="median-money-trend"></span></div>
  <div>Gini coefficient: <strong id="gini"></strong><span class="trend" id="gini-trend"></span></div>
</div>
<div class="chart-container"><canvas id="priceChart"></canvas></div>
<div class="chart-container"><canvas id="wealthChart"></canvas></div>
<div class="chart-container"><canvas id="mobilityChart"></canvas></div>
<script>
const DASHBOARD_DATA = {data_json};
const MAX_VISIBLE_PRICE_SERIES = {max_visible_price_series};
const MAX_WEALTH_BUCKETS = {max_wealth_buckets};

document.getElementById('total-steps').textContent = DASHBOARD_DATA.total_steps;
document.getElementById('active-persons').textContent = DASHBOARD_DATA.active_persons;
document.getElementById('average-money').textContent = DASHBOARD_DATA.average_money.toFixed(2);
document.getElementById('average-money-trend').textContent = DASHBOARD_DATA.average_money_trend;
document.getElementById('median-money').textContent = DASHBOARD_DATA.median_money.toFixed(2);
document.getElementById('median-money-trend').textContent = DASHBOARD_DATA.median_money_trend;
document.getElementById('gini').textContent = DASHBOARD_DATA.gini_coefficient.toFixed(4);
document.getElementById('gini-trend').textContent = DASHBOARD_DATA.gini_coefficient_trend;

// Price history over time
const priceLabels = Array.from(
  {{ length: Math.max(0, ...Object.values(DASHBOARD_DATA.skill_price_history).map(v => v.length)) }},
  (_, i) => i
);
const priceDatasets = Object.entries(DASHBOARD_DATA.skill_price_history).map(([skill, prices], idx) => ({{
  label: skill,
  data: prices,
  borderWidth: 1,
  fill: false,
  hidden: idx >= MAX_VISIBLE_PRICE_SERIES,
}}));
new Chart(document.getElementById('priceChart'), {{
  type: 'line',
  data: {{ labels: priceLabels, datasets: priceDatasets }},
  options: {{
    responsive: true,
    plugins: {{ title: {{ display: true, text: 'Skill Price History', color: '#eee' }} }},
    scales: {{ x: {{ ticks: {{ color: '#ccc' }} }}, y: {{ ticks: {{ color: '#ccc' }} }} }},
  }},
}});

// Wealth distribution histogram
const money = DASHBOARD_DATA.final_money_distribution.slice().sort((a, b) => a - b);
const bucketCount = Math.min(MAX_WEALTH_BUCKETS, Math.max(1, money.length));
const min = money.length ? money[0] : 0;
const max = money.length ? money[money.length - 1] : 0;
const bucketSize = (max - min) / bucketCount || 1;
const buckets = new Array(bucketCount).fill(0);
for (const m of money) {{
  let idx = Math.floor((m - min) / bucketSize);
  if (idx >= bucketCount) idx = bucketCount - 1;
  if (idx < 0) idx = 0;
  buckets[idx]++;
}}
const bucketLabels = buckets.map((_, i) => (min + i * bucketSize).toFixed(1));
new Chart(document.getElementById('wealthChart'), {{
  type: 'bar',
  data: {{ labels: bucketLabels, datasets: [{{ label: 'Persons', data: buckets, backgroundColor: '#4e79a7' }}] }},
  options: {{
    responsive: true,
    plugins: {{ title: {{ display: true, text: 'Wealth Distribution', color: '#eee' }} }},
    scales: {{ x: {{ ticks: {{ color: '#ccc' }} }}, y: {{ ticks: {{ color: '#ccc' }} }} }},
  }},
}});

// Social class mobility
new Chart(document.getElementById('mobilityChart'), {{
  type: 'bar',
  data: {{
    labels: DASHBOARD_DATA.social_class_labels,
    datasets: [
      {{ label: 'Persons per class', data: DASHBOARD_DATA.social_class_counts, backgroundColor: '#59a14f' }},
    ],
  }},
  options: {{
    responsive: true,
    plugins: {{
      title: {{
        display: true,
        text: `Social Class Mobility (upward ${{(DASHBOARD_DATA.upward_mobility_rate * 100).toFixed(1)}}%, downward ${{(DASHBOARD_DATA.downward_mobility_rate * 100).toFixed(1)}}%)`,
        color: '#eee',
      }},
    }},
    scales: {{ x: {{ ticks: {{ color: '#ccc' }} }}, y: {{ ticks: {{ color: '#ccc' }} }} }},
  }},
}});
</script>
</body>
</html>
"#,
        data_json = data_json,
        chart_js_version = CHART_JS_VERSION,
        max_visible_price_series = MAX_VISIBLE_PRICE_SERIES,
        max_wealth_buckets = MAX_WEALTH_BUCKETS,
    )
}

/// Serve the given HTML page over HTTP on `127.0.0.1:<port>` until the process is
/// interrupted (e.g. with Ctrl+C).
///
/// Every request receives the same page; the dashboard is fully self-contained and
/// does not require any additional API endpoints since all data is embedded inline.
pub fn serve(html: String, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("Dashboard available at http://127.0.0.1:{port}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &html),
            Err(e) => eprintln!("Connection failed: {e}"),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, html: &str) {
    // We don't need to parse the request; the dashboard serves the same page for
    // every request. Reading (and discarding) the request avoids some clients
    // seeing a connection reset before the response is written.
    let mut buf = [0u8; 1024];
    let _ = stream.read(&mut buf);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SimulationConfig;
    use crate::engine::SimulationEngine;
    use std::net::TcpStream;

    fn sample_result() -> SimulationResult {
        let config = SimulationConfig {
            max_steps: 5,
            entity_count: 6,
            seed: 42,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            ..Default::default()
        };
        let mut engine = SimulationEngine::new(config);
        engine.run()
    }

    #[test]
    fn generate_dashboard_html_contains_expected_structure() {
        let html = generate_dashboard_html(&sample_result());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("chart.js"));
        assert!(html.contains("priceChart"));
        assert!(html.contains("wealthChart"));
        assert!(html.contains("mobilityChart"));
    }

    #[test]
    fn generate_dashboard_html_embeds_result_data() {
        let result = sample_result();
        let html = generate_dashboard_html(&result);
        assert!(html.contains(&format!("\"total_steps\":{}", result.total_steps)));
        assert!(html.contains(&format!("\"active_persons\":{}", result.active_persons)));
        assert!(!result.skill_price_history.is_empty());
        for skill in result.skill_price_history.keys() {
            assert!(html.contains(skill));
        }
    }

    #[test]
    fn generate_dashboard_html_handles_empty_data() {
        let config = SimulationConfig { max_steps: 0, entity_count: 0, ..Default::default() };
        let mut engine = SimulationEngine::new(config);
        let result = engine.run();
        let html = generate_dashboard_html(&result);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("\"skill_price_history\":{}"));
    }

    #[test]
    fn generate_dashboard_html_embeds_metric_trends_from_previous_snapshot() {
        let mut result = sample_result();
        let previous = result
            .wealth_stats_history
            .get(result.wealth_stats_history.len() - 2)
            .expect("sample result has a previous wealth snapshot")
            .clone();

        result.money_statistics.average = previous.average + 1.0;
        result.money_statistics.median = previous.median - 1.0;
        result.money_statistics.gini_coefficient = previous.gini_coefficient;

        let html = generate_dashboard_html(&result);
        assert!(html.contains("\"average_money_trend\":\"↑\""));
        assert!(html.contains("\"median_money_trend\":\"↓\""));
        assert!(html.contains("\"gini_coefficient_trend\":\"→\""));
        assert!(html.contains("average-money-trend"));
        assert!(html.contains("median-money-trend"));
        assert!(html.contains("gini-trend"));
    }

    #[test]
    fn trend_symbol_is_neutral_without_a_previous_value_or_comparable_delta() {
        assert_eq!(trend_symbol(2.0, Some(1.0)), "↑");
        assert_eq!(trend_symbol(1.0, Some(2.0)), "↓");
        assert_eq!(trend_symbol(1.0, Some(1.0)), "→");
        assert_eq!(trend_symbol(1.0, None), "→");
        assert_eq!(trend_symbol(f64::NAN, Some(1.0)), "→");
    }

    #[test]
    fn serve_responds_with_html_over_tcp() {
        let html = generate_dashboard_html(&sample_result());
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();

        let expected = html.clone();
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept connection");
            handle_connection(stream, &expected);
        });

        let mut client = TcpStream::connect(addr).expect("connect to server");
        client.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        handle.join().unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Type: text/html"));
        assert!(response.contains("<!DOCTYPE html>"));
    }
}
