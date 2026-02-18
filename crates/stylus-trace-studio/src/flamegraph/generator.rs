//! SVG flamegraph generation using custom Stylus-optimized logic.
//!
//! Replaces inferno with a manual SVG generator to support:
//! - Custom color coding for Stylus HostIOs (e.g. storage flush = crimson)
//! - Inverted layout (Root at bottom)
//! - Simplified dependency tree

use crate::aggregator::stack_builder::CollapsedStack;
use crate::parser::source_map::SourceMapper;
use crate::parser::HostIoType;
use crate::utils::error::FlamegraphError;
use log::info;
use std::collections::HashMap;

/// Categories for flamegraph nodes to determine colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    StorageExpensive,
    StorageNormal,
    Crypto,
    Memory,
    Call,
    System,
    UserCode,
    Root,
}

impl NodeCategory {
    /// Classify a node based on its name (used as fallback or for user code)
    pub fn from_name(name: &str) -> Self {
        if name == "root" {
            return Self::Root;
        }

        // Try structured signal first (HostIoType enum)
        let io_type = name.parse::<HostIoType>().unwrap_or(HostIoType::Other);
        if io_type != HostIoType::Other {
            return Self::from_hostio(io_type);
        }

        // Fallback for non-HostIO system components or user code
        if name.contains("Stylus") || name.contains("host") {
            Self::System
        } else {
            Self::UserCode
        }
    }

    /// Map structured HostIoType to a visual category
    pub fn from_hostio(io_type: HostIoType) -> Self {
        match io_type {
            HostIoType::StorageStore | HostIoType::StorageFlush => Self::StorageExpensive,
            HostIoType::StorageLoad | HostIoType::StorageCache => Self::StorageNormal,
            HostIoType::NativeKeccak256 => Self::Crypto,
            HostIoType::ReadArgs | HostIoType::WriteResult => Self::Memory,
            HostIoType::Call
            | HostIoType::StaticCall
            | HostIoType::DelegateCall
            | HostIoType::Create => Self::Call,
            HostIoType::Log
            | HostIoType::AccountBalance
            | HostIoType::BlockHash
            | HostIoType::MsgValue
            | HostIoType::MsgSender
            | HostIoType::MsgReentrant
            | HostIoType::SelfDestruct => Self::System,
            HostIoType::Other => Self::UserCode,
        }
    }
}

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
    pc: Option<u64>,
    category: NodeCategory,
    children: HashMap<String, Node>,
}

impl Node {
    fn new(name: String) -> Self {
        let category = NodeCategory::from_name(&name);
        Self {
            name,
            value: 0,
            pc: None,
            category,
            children: HashMap::new(),
        }
    }

    fn insert(&mut self, stack: &[&str], value: u64, pc: Option<u64>) {
        self.value += value;
        if pc.is_some() {
            self.pc = pc;
        }
        if let Some((head, tail)) = stack.split_first() {
            let child = self
                .children
                .entry(head.to_string())
                .or_insert_with(|| Node::new(head.to_string()));
            child.insert(tail, value, pc);
        }
    }
}

/// Generate SVG flamegraph from collapsed stacks
pub fn generate_flamegraph(
    stacks: &[CollapsedStack],
    config: Option<&FlamegraphConfig>,
    mapper: Option<&SourceMapper>,
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
        root.insert(&stack_parts, stack.weight, stack.last_pc);
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
        width / 2,
        config.title
    ));

    // Render Nodes (Inverted: Root at bottom)
    let mut ctx = RenderContext {
        output: &mut svg_content,
        line_height: height_per_level,
        graph_height,
        mapper,
    };

    render_node(&root, 0, 0.0, width as f64, &mut ctx);

    // Render Legend
    render_legend(&mut svg_content, graph_height);

    svg_content.push_str("</svg>");

    info!(
        "Flamegraph generated successfully ({} bytes)",
        svg_content.len()
    );
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

