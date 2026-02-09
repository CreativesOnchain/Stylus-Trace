//! Main trace parser for stylusTracer output.
//!
//! Parses raw JSON from debug_traceTransaction into structured data.
//! Handles schema validation and extraction of execution steps.

use super::hostio::{extract_hostio_events, HostIoStats};
use super::schema::Profile;
use crate::utils::config::{
    GAS_FIELD_NAMES, GAS_TO_INK_MULTIPLIER, MAX_REASONABLE_GAS, SCHEMA_VERSION, STEP_FIELD_NAMES,
};
use crate::utils::error::ParseError;
use log::{debug, warn};
use serde::Deserialize;

/// Detected trace format from RPC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceFormat {
    /// Standard EVM trace with structLogs/gasUsed
    StandardEvm,
    /// Stylus tracer format (array of steps with ink values)
    StylusTracer,
}

/// Raw execution step from stylusTracer
///
/// This represents a single step in the WASM execution.
/// The exact fields depend on the stylusTracer implementation.
/// Raw execution step from stylusTracer or standard EVM tracer
#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionStep {
    /// Gas cost of this operation (in Ink if stylusTracer, or EVM gas if standard)
    #[serde(default, alias = "gasCost")]
    pub gas_cost: u64,

    /// Operation name
    #[serde(default, alias = "name")]
    pub op: Option<String>,

    /// Stack depth
    #[serde(default)]
    pub depth: u32,

    /// Function name (if debug symbols present)
    #[serde(default)]
    pub function: Option<String>,

    /// Start Ink (specific to stylusTracer)
    #[serde(default, rename = "startInk")]
    pub start_ink: Option<u64>,

    /// End Ink (specific to stylusTracer)
    #[serde(default, rename = "endInk")]
    pub end_ink: Option<u64>,

    /// Program Counter / Offset (needed for source mapping)
    #[serde(default)]
    pub pc: u64,
}

/// Parsed trace data (internal representation)
/// Standardizes all gas/ink values to 10,000x base (Stylus Ink)
#[derive(Debug, Clone)]
pub struct ParsedTrace {
    pub transaction_hash: String,
    pub total_gas_used: u64, // In Ink
    pub execution_steps: Vec<ExecutionStep>,
    pub hostio_stats: HostIoStats,
}

/// Parse raw trace JSON from stylusTracer
///
/// **Public** - main entry point for parsing
///
/// # Arguments
/// * `tx_hash` - Transaction hash being profiled
/// * `raw_trace` - Raw JSON from debug_traceTransaction
///
/// # Returns
/// Parsed trace data ready for aggregation
///
/// # Errors
/// * `ParseError::JsonError` - Invalid JSON structure
/// * `ParseError::InvalidFormat` - Missing required fields
/// * `ParseError::UnsupportedVersion` - Incompatible trace format
pub fn parse_trace(
    tx_hash: &str,
    raw_trace: &serde_json::Value,
) -> Result<ParsedTrace, ParseError> {
    debug!("Parsing trace for transaction: {}", tx_hash);

    // Detect and normalize trace format
    let (trace_obj, format) = detect_trace_format(raw_trace)?;

    // Extract total gas used and normalize to Ink
    let mut total_gas_used = extract_total_gas(&trace_obj)?;
    total_gas_used = normalize_to_ink(total_gas_used, format == TraceFormat::StylusTracer);

    // Extract and process execution steps
    let mut execution_steps = extract_execution_steps(&trace_obj)?;
    process_execution_steps(&mut execution_steps, format);

    // Calculate total gas from steps if not provided
    if total_gas_used == 0 {
        total_gas_used = execution_steps.iter().map(|s| s.gas_cost).sum();
    }

    debug!("Parsed {} execution steps", execution_steps.len());

    // Extract HostIO statistics with fallback detection
    let hostio_stats = extract_or_detect_hostio_stats(raw_trace, &execution_steps, format);

    Ok(ParsedTrace {
        transaction_hash: tx_hash.to_string(),
        total_gas_used,
        execution_steps,
        hostio_stats,
    })
}

