use crate::aggregator::stack_builder::CollapsedStack;
use crate::diff::schema::{AnalysisInsight, InsightSeverity};
use crate::parser::schema::Profile;
use std::collections::HashMap;

/// Analyze a profile for qualitative insights
pub fn analyze_profile(target: &Profile) -> Vec<AnalysisInsight> {
    let mut insights = Vec::new();
    let stacks = target.all_stacks.as_deref().unwrap_or(&[]);

    // Heuristic 1: Redundant HostIO Detection (using total counts)
    detect_redundant_host_calls(target, &mut insights);

    // Heuristic 2: Cold/Warm Storage Tax Analysis (using stack weights)
    analyze_storage_tax(stacks, target.total_gas, &mut insights);

    insights
}

fn detect_redundant_host_calls(profile: &Profile, insights: &mut Vec<AnalysisInsight>) {
    let hostio_labels = [
        ("msg_sender", "msg_sender"),
        ("msg_value", "msg_value"),
        ("read_args", "read_args"),
        ("block_hash", "block_hash"),
        ("account_balance", "account_balance"),
    ];

    let stacks = profile.all_stacks.as_deref().unwrap_or(&[]);

    for (label, stats_key) in hostio_labels {
        let stats = collect_stack_stats(stacks, label);
        let total_calls = profile
            .hostio_summary
            .by_type
            .get(stats_key)
            .cloned()
            .unwrap_or(0);

        if total_calls > 5 {
            let gas_impact_pct = if profile.total_gas > 0 {
                (stats.total_weight as f64 / profile.total_gas as f64) * 100.0
            } else {
                0.0
            };

            insights.push(AnalysisInsight {
                category: "HostIO".to_string(),
                description: format_redundancy_description(
                    label,
                    total_calls,
                    stats.unique_stacks,
                    gas_impact_pct,
                ),
                severity: calculate_insight_severity(total_calls, gas_impact_pct),
                tag: Some("redundant_call".to_string()),
            });
        }
    }
}

struct StackStats {
    unique_stacks: usize,
    total_weight: u64,
}

fn collect_stack_stats(stacks: &[CollapsedStack], label: &str) -> StackStats {
    let mut occurrences_by_stack: HashMap<&str, u64> = HashMap::new();
    let mut total_weight = 0;

    for stack in stacks {
        if stack.stack.contains(label) {
            *occurrences_by_stack.entry(&stack.stack).or_insert(0) += 1;
            total_weight += stack.weight;
        }
    }

    StackStats {
        unique_stacks: occurrences_by_stack.len(),
        total_weight,
    }
}

fn calculate_insight_severity(total_calls: u64, gas_impact_pct: f64) -> InsightSeverity {
    if gas_impact_pct > 5.0 || total_calls > 20 {
        InsightSeverity::High
    } else if gas_impact_pct > 1.0 || total_calls > 10 {
        InsightSeverity::Medium
    } else {
        InsightSeverity::Low
    }
}

fn format_redundancy_description(
    label: &str,
    total_calls: u64,
    unique_stacks: usize,
    gas_impact_pct: f64,
) -> String {
    if unique_stacks == 1 {
        format!(
            "Loop-based redundancy: `{}` called {} times from a single location ({:.2}% total gas). Cache the result before the loop.",
            label, total_calls, gas_impact_pct
        )
    } else {
        format!(
            "Cross-stack redundancy: `{}` called {} times across {} locations ({:.2}% total gas). Consider a shared state or variable.",
            label, total_calls, unique_stacks, gas_impact_pct
        )
    }
}

/// Analyzes storage gas costs to identify cold vs warm reads
fn analyze_storage_tax(
    stacks: &[CollapsedStack],
    total_gas: u64,
    insights: &mut Vec<AnalysisInsight>,
) {
    let stats = collect_storage_stats(stacks);

    let total_storage_gas = stats.cold_read_gas + stats.warm_read_gas + stats.write_gas;
    if total_storage_gas == 0 {
        return;
    }

    let impact_pct = if total_gas > 0 {
        (total_storage_gas as f64 / total_gas as f64) * 100.0
    } else {
        0.0
    };

    generate_cold_tax_insight(&stats, impact_pct, insights);
    generate_write_impact_insight(&stats, total_gas, insights);
}

struct StorageStats {
    cold_read_gas: u64,
    warm_read_gas: u64,
    write_gas: u64,
    cold_count: u64,
}

fn collect_storage_stats(stacks: &[CollapsedStack]) -> StorageStats {
    let mut stats = StorageStats {
        cold_read_gas: 0,
        warm_read_gas: 0,
        write_gas: 0,
        cold_count: 0,
    };

    for stack in stacks {
        if stack.stack.contains("storage_load") {
            if stack.weight >= 2000 {
                stats.cold_read_gas += stack.weight;
                stats.cold_count += 1;
            } else {
                stats.warm_read_gas += stack.weight;
            }
        } else if stack.stack.contains("storage_store") || stack.stack.contains("storage_cache") {
            stats.write_gas += stack.weight;
        }
    }
    stats
}

fn generate_cold_tax_insight(
    stats: &StorageStats,
    impact_pct: f64,
    insights: &mut Vec<AnalysisInsight>,
) {
    let cold_tax_pct = (stats.cold_read_gas as f64
        / (stats.cold_read_gas + stats.warm_read_gas).max(1) as f64)
        * 100.0;

    if stats.cold_count > 0 && cold_tax_pct > 50.0 {
        let severity = if impact_pct > 15.0 {
            InsightSeverity::High
        } else if impact_pct > 5.0 {
            InsightSeverity::Medium
        } else {
            InsightSeverity::Low
        };

        let advice = if stats.cold_count > 3 {
            " Consider packing small variables into single slots or using a more compact data structure."
        } else {
            ""
        };

        let read_suffix = if stats.cold_count == 1 {
            "read"
        } else {
            "reads"
        };

        insights.push(AnalysisInsight {
            category: "Storage".to_string(),
            description: format!(
                "Significant 'Cold Tax': {:.1}% of storage reads are cold, consuming {:.1}% of total gas ({} {}).{}",
                cold_tax_pct, impact_pct, stats.cold_count, read_suffix, advice
            ),
            severity,
            tag: Some("storage_tax".to_string()),
        });
    }
}

fn generate_write_impact_insight(
    stats: &StorageStats,
    total_gas: u64,
    insights: &mut Vec<AnalysisInsight>,
) {
    if stats.write_gas > 5000 && total_gas > 0 {
        let write_impact_pct = (stats.write_gas as f64 / total_gas as f64) * 100.0;
        if write_impact_pct > 10.0 {
            insights.push(AnalysisInsight {
                category: "Storage".to_string(),
                description: format!(
                    "High storage write contribution: Writes account for {:.1}% of total gas. Ensure state updates are minimized.",
                    write_impact_pct
                ),
                severity: InsightSeverity::Medium,
                tag: Some("storage_write_impact".to_string()),
            });
        }
    }
}
