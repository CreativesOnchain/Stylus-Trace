//! SVG flamegraph generation using custom Stylus-optimized logic.
//!
//! Replaces inferno with a manual SVG generator to support:
//! - Custom color coding for Stylus HostIOs (e.g. storage flush = crimson)
//! - Inverted layout (Root at bottom)
//! - Simplified dependency tree

use crate::aggregator::stack_builder::CollapsedStack;
use crate::utils::error::FlamegraphError;
use log::info;
use std::collections::HashMap;


/// Flamegraph configuration
#[derive(Debug, Clone)]
pub struct FlamegraphConfig {
    pub title: String,
    pub width: usize,
    pub ink: bool,
}

impl Default for FlamegraphConfig {
    fn default() -> Self {
        Self {
            title: "Stylus Transaction Profile".to_string(),
            width: 1200,
            ink: false,
        }
    }
}

impl FlamegraphConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_ink(mut self, ink: bool) -> Self {
        self.ink = ink;
        self
    }
}

/// Internal Node structure for building the tree
struct Node {
    name: String,
    value: u64,
    children: HashMap<String, Node>,
}

impl Node {
    fn new(name: String) -> Self {
        Self {
            name,
            value: 0,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, stack: &[&str], value: u64) {
        self.value += value;
        if let Some((head, tail)) = stack.split_first() {
            let child = self
                .children
                .entry(head.to_string())
                .or_insert_with(|| Node::new(head.to_string()));
            child.insert(tail, value);
        }
    }
}

/// Generate SVG flamegraph from collapsed stacks
pub fn generate_flamegraph(
    stacks: &[CollapsedStack],
    config: Option<&FlamegraphConfig>,
) -> Result<String, FlamegraphError> {
    if stacks.is_empty() {
        return Err(FlamegraphError::EmptyStacks);
    }

    let config = config.cloned().unwrap_or_default();
    info!("Generating custom flamegraph with {} stacks", stacks.len());

    // 1. Build Tree
    let mut root = Node::new("root".to_string());
    for stack in stacks {
        // format: "a;b;c" and we have weight separately
        let stack_parts: Vec<&str> = stack.stack.split(';').collect();
        root.insert(&stack_parts, stack.weight);
    }

    // Calculate depth
    let max_depth = calculate_max_depth(&root);
    
    // 2. Render SVG
    let mut svg_content = String::new();
    let width = config.width;
    let height_per_level = 20;
    let graph_height = (max_depth + 1) * height_per_level;
    let legend_height = 80;
    let total_height = graph_height + legend_height;
    
    // Header
    svg_content.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        width, total_height, width, total_height
    ));
    
    // Styles
    svg_content.push_str(
        r#"<style>.func { font: 12px sans-serif; } .func:hover { stroke: black; stroke-width: 1; cursor: pointer; opacity: 0.9; }</style>"#
    );
    
    // Title
    svg_content.push_str(&format!(
        r#"<text x="{}" y="20" font-size="16" text-anchor="middle" font-weight="bold">{}</text>"#,
        width / 2, config.title
    ));
 
    // Render Nodes (Inverted: Root at bottom)
    render_node(
        &root,
        0,
        0.0,
        width as f64,
        &mut svg_content,
        height_per_level,
        graph_height,
    );

    // Render Legend
    render_legend(&mut svg_content, graph_height);

    svg_content.push_str("</svg>");
    
    info!("Flamegraph generated successfully ({} bytes)", svg_content.len());
    Ok(svg_content)
}



fn calculate_max_depth(node: &Node) -> usize {
    if node.children.is_empty() {
        return 0;
    }
    let max_child_depth = node
        .children
        .values()
        .map(calculate_max_depth)
        .max()
        .unwrap_or(0);
    max_child_depth + 1
}

fn get_node_color(name: &str) -> &'static str {
    if name.contains("storage_") {
        if name.contains("flush") {
            "rgb(220, 20, 60)" // Crimson (Expensive!)
        } else if name.contains("load") {
            "rgb(255, 140, 0)" // Dark Orange
        } else {
            "rgb(255, 165, 0)" // Orange
        }
    } else if name.contains("keccak") {
        "rgb(138, 43, 226)" // Blue Violet
    } else if name.contains("memory") 
        || name.contains("read_args") 
        || name.contains("write_result") {
        "rgb(34, 139, 34)" // Forest Green
    } else if name.contains("msg_") 
        || name.contains("call") 
        || name.contains("create") {
        "rgb(70, 130, 180)" // Steel Blue
    } else if name == "root" || name.contains("Stylus") {
        "rgb(100, 149, 237)" // Cornflower Blue
    } else {
        "rgb(169, 169, 169)" // Gray (Generic)
    }
}

fn get_ansi_color(name: &str) -> &'static str {
    if name.contains("storage_") {
        if name.contains("flush") {
            "\x1b[31;1m" // Red/Crimson
        } else {
            "\x1b[33m" // Yellow/Orange
        }
    } else if name.contains("keccak") {
        "\x1b[35m" // Magenta/Violet
    } else if name.contains("memory") || name.contains("read_args") || name.contains("write_result") {
        "\x1b[32m" // Green
    } else if name.contains("msg_") || name.contains("call") || name.contains("create") {
        "\x1b[34m" // Blue
    } else if name == "root" || name.contains("Stylus") {
        "\x1b[36m" // Cyan
    } else {
        "\x1b[90m" // Gray
    }
}