/// Detect the trace format and normalize to a standard object structure
///
/// **Private** - internal helper for parse_trace
fn detect_trace_format(
    raw_trace: &serde_json::Value,
) -> Result<(serde_json::Map<String, serde_json::Value>, TraceFormat), ParseError> {
    match raw_trace {
        // Format 1: Direct object (could be either format)
        serde_json::Value::Object(obj) => {
            // Heuristic: If it has "result" array, it's likely Stylus tracer
            let format = if obj.contains_key("result") && obj["result"].is_array() {
                TraceFormat::StylusTracer
            } else {
                TraceFormat::StandardEvm
            };
            Ok((obj.clone(), format))
        }

        // Format 2: Array (typical for stylusTracer result)
        serde_json::Value::Array(_) => {
            debug!("Trace is array format (stylusTracer), wrapping in object");
            let mut wrapper = serde_json::Map::new();
            wrapper.insert("steps".to_string(), raw_trace.clone());
            wrapper.insert("gasUsed".to_string(), serde_json::json!(0));
            Ok((wrapper, TraceFormat::StylusTracer))
        }

        _ => Err(ParseError::InvalidFormat(
            "Trace must be a JSON object or array".to_string(),
        )),
    }
}

/// Normalize gas value to Ink units (10,000x multiplier)
///
/// **Private** - internal helper for parse_trace
fn normalize_to_ink(value: u64, is_already_ink: bool) -> u64 {
    if is_already_ink {
        value
    } else if value < MAX_REASONABLE_GAS {
        // Value is in gas units, convert to ink
        value.saturating_mul(GAS_TO_INK_MULTIPLIER)
    } else {
        // Value is already in ink units (too large to be gas)
        value
    }
}

/// Process execution steps: calculate costs and normalize to Ink
///
/// **Private** - internal helper for parse_trace
fn process_execution_steps(steps: &mut [ExecutionStep], format: TraceFormat) {
    for step in steps {
        // If we have explicit ink values, calculate from those
        if let (Some(start), Some(end)) = (step.start_ink, step.end_ink) {
            step.gas_cost = start.saturating_sub(end);
        } else if format == TraceFormat::StandardEvm {
            // Convert EVM gas to ink
            step.gas_cost = step.gas_cost.saturating_mul(GAS_TO_INK_MULTIPLIER);
        }
        // Otherwise, assume gas_cost is already in ink units
    }
}

/// Extract HostIO statistics, with fallback detection from execution steps
///
/// **Private** - internal helper for parse_trace
fn extract_or_detect_hostio_stats(
    raw_trace: &serde_json::Value,
    execution_steps: &[ExecutionStep],
    format: TraceFormat,
) -> HostIoStats {
    let mut hostio_stats = extract_hostio_events(raw_trace);

    // Fallback: If no HostIOs found explicitly, detect from steps
    if hostio_stats.total_calls() == 0 && !execution_steps.is_empty() {
        debug!("Explicit hostio field missing, detecting from execution steps");
        detect_hostio_from_steps(&mut hostio_stats, execution_steps, format);
    }

    hostio_stats
}

/// Detect HostIO events from execution steps
///
/// **Private** - internal helper for extract_or_detect_hostio_stats
fn detect_hostio_from_steps(
    hostio_stats: &mut HostIoStats,
    execution_steps: &[ExecutionStep],
    format: TraceFormat,
) {
    use super::hostio::{HostIoEvent, HostIoType};

    for step in execution_steps {
        // Priority: op (alias for name) > function > "unknown"
        let op_name = step
            .op
            .as_deref()
            .or(step.function.as_deref())
            .unwrap_or("unknown");

        // Handle formats like "call;SSTORE" - take the last part
        let op_part = op_name.split(';').next_back().unwrap_or(op_name);

        if let Some(io_type) = HostIoType::from_opcode(op_part) {
            hostio_stats.add_event(HostIoEvent {
                io_type,
                gas_cost: step.gas_cost,
            });
        } else if format == TraceFormat::StylusTracer {
            // In stylusTracer, attempt to parse all operations as HostIO
            // This may fail for unknown opcodes, which we silently ignore
            let _ = op_part.parse::<HostIoType>().map(|io_type| {
                hostio_stats.add_event(HostIoEvent {
                    io_type,
                    gas_cost: step.gas_cost,
                });
            });
        }
    }
}

