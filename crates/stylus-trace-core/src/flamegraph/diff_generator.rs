//! SVG flamegraph generation for diffing two profiles.
//!
//! Colors nodes based on gas change:
//! - Red: Regression (Target > Baseline)
//! - Green: Improvement (Target < Baseline)
//! - Gray/Yellow: No change

use crate::aggregator::stack_builder::CollapsedStack;
use crate::flamegraph::generator::{get_truncated_name, FlamegraphConfig};
use crate::utils::error::FlamegraphError;
use log::info;
use std::collections::HashMap;

/// Internal DiffNode structure for building the merged tree
struct DiffNode {
    name: String,
    baseline_value: u64,
    target_value: u64,
    children: HashMap<String, DiffNode>,
}

impl DiffNode {
    fn new(name: String) -> Self {
        Self {
            name,
            baseline_value: 0,
            target_value: 0,
            children: HashMap::new(),
        }
    }

    fn insert_baseline(&mut self, stack: &[&str], value: u64) {
        self.baseline_value += value;
        if let Some((head, tail)) = stack.split_first() {
            let child = self
                .children
                .entry(head.to_string())
                .or_insert_with(|| DiffNode::new(head.to_string()));
            child.insert_baseline(tail, value);
        }
    }

    fn insert_target(&mut self, stack: &[&str], value: u64) {
        self.target_value += value;
        if let Some((head, tail)) = stack.split_first() {
            let child = self
                .children
                .entry(head.to_string())
                .or_insert_with(|| DiffNode::new(head.to_string()));
            child.insert_target(tail, value);
        }
    }
}

/// Generate a comparison SVG flamegraph
pub fn generate_diff_flamegraph(
    baseline_stacks: &[CollapsedStack],
    target_stacks: &[CollapsedStack],
    config: Option<&FlamegraphConfig>,
) -> Result<String, FlamegraphError> {
    info!(
        "Generating diff flamegraph (B:{} stacks, T:{} stacks)",
        baseline_stacks.len(),
        target_stacks.len()
    );

    let config = config.cloned().unwrap_or_default();
    let mut root = DiffNode::new("root".to_string());

    // 1. Build Merged Tree
    for stack in baseline_stacks {
        let mut parts: Vec<&str> = stack.stack.split(';').collect();
        // Skip redundant root if present
        if parts.first() == Some(&"root") {
            parts.remove(0);
        }
        root.insert_baseline(&parts, stack.weight);
    }
    for stack in target_stacks {
        let mut parts: Vec<&str> = stack.stack.split(';').collect();
        // Skip redundant root if present
        if parts.first() == Some(&"root") {
            parts.remove(0);
        }
        root.insert_target(&parts, stack.weight);
    }

    let max_depth = calculate_max_depth(&root);

    // 2. Render SVG
    let mut svg = String::new();
    let width = config.width;
    let height_per_level = 20;
    let graph_height = (max_depth + 1) * height_per_level;
    let legend_height = 80;
    let total_height = graph_height + legend_height + 40;

    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        width, total_height, width, total_height
    ));

    svg.push_str(
        r#"<style>.func { font: 12px sans-serif; } .func:hover { stroke: black; stroke-width: 1; cursor: pointer; opacity: 0.9; }</style>"#
    );

    svg.push_str(&format!(
        r#"<text x="{}" y="25" font-size="18" text-anchor="middle" font-weight="bold">{} (Diff)</text>"#,
        width / 2,
        config.title
    ));

    let mut ctx = DiffRenderContext {
        output: &mut svg,
        line_height: height_per_level,
        graph_height,
    };

    render_diff_node(&root, 0, 0.0, width as f64, &mut ctx);

    render_diff_legend(&mut svg, graph_height + 40);

    svg.push_str("</svg>");
    Ok(svg)
}

fn calculate_max_depth(node: &DiffNode) -> usize {
    if node.children.is_empty() {
        return 0;
    }
    node.children
        .values()
        .map(calculate_max_depth)
        .max()
        .unwrap_or(0)
        + 1
}

