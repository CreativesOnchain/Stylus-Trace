//! Terminal output rendering for diff reports.
//!
//! Provides human-readable summaries of profile comparisons
//! with visual cues (emojis) for regressions and improvements.

use super::schema::DiffReport;
use colored::*;

/// Render a human-readable summary of a diff report for the terminal
pub fn render_terminal_diff(report: &DiffReport) -> String {
    let mut out = String::new();

    out.push_str(&render_header(report));
    out.push_str(&render_gas_delta(report));
    out.push_str(&render_hostio_summary(report));
    out.push_str(&render_hostio_details(report));
    out.push_str(&render_hot_paths(report));
    out.push_str(&render_status(report));

    out
}

fn render_header(report: &DiffReport) -> String {
    let mut out = String::new();
    out.push_str("\n📊 ");
    out.push_str(&"Profile Comparison Summary".bold().to_string());
    out.push_str("\n---------------------------------------------------\n");
    out.push_str(&format!("Baseline: {}\n", report.baseline.transaction_hash));
    out.push_str(&format!("Target:   {}\n", report.target.transaction_hash));
    out.push_str("---------------------------------------------------\n\n");
    out
}

fn render_gas_delta(report: &DiffReport) -> String {
    let gas_delta = &report.deltas.gas;
    let symbol = get_delta_symbol(gas_delta.absolute_change);
    format!(
        "{} Total Gas: {} -> {} ({:+.2}%)\n",
        symbol, gas_delta.baseline, gas_delta.target, gas_delta.percent_change
    )
}

fn render_hostio_summary(report: &DiffReport) -> String {
    let hostio_delta = &report.deltas.hostio;
    let symbol = get_delta_symbol(hostio_delta.total_calls_change);
    format!(
        "{} HostIO Calls: {} -> {} ({:+.2}%)\n",
        symbol,
        hostio_delta.baseline_total_calls,
        hostio_delta.target_total_calls,
        hostio_delta.total_calls_percent_change
    )
}

fn render_hostio_details(report: &DiffReport) -> String {
    let mut out = String::new();
    let hostio_delta = &report.deltas.hostio;

    if !hostio_delta.by_type_changes.is_empty() {
        out.push_str("\nTop HostIO Changes:\n");
        let mut changes: Vec<_> = hostio_delta.by_type_changes.iter().collect();
        changes.sort_by(|a, b| b.1.delta.abs().cmp(&a.1.delta.abs()));

        for (hostio_type, change) in changes.iter().take(5) {
            let symbol = if change.delta > 0 { "📈" } else { "📉" };
            out.push_str(&format!(
                "  {} {}: {} -> {} ({:+})\n",
                symbol, hostio_type, change.baseline, change.target, change.delta
            ));
        }
    }
    out
}

fn render_hot_paths(report: &DiffReport) -> String {
    let mut out = String::new();
    let hot_paths = &report.deltas.hot_paths;

    if !hot_paths.common_paths.is_empty() {
        out.push_str("\nTop Hot Path Regressions/Improvements:\n");
        let mut hp_changes = hot_paths.common_paths.clone();
        hp_changes.sort_by(|a, b| b.gas_change.abs().cmp(&a.gas_change.abs()));

        for hp in hp_changes.iter().take(5) {
            let symbol = if hp.gas_change > 0 { "📈" } else { "📉" };
            out.push_str(&format!(
                "  {} {}: {} -> {} ({:+.2}%)\n",
                symbol,
                shorten_stack(&hp.stack),
                hp.baseline_gas,
                hp.target_gas,
                hp.percent_change
            ));
        }
    }
    out
}

fn render_status(report: &DiffReport) -> String {
    let mut out = String::new();
    out.push_str("\n---------------------------------------------------\n");
    let status_msg = match report.summary.status.as_str() {
        "FAILED" => format!(
            "❌ STATUS: REGRESSION DETECTED ({} violations)",
            report.summary.violation_count
        )
        .red()
        .bold(),
        "WARNING" => format!(
            "⚠️  STATUS: WARNING ({} violations)",
            report.summary.violation_count
        )
        .yellow()
        .bold(),
        _ => "✅ STATUS: PASSED".green().bold(),
    };
    out.push_str(&status_msg.to_string());
    out.push('\n');
    out
}

fn get_delta_symbol(change: i64) -> &'static str {
    if change > 0 {
        "📈"
    } else if change < 0 {
        "📉"
    } else {
        "➡️"
    }
}

fn shorten_stack(stack: &str) -> String {
    let parts: Vec<&str> = stack.split(';').collect();
    if parts.len() <= 2 {
        stack.to_string()
    } else {
        format!("...;{};{}", parts[parts.len() - 2], parts[parts.len() - 1])
    }
}
