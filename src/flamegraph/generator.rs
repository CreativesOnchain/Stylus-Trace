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
}

impl Default for FlamegraphConfig {
    fn default() -> Self {
        Self {
            title: "Stylus Transaction Profile".to_string(),
            width: 1200,
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

    // Custom Color Logic (Ported from Stylus Studio)
    let color = if node.name.contains("storage_") {
        if node.name.contains("flush") {
            "rgb(220, 20, 60)" // Crimson (Expensive!)
        } else if node.name.contains("load") {
            "rgb(255, 140, 0)" // Dark Orange
        } else {
            "rgb(255, 165, 0)" // Orange
        }
    } else if node.name.contains("keccak") {
        "rgb(138, 43, 226)" // Blue Violet
    } else if node.name.contains("memory") 
        || node.name.contains("read_args") 
        || node.name.contains("write_result") {
        "rgb(34, 139, 34)" // Forest Green
    } else if node.name.contains("msg_") 
        || node.name.contains("call") 
        || node.name.contains("create") {
        "rgb(70, 130, 180)" // Steel Blue
    } else if node.name == "root" || node.name.contains("Stylus") {
        "rgb(100, 149, 237)" // Cornflower Blue
    } else {
        "rgb(169, 169, 169)" // Gray (Generic)
    };

    // Y position (Inverted: Graph Bottom - (Level * Height))
    // We add margin for title (30px)
    let y = graph_height - ((level + 1) * h) + 30;

    // Draw Rect
    out.push_str(&format!(
        r#"<rect x="{:.2}" y="{}" width="{:.2}" height="{}" fill="{}" class="func"><title>{} ({} gas)</title></rect>"#,
        x, y, w, h, color, node.name, node.value
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

/// Create a text summary (unchanged functionality, simplified impl)
pub fn generate_text_summary(stacks: &[CollapsedStack], max_lines: usize) -> String {
    let mut lines = Vec::new();
    lines.push("Top Gas Consumers:".to_string());
    lines.push("─".repeat(40));

    for (i, stack) in stacks.iter().take(max_lines).enumerate() {
        lines.push(format!(
            "{:>2}. {:>10} gas | {}",
            i + 1, stack.weight, stack.stack
        ));
    }
    
    if stacks.len() > max_lines {
        lines.push(format!("... and {} more", stacks.len() - max_lines));
    }
    
    lines.join("\n")
}