fn get_node_color(category: NodeCategory) -> &'static str {
    match category {
        NodeCategory::StorageExpensive => "rgb(220, 20, 60)", // Crimson
        NodeCategory::StorageNormal => "rgb(255, 140, 0)",    // Dark Orange
        NodeCategory::Crypto => "rgb(138, 43, 226)",          // Blue Violet
        NodeCategory::Memory => "rgb(34, 139, 34)",           // Forest Green
        NodeCategory::Call => "rgb(70, 130, 180)",            // Steel Blue
        NodeCategory::System => "rgb(100, 149, 237)",         // Cornflower Blue
        NodeCategory::Root => "rgb(75, 0, 130)",              // Indigo
        NodeCategory::UserCode => "rgb(169, 169, 169)",       // Gray
    }
}

fn get_ansi_color(category: NodeCategory) -> &'static str {
    match category {
        NodeCategory::StorageExpensive => "\x1b[31;1m", // Bold Red
        NodeCategory::StorageNormal => "\x1b[33m",      // Yellow
        NodeCategory::Crypto => "\x1b[35m",             // Magenta
        NodeCategory::Memory => "\x1b[32m",             // Green
        NodeCategory::Call => "\x1b[34m",               // Blue
        NodeCategory::System => "\x1b[36m",             // Cyan
        NodeCategory::Root => "\x1b[37;1m",             // Bold White
        NodeCategory::UserCode => "\x1b[90m",           // Gray
    }
}

struct RenderContext<'a> {
    output: &'a mut String,
    line_height: usize,
    graph_height: usize,
    mapper: Option<&'a SourceMapper>,
}

fn render_node(node: &Node, level: usize, x: f64, w: f64, ctx: &mut RenderContext) {
    if w < 0.5 {
        return;
    } // Optimization: Don't render invisible blocks

    let color = get_node_color(node.category);

    // Y position (Inverted: Graph Bottom - (Level * Height))
    // We add margin for title (30px)
    let y = (ctx.graph_height as f64)
        - (level as f64 * ctx.line_height as f64)
        - (ctx.line_height as f64)
        + 30.0;

    let tooltip = format_tooltip(node, ctx);

    ctx.output.push_str(&format!(
        r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{}" fill="{}" stroke="white" stroke-width="0.5" class="func">"#,
        x, y, w, ctx.line_height, color
    ));
    ctx.output
        .push_str(&format!(r#"<title>{}</title></rect>"#, tooltip));

    if let Some(display_name) = get_truncated_name(&node.name, w) {
        ctx.output.push_str(&format!(
            r#"<text x="{:.2}" y="{:.2}" dx="4" dy="14" font-size="12" fill="white" pointer-events="none">{}</text>"#,
            x, y, display_name
        ));
    }

    // Recurse
    let mut current_x = x;
    let mut children_vec: Vec<&Node> = node.children.values().collect();
    children_vec.sort_by(|a, b| b.value.cmp(&a.value)); // Sort descending

    for child in children_vec {
        let child_w = (child.value as f64 / node.value as f64) * w;
        if child_w > 0.0 {
            render_node(child, level + 1, current_x, child_w, ctx);
            current_x += child_w;
        }
    }
}

/// Helper to format a rich tooltip for a node
fn format_tooltip(node: &Node, ctx: &RenderContext) -> String {
    let mut tooltip = format!(
        "{}: {} ink / {} gas",
        node.name,
        node.value,
        node.value / 10_000
    );

    if let (Some(pc), Some(mapper)) = (node.pc, ctx.mapper) {
        if let Some(loc) = mapper.lookup(pc) {
            let file_name = loc.file.split('/').next_back().unwrap_or(&loc.file);
            tooltip = format!("{} | {}:{}", tooltip, file_name, loc.line.unwrap_or(0));
        }
    }
    tooltip
}

/// Helper to truncate node names based on available width
/// Calculate truncated name for a node based on width
pub fn get_truncated_name(name: &str, width: f64) -> Option<String> {
    const MIN_LABEL_WIDTH: f64 = 35.0;
    const CHAR_WIDTH: f64 = 7.0;

    if width <= MIN_LABEL_WIDTH {
        return None;
    }

    let max_chars = (width / CHAR_WIDTH) as usize;
    if name.len() > max_chars && max_chars > 3 {
        Some(format!("{}...", &name[0..max_chars.saturating_sub(3)]))
    } else if !name.is_empty() {
        Some(name.to_string())
    } else {
        None
    }
}

fn render_legend(out: &mut String, graph_height: usize) {
    let legend_y = graph_height + 50;

    out.push_str(&format!(
        r#"<text x="10" y="{}" font-size="14" font-weight="bold">Legend:</text>"#,
        legend_y
    ));

    let items = [
        ("Storage (Ex)", "rgb(220, 20, 60)"),
        ("Storage", "rgb(255, 140, 0)"),
        ("Crypto", "rgb(138, 43, 226)"),
        ("Memory", "rgb(34, 139, 34)"),
        ("Call/Msg", "rgb(70, 130, 180)"),
        ("System", "rgb(100, 149, 237)"),
    ];

    for (i, (label, color)) in items.iter().enumerate() {
        let x = 80 + (i * 120);
        out.push_str(&format!(
            r#"<rect x="{}" y="{}" width="15" height="15" fill="{}" rx="2"/>"#,
            x,
            legend_y - 12,
            color
        ));
        out.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="12">{}</text>"#,
            x + 20,
            legend_y,
            label
        ));
    }
}