/// Extract total gas used from trace
///
/// **Private** - internal extraction logic
pub fn extract_total_gas(
    trace_obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<u64, ParseError> {
    let gas = GAS_FIELD_NAMES.iter().find_map(|field| {
        trace_obj
            .get(*field)
            .and_then(|val| match parse_json_u64(val) {
                Ok(gas) => Some(gas),
                Err(e) => {
                    warn!("Found gas field '{}' but failed to parse: {}", field, e);
                    None
                }
            })
    });

    if let Some(g) = gas {
        Ok(g)
    } else {
        // If no gas field found, try to calculate from steps
        warn!("No valid gas field found in trace, will calculate from steps");
        Ok(0)
    }
}

/// Helper to parse reaching u64 from various JSON types (number, string)
///
/// **Private** - internal utility
fn parse_json_u64(val: &serde_json::Value) -> Result<u64, ParseError> {
    if let Some(n) = val.as_u64() {
        Ok(n)
    } else if let Some(s) = val.as_str() {
        parse_gas_value(s)
    } else {
        Err(ParseError::InvalidFormat(format!(
            "Expected number or string, found {}",
            val
        )))
    }
}

/// Extract execution steps from trace
///
/// **Private** - internal extraction logic
fn extract_execution_steps(
    trace_obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<ExecutionStep>, ParseError> {
    // Try multiple possible field names
    for field in STEP_FIELD_NAMES {
        if let Some(steps_value) = trace_obj.get(*field) {
            if let Some(steps_array) = steps_value.as_array() {
                return parse_steps_array(steps_array);
            }
        }
    }

    // No steps found - this might be valid for very simple transactions
    warn!("No execution steps found in trace");
    Ok(Vec::new())
}

/// Parse array of execution steps
///
/// **Private** - internal parsing logic
fn parse_steps_array(steps_array: &[serde_json::Value]) -> Result<Vec<ExecutionStep>, ParseError> {
    let mut steps = Vec::with_capacity(steps_array.len());

    for (index, step_value) in steps_array.iter().enumerate() {
        match serde_json::from_value::<ExecutionStep>(step_value.clone()) {
            Ok(step) => steps.push(step),
            Err(e) => {
                // Log but don't fail - some steps may be malformed
                warn!("Failed to parse step {}: {}", index, e);
            }
        }
    }

    if steps.is_empty() && !steps_array.is_empty() {
        return Err(ParseError::InvalidFormat(
            "All execution steps failed to parse".to_string(),
        ));
    }

    Ok(steps)
}

/// Parse gas value from hex string or decimal
///
/// **Private** - internal utility
/// Parse a gas value from hex or decimal string
pub fn parse_gas_value(value: &str) -> Result<u64, ParseError> {
    // Handle hex values (0x prefix)
    if let Some(hex_str) = value.strip_prefix("0x") {
        u64::from_str_radix(hex_str, 16)
            .map_err(|e| ParseError::InvalidFormat(format!("Invalid hex gas value: {}", e)))
    } else {
        // Try parsing as decimal
        value
            .parse::<u64>()
            .map_err(|e| ParseError::InvalidFormat(format!("Invalid decimal gas value: {}", e)))
    }
}

/// Convert parsed trace to output profile format
///
/// **Public** - used by commands to create final output
pub fn to_profile(
    parsed_trace: &ParsedTrace,
    mut hot_paths: Vec<super::schema::HotPath>,
    mapper: Option<&super::source_map::SourceMapper>,
) -> Profile {
    use chrono::Utc;

    // Enrich hot paths with source information if mapper is available
    if let Some(mapper) = mapper {
        enrich_source_hints(&mut hot_paths, mapper);
    }

    Profile {
        version: SCHEMA_VERSION.to_string(),
        transaction_hash: parsed_trace.transaction_hash.clone(),
        total_gas: parsed_trace.total_gas_used,
        hostio_summary: parsed_trace.hostio_stats.to_summary(),
        hot_paths,
        generated_at: Utc::now().to_rfc3339(),
    }
}

/// Enrich hot paths with source-to-line mapping information
///
/// **Private** - internal helper for to_profile
fn enrich_source_hints(
    hot_paths: &mut [super::schema::HotPath],
    mapper: &super::source_map::SourceMapper,
) {
    for path in hot_paths {
        let Some(hint) = &path.source_hint else {
            continue;
        };
        let Some(pc_str) = &hint.function else {
            continue;
        };

        let Ok(pc) = pc_str
            .strip_prefix("0x")
            .and_then(|h| u64::from_str_radix(h, 16).ok())
            .ok_or(())
        else {
            continue;
        };

        if let Some(loc) = mapper.lookup(pc) {
            path.source_hint = Some(super::schema::SourceHint {
                file: loc.file,
                line: loc.line,
                column: loc.column,
                function: loc.function,
            });
        }
    }
}