struct DiffRenderContext<'a> {
    output: &'a mut String,
    line_height: usize,
    graph_height: usize,
}

fn render_diff_node(node: &DiffNode, level: usize, x: f64, w: f64, ctx: &mut DiffRenderContext) {
    if w < 0.5 {
        return;
    }

    let color = get_diff_color(node.baseline_value, node.target_value);
    let y = (ctx.graph_height as f64)
        - (level as f64 * ctx.line_height as f64)
        - (ctx.line_height as f64)
        + 40.0;

    let tooltip = format_diff_tooltip(node);

    ctx.output.push_str(&format!(
        r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{}" fill="{}" stroke="white" stroke-width="0.5" class="func">"#,
        x, y, w, ctx.line_height, color
    ));
    ctx.output
        .push_str(&format!(r#"<title>{}</title></rect>"#, tooltip));

    if let Some(display_name) = get_truncated_name(&node.name, w) {
        ctx.output.push_str(&format!(
            r#"<text x="{:.2}" y="{:.2}" dx="4" dy="14" font-size="12" fill="black" style="pointer-events:none">{}</text>"#,
            x, y, display_name
        ));
    }

    // Children: Recurse using target width as primary, but if target is 0, use baseline width to show it disappeared
    let mut current_x = x;
    let mut children_vec: Vec<&DiffNode> = node.children.values().collect();
    children_vec.sort_by(|a, b| {
        let a_max = a.target_value.max(a.baseline_value);
        let b_max = b.target_value.max(b.baseline_value);
        b_max.cmp(&a_max)
    });

    let parent_max = node.target_value.max(node.baseline_value);

    for child in children_vec {
        let child_max = child.target_value.max(child.baseline_value);
        let child_w = (child_max as f64 / parent_max as f64) * w;
        if child_w > 0.0 {
            render_diff_node(child, level + 1, current_x, child_w, ctx);
            current_x += child_w;
        }
    }
}

fn get_diff_color(baseline: u64, target: u64) -> String {
    if baseline == 0 && target == 0 {
        return "rgb(200, 200, 200)".into();
    }
    if baseline == 0 {
        return "rgb(255, 100, 100)".into();
    } // New code (Red)
    if target == 0 {
        return "rgb(100, 255, 100)".into();
    } // Removed code (Green)

    let change = (target as f64 - baseline as f64) / baseline as f64;

    if change > 0.01 {
        // Red scale for regressions
        let intensity = (change * 100.0).min(155.0) as u8;
        format!("rgb(255, {}, {})", 200 - intensity, 200 - intensity)
    } else if change < -0.01 {
        // Green scale for improvements
        let intensity = (change.abs() * 100.0).min(155.0) as u8;
        format!("rgb({}, 255, {})", 200 - intensity, 200 - intensity)
    } else {
        "rgb(240, 240, 240)".into() // Stable
    }
}

fn format_diff_tooltip(node: &DiffNode) -> String {
    let baseline = node.baseline_value;
    let target = node.target_value;

    if baseline == 0 {
        return format!("{}: {} (NEW)", node.name, target);
    }
    if target == 0 {
        return format!("{}: {} (REMOVED)", node.name, baseline);
    }

    let diff = target as i64 - baseline as i64;
    let percent = (diff as f64 / baseline as f64) * 100.0;

    format!(
        "{}: {} -> {} ({:+.2}%)",
        node.name, baseline, target, percent
    )
}

fn render_diff_legend(out: &mut String, y: usize) {
    let items = [
        ("Regression", "rgb(255, 100, 100)"),
        ("Improvement", "rgb(100, 255, 100)"),
        ("No Change", "rgb(240, 240, 240)"),
    ];

    for (i, (label, color)) in items.iter().enumerate() {
        let x = 10 + (i * 150);
        out.push_str(&format!(
            r##"<rect x="{}" y="{}" width="15" height="15" fill="{}" stroke="#999999" rx="2"/>"##,
            x,
            y - 12,
            color
        ));
        out.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="12">{}</text>"##,
            x + 20,
            y,
            label
        ));
    }
}
