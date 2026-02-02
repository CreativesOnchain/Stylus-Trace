//! Build collapsed stack format from parsed trace data.
//!
//! Collapsed stacks are the input format for flamegraph generation.
//! Format: "parent;child;grandchild weight"
//!
//! Example: "main;execute_tx;storage_read 1000"
//! This means: main called execute_tx which called storage_read, consuming 1000 gas.

use crate::parser::{ParsedTrace, HostIoType};
use log::debug;
use std::collections::HashMap;

/// A single collapsed stack entry
///
/// **Public** - used by flamegraph generator
#[derive(Debug, Clone)]
pub struct CollapsedStack {
    /// Stack trace as semicolon-separated string
    pub stack: String,
    
    /// Weight (gas consumed by this stack)
    pub weight: u64,
}

impl CollapsedStack {
    /// Create a new collapsed stack
    ///
    /// **Public** - constructor
    pub fn new(stack: String, weight: u64) -> Self {
        Self { stack, weight }
    }
    
}


/// Build collapsed stacks from parsed trace
///
/// **Public** - main entry point for stack building
///
/// # Arguments
/// * `parsed_trace` - Parsed trace data from parser
///
/// # Returns
/// Vector of collapsed stacks, one per unique execution path
///
/// # Algorithm
/// 1. Walk through execution steps
/// 2. Track call stack depth
/// 3. Build stack strings for each gas-consuming operation
/// 4. Aggregate by unique stack (sum weights)
pub fn build_collapsed_stacks(parsed_trace: &ParsedTrace) -> Vec<CollapsedStack> {
    debug!("Building collapsed stacks from {} execution steps", 
           parsed_trace.execution_steps.len());
    
    // Map to aggregate stacks: stack_string -> total_weight
    let mut stack_map: HashMap<String, u64> = HashMap::new();
    
    // Current call stack (tracks function hierarchy)
    let mut call_stack: Vec<String> = Vec::new();

    
    // Process each execution step
    for step in &parsed_trace.execution_steps {
        // Get operation name
        let operation = step.function.as_deref()
            .or(step.op.as_deref())
            .unwrap_or("unknown");
        
        // Handle depth changes properly
        let current_depth = step.depth as usize;
        
        // If depth decreased, we returned from function calls
        if current_depth < call_stack.len() {
            call_stack.truncate(current_depth);
        }
        
        // If depth increased, we entered a new call
        while call_stack.len() < current_depth {
            call_stack.push("call".to_string());
        }
        
        // Build the full stack string with current operation
        let stack_str = if call_stack.is_empty() {
            operation.to_string()
        } else {
            format!("{};{}", call_stack.join(";"), operation)
        };
        
        // FIXED: Always add to map, accumulate all gas costs (even 0)
        *stack_map.entry(stack_str).or_insert(0) += step.gas_cost;
        

    }
    
    // Also add HostIO stacks if we have HostIO events
    add_hostio_stacks(&mut stack_map, parsed_trace);
    
    // Convert map to vector and sort by weight (descending)
    let mut stacks: Vec<CollapsedStack> = stack_map
        .into_iter()
        .map(|(stack, weight)| CollapsedStack::new(stack, weight))
        .collect();
    
    stacks.sort_by(|a, b| b.weight.cmp(&a.weight));
    
    debug!("Built {} unique collapsed stacks", stacks.len());
    
    stacks
}


/// Add HostIO events as separate stacks
///
/// **Private** - internal HostIO stack generation
///
/// HostIO events are important enough to show separately in the flamegraph
fn add_hostio_stacks(
    stack_map: &mut HashMap<String, u64>,
    parsed_trace: &ParsedTrace,
) {
    // Create a synthetic "hostio" root for all HostIO operations
    let hostio_counts = &parsed_trace.hostio_stats;
    
    // For each HostIO type with non-zero count, add a stack
    for hostio_type in [
        HostIoType::StorageLoad,
        HostIoType::StorageStore,
        HostIoType::Call,
        HostIoType::StaticCall,
        HostIoType::DelegateCall,
        HostIoType::Create,
        HostIoType::Log,
        HostIoType::SelfDestruct,
        HostIoType::AccountBalance,
        HostIoType::BlockHash,
        HostIoType::Other,
    ] {
        let count = hostio_counts.count_for_type(hostio_type);
        if count > 0 {
            let type_name = match hostio_type {
                HostIoType::StorageLoad => "storage_load",
                HostIoType::StorageStore => "storage_store",
                HostIoType::StorageFlush => "storage_flush",
                HostIoType::StorageCache => "storage_cache",
                HostIoType::Call => "call",
                HostIoType::StaticCall => "staticcall",
                HostIoType::DelegateCall => "delegatecall",
                HostIoType::Create => "create",
                HostIoType::Log => "log",
                HostIoType::SelfDestruct => "selfdestruct",
                HostIoType::AccountBalance => "balance",
                HostIoType::BlockHash => "blockhash",
                HostIoType::Other => "other",
            };
            let stack_name = format!("hostio;{}", type_name);
            // We don't have per-event gas, so distribute total HostIO gas proportionally
            let weight = (hostio_counts.total_gas() * count) / hostio_counts.total_calls().max(1);
            *stack_map.entry(stack_name).or_insert(0) += weight;
        }
    }
}


#[cfg(test)]
mod tests {
    // use super::*;
/*
    #[test]
    fn test_collapsed_stack_to_line() {
        let stack = CollapsedStack::new("main;execute;storage_read".to_string(), 1000);
        assert_eq!(stack.to_line(), "main;execute;storage_read 1000");
    }
*/


/*
    #[test]
    fn test_merge_small_stacks() {
        let stacks = vec![
            CollapsedStack::new("big_stack".to_string(), 1000),
            CollapsedStack::new("small_stack_1".to_string(), 10),
            CollapsedStack::new("small_stack_2".to_string(), 15),
            CollapsedStack::new("medium_stack".to_string(), 500),
        ];
        
        let merged = merge_small_stacks(stacks, 100);
        
        // Should have: big_stack (1000), medium_stack (500), other (25)
        assert_eq!(merged.len(), 3);
        
        let other = merged.iter().find(|s| s.stack == "other").unwrap();
        assert_eq!(other.weight, 25);
    }
*/
}