/// Create a rich text summary with percentages and table formatting
pub fn generate_text_summary(
    hot_paths: &[crate::parser::schema::HotPath],
    max_lines: usize,
    _ink_mode: bool,
) -> String {
    let mut lines = Vec::new();

    lines.extend(render_hot_path_table(hot_paths, max_lines));
    lines.push("".to_string());
    lines.extend(render_ascii_flamegraph(hot_paths));

    if hot_paths.len() > max_lines {
        lines.push("".to_string());
        lines.push(format!(
            "   (Showing top {} of {} unique paths)",
            max_lines,
            hot_paths.len()
        ));
    }

    lines.join("\n")
}

/// Helper to render the hot path table for terminal output
fn render_hot_path_table(
    hot_paths: &[crate::parser::schema::HotPath],
    max_lines: usize,
) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("  🚀 EXECUTION HOT PATHS".to_string());
    lines.push("  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━┳━━━━━━━━━┓".to_string());
    lines.push(format!(
        "  ┃ {:<42} ┃ {:^12} ┃ {:^12} ┃ {:^7} ┃",
        "Execution Stack (Hottest First)", "GAS", "INK (x10k)", "%"
    ));
    lines.push("  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━╋━━━━━━━━━┫".to_string());

    for path in hot_paths.iter().take(max_lines) {
        let weight_ink = path.gas;
        let weight_gas = path.gas / 10_000;
        let percentage = path.percentage;

        let op_name = path.stack.split(';').next_back().unwrap_or(&path.stack);
        let category = NodeCategory::from_name(op_name);
        let color = get_ansi_color(category);
        let reset = "\x1b[0m";

        let display_stack = truncate_stack(&path.stack, 42);

        lines.push(format!(
            "  ┃ {}{:<42}{} ┃ {:>12} ┃ {:>12} ┃ {:>6.1}% ┃",
            color, display_stack, reset, weight_gas, weight_ink, percentage
        ));
    }

    lines.push("  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━┻━━━━━━━━━┛".to_string());
    lines
}

/// Helper to render the ASCII flamegraph visualization
fn render_ascii_flamegraph(hot_paths: &[crate::parser::schema::HotPath]) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("  🔥 SIMPLIFIED FLAMEGRAPH".to_string());
    lines.push("  root ██████████████████████████████████████████████████ 100%".to_string());

    for path in hot_paths.iter().take(5) {
        let percentage = path.percentage;
        let bar_width = (percentage / 2.0) as usize; // Max 50 chars
        let bar = "█".repeat(bar_width);

        let op_name = path.stack.split(';').next_back().unwrap_or(&path.stack);
        let category = NodeCategory::from_name(op_name);
        let color = get_ansi_color(category);
        let reset = "\x1b[0m";

        lines.push(format!(
            "  └─ {}{:<20}{} {}{:50}{} {:>5.1}%",
            color, op_name, reset, color, bar, reset, percentage
        ));
    }
    lines
}

/// Helper to truncate strings with ellipsis for table display
fn truncate_stack(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("...{}", &s[s.len().saturating_sub(max_len - 3)..])
    } else {
        s.to_string()
    }
}