fn render_node(
    node: &Node,
    level: usize,
    x: f64,
    w: f64,
    out: &mut String,
    h: usize,
    graph_height: usize,
) {
    if w < 0.5 {
        return;
    } // Optimization: Don't render invisible blocks

    let color = get_node_color(&node.name);

    // Y position (Inverted: Graph Bottom - (Level * Height))
    // We add margin for title (30px)
    let y = graph_height - ((level + 1) * h) + 30;

    // Draw Rect
    let gas_val = node.value / 10_000;
    out.push_str(&format!(
        r#"<rect x="{:.2}" y="{}" width="{:.2}" height="{}" fill="{}" class="func"><title>{} ({} ink / {} gas)</title></rect>"#,
        x, y, w, h, color, node.name, node.value, gas_val
    ));

    // Draw Text (if wide enough)
    if w > 35.0 {
        // Check if name fits
        let char_width = 7.0;
        let max_chars = (w / char_width) as usize;
        let display_name = if node.name.len() > max_chars && max_chars > 3 {
             format!("{}...", &node.name[0..max_chars - 3])
        } else {
             node.name.clone()
        };
        
        if !display_name.is_empty() {
             out.push_str(&format!(
                r#"<text x="{:.2}" y="{}" dx="4" dy="14" font-size="12" fill="white" pointer-events="none">{}</text>"#,
                x, y, display_name
            ));
        }
    }

    // Recurse
    let mut current_x = x;
    let mut children_vec: Vec<&Node> = node.children.values().collect();
    children_vec.sort_by(|a, b| b.value.cmp(&a.value)); // Sort descending

    for child in children_vec {
        let child_w = (child.value as f64 / node.value as f64) * w;
            render_node(
                child,
                level + 1,
                current_x,
                child_w,
                out,
                h,
                graph_height,
            );
        current_x += child_w;
    }
}

fn render_legend(out: &mut String, graph_height: usize) {
    let legend_y = graph_height + 50;
    
    out.push_str(&format!(
        r#"<text x="10" y="{}" font-size="14" font-weight="bold">Legend:</text>"#, 
        legend_y
    ));

    let items = [
        ("Flush", "rgb(220, 20, 60)"),
        ("Load", "rgb(255, 140, 0)"),
        ("Cache", "rgb(255, 165, 0)"),
        ("Keccak", "rgb(138, 43, 226)"),
        ("Memory", "rgb(34, 139, 34)"),
        ("Call/Msg", "rgb(70, 130, 180)"),
    ];

    for (i, (label, color)) in items.iter().enumerate() {
        let x = 80 + (i * 120);
        out.push_str(&format!(
            r#"<rect x="{}" y="{}" width="15" height="15" fill="{}" rx="2"/>"#,
            x, legend_y - 12, color
        ));
        out.push_str(&format!(
             r#"<text x="{}" y="{}" font-size="12">{}</text>"#,
             x + 20, legend_y, label
        ));
    }
}

/// Create a rich text summary with percentages and table formatting
pub fn generate_text_summary(stacks: &[CollapsedStack], max_lines: usize, _ink_mode: bool, total_execution_gas: u64) -> String {
    let mut lines = Vec::new();
    
    lines.push("  🚀 EXECUTION HOT PATHS".to_string());
    lines.push("  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━┳━━━━━━━━━┓".to_string());
    lines.push(format!("  ┃ {:<42} ┃ {:^12} ┃ {:^12} ┃ {:^7} ┃", "Execution Stack (Hottest First)", "GAS", "INK (x10k)", "%" ));
    lines.push("  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━╋━━━━━━━━━┫".to_string());

    let total_gas = total_execution_gas.max(1);

    for stack in stacks.iter().take(max_lines) {
        let weight_ink = stack.weight;
        let weight_gas = stack.weight / 10_000;
        let percentage = (stack.weight as f64 / total_gas as f64) * 100.0;
        
        let op_name = stack.stack.split(';').next_back().unwrap_or(&stack.stack);
        let color = get_ansi_color(op_name);
        let reset = "\x1b[0m";

        // Truncate stack if too long for display
        let display_stack = if stack.stack.len() > 40 {
            format!("...{}", &stack.stack[stack.stack.len() - 37..])
        } else {
            stack.stack.clone()
        };

        lines.push(format!(
            "  ┃ {}{:<42}{} ┃ {:>12} ┃ {:>12} ┃ {:>6.1}% ┃",
            color, display_stack, reset, weight_gas, weight_ink, percentage
        ));
    }
    
    lines.push("  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━┻━━━━━━━━━┛".to_string());
    
    // Add Simplified Flamegraph section
    lines.push("".to_string());
    lines.push("  🔥 SIMPLIFIED FLAMEGRAPH".to_string());
    lines.push("  root ██████████████████████████████████████████████████ 100%".to_string());
    
    for stack in stacks.iter().take(5) {
        let percentage = (stack.weight as f64 / total_gas as f64) * 100.0;
        let bar_width = (percentage / 2.0) as usize; // Max 50 chars
        let bar = "█".repeat(bar_width);
        
        let op_name = stack.stack.split(';').next_back().unwrap_or(&stack.stack);
        let color = get_ansi_color(op_name);
        let reset = "\x1b[0m";
        
        lines.push(format!(
            "  └─ {}{:<20}{} {}{:50}{} {:>5.1}%",
            color, op_name, reset, color, bar, reset, percentage
        ));
    }

    if stacks.len() > max_lines {
        lines.push("".to_string());
        lines.push(format!("   (Showing top {} of {} unique paths)", max_lines, stacks.len()));
    }
    
    lines.join("\n")
}