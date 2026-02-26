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
    out.push_str(&render_insights(report));
    out.push_str(&render_status(report));

    out
}

fn render_insights(report: &DiffReport) -> String {
    let mut out = String::new();

    if !report.insights.is_empty() {
        out.push_str("\n💡 ");
        out.push_str(&"Optimization Insights:".bold().to_string());
        out.push('\n');

        for insight in &report.insights {
            let color_desc = match insight.severity {
                super::schema::InsightSeverity::High => insight.description.red().bold(),
                super::schema::InsightSeverity::Medium => insight.description.yellow().bold(),
                super::schema::InsightSeverity::Low => insight.description.cyan(),
                super::schema::InsightSeverity::Info => insight.description.normal(),
            };

            out.push_str(&format!(
                "  • [{}] {}\n",
                insight.category.blue(),
                color_desc
            ));
        }
    }
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
        out.push_str(&render_hot_path_comparison_table(report));
    }
    out
}

fn render_hot_path_comparison_table(report: &DiffReport) -> String {
    let mut out = String::new();
    let hot_paths = &report.deltas.hot_paths;

    out.push_str("\n  🚀 HOT PATH COMPARISON\n");
    out.push_str(
        "  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━┳━━━━━━━━━━━━┓\n",
    );
    out.push_str(&format!(
        "  ┃ {:<38} ┃ {:^12} ┃ {:^12} ┃ {:^10} ┃\n",
        "Execution Stack (Common Changes)", "BASELINE", "TARGET", "DELTA"
    ));
    out.push_str(
        "  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━╋━━━━━━━━━━━━┫\n",
    );

    let mut hp_changes = hot_paths.common_paths.clone();
    hp_changes.sort_by(|a, b| b.gas_change.abs().cmp(&a.gas_change.abs()));

    for hp in hp_changes.iter().take(10) {
        let delta_color = if hp.gas_change > 0 {
            "\x1b[31;1m" // Bold Red
        } else if hp.gas_change < 0 {
            "\x1b[32;1m" // Bold Green
        } else {
            "\x1b[0m" // Reset
        };
        let reset = "\x1b[0m";

        let display_stack = shorten_stack(&hp.stack);
        let display_stack_fixed = if display_stack.len() > 38 {
            format!("...{}", &display_stack[display_stack.len() - 35..])
        } else {
            format!("{:<38}", display_stack)
        };

        // Scale to Gas (ink / 10,000)
        let baseline_gas = hp.baseline_gas / 10_000;
        let target_gas = hp.target_gas / 10_000;

        out.push_str(&format!(
            "  ┃ {} ┃ {:>12} ┃ {:>12} ┃ {}{:>9.2}%{} ┃\n",
            display_stack_fixed, baseline_gas, target_gas, delta_color, hp.percent_change, reset
        ));
    }

    out.push_str(
        "  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━┻━━━━━━━━━━━━┛\n",
    );

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
