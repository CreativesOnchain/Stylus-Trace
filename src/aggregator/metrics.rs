//! Calculate performance metrics and hot paths from trace data.
//!
//! Hot paths are the execution paths that consume the most gas.
//! These are the primary targets for optimization.

use crate::parser::schema::HotPath;
use super::stack_builder::CollapsedStack;
use log::debug;

/// Calculate hot paths from collapsed stacks
///
/// **Public** - main entry point for metrics calculation
///
/// # Arguments
/// * `stacks` - Collapsed stacks from stack_builder
/// * `total_gas` - Total gas used by transaction
/// * `top_n` - Number of top paths to return (e.g., 10)
///
/// # Returns
/// Vector of hot paths, sorted by gas consumption (descending)
pub fn calculate_hot_paths(
    stacks: &[CollapsedStack],
    total_gas: u64,
    top_n: usize,
) -> Vec<HotPath> {
    debug!("Calculating top {} hot paths from {} stacks", top_n, stacks.len());
    
    // Stacks are already sorted by weight from stack_builder
    // Just take the top N and convert to HotPath format
    stacks
        .iter()
        .take(top_n)
        .map(|stack| create_hot_path(stack, total_gas))
        .collect()
}

/// Create a HotPath from a CollapsedStack
///
/// **Private** - internal conversion
fn create_hot_path(stack: &CollapsedStack, total_gas: u64) -> HotPath {
    // Calculate percentage of total gas
    let percentage = if total_gas > 0 {
        (stack.weight as f64 / total_gas as f64) * 100.0
    } else {
        0.0
    };
    
    HotPath {
        stack: stack.stack.clone(),
        gas: stack.weight,
        percentage,
        source_hint: None, // Will be populated in Milestone 3
    }
}

/// Calculate gas distribution statistics
///
/// **Public** - provides summary statistics
///
/// # Arguments
/// * `stacks` - Collapsed stacks
///
/// # Returns
/// Statistics about gas distribution
pub fn calculate_gas_distribution(stacks: &[CollapsedStack]) -> GasDistribution {
    if stacks.is_empty() {
        return GasDistribution::default();
    }
    
    let total: u64 = stacks.iter().map(|s| s.weight).sum();
    let count = stacks.len();
    let mean = total / count.max(1) as u64;
    
    // Get median
    let mut weights: Vec<u64> = stacks.iter().map(|s| s.weight).collect();
    weights.sort_unstable();
    let median = if weights.is_empty() {
        0
    } else {
        weights[weights.len() / 2]
    };
    
    // Top 10% of stacks
    let top_10_percent_count = (count as f64 * 0.1).ceil() as usize;
    let top_10_percent_gas: u64 = stacks
        .iter()
        .take(top_10_percent_count)
        .map(|s| s.weight)
        .sum();
    
    GasDistribution {
        total_gas: total,
        stack_count: count,
        mean_gas_per_stack: mean,
        median_gas_per_stack: median,
        top_10_percent_gas,
        top_10_percent_percentage: if total > 0 {
            (top_10_percent_gas as f64 / total as f64) * 100.0
        } else {
            0.0
        },
    }
}

/// Gas distribution statistics
///
/// **Public** - returned from calculate_gas_distribution
#[derive(Debug, Clone)]
pub struct GasDistribution {
    /// Total gas across all stacks
    pub total_gas: u64,
    
    /// Number of unique stacks
    pub stack_count: usize,
    
    /// Mean gas per stack
    pub mean_gas_per_stack: u64,
    
    /// Median gas per stack
    pub median_gas_per_stack: u64,
    
    /// Gas consumed by top 10% of stacks
    pub top_10_percent_gas: u64,
    
    /// Percentage of total gas in top 10%
    pub top_10_percent_percentage: f64,
}

impl Default for GasDistribution {
    fn default() -> Self {
        Self {
            total_gas: 0,
            stack_count: 0,
            mean_gas_per_stack: 0,
            median_gas_per_stack: 0,
            top_10_percent_gas: 0,
            top_10_percent_percentage: 0.0,
        }
    }
}

impl GasDistribution {
    /// Check if gas distribution is highly concentrated
    ///
    /// **Public** - useful for identifying optimization opportunities
    ///
    /// Returns true if top 10% of stacks consume >80% of gas
    pub fn is_highly_concentrated(&self) -> bool {
        self.top_10_percent_percentage > 80.0
    }
    
    /// Get human-readable summary
    ///
    /// **Public** - for logging and debugging
    pub fn summary(&self) -> String {
        format!(
            "Total: {} gas | Stacks: {} | Mean: {} | Median: {} | Top 10%: {:.1}%",
            self.total_gas,
            self.stack_count,
            self.mean_gas_per_stack,
            self.median_gas_per_stack,
            self.top_10_percent_percentage
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregator::stack_builder::CollapsedStack;

    #[test]
    fn test_calculate_hot_paths() {
        let stacks = vec![
            CollapsedStack::new("main;execute".to_string(), 5000),
            CollapsedStack::new("main;storage".to_string(), 3000),
            CollapsedStack::new("main;compute".to_string(), 2000),
        ];
        
        let hot_paths = calculate_hot_paths(&stacks, 10000, 2);
        
        assert_eq!(hot_paths.len(), 2);
        assert_eq!(hot_paths[0].stack, "main;execute");
        assert_eq!(hot_paths[0].gas, 5000);
        assert_eq!(hot_paths[0].percentage, 50.0);
    }

    #[test]
    fn test_calculate_gas_distribution() {
        let stacks = vec![
            CollapsedStack::new("stack1".to_string(), 8000),
            CollapsedStack::new("stack2".to_string(), 1000),
            CollapsedStack::new("stack3".to_string(), 500),
            CollapsedStack::new("stack4".to_string(), 500),
        ];
        
        let dist = calculate_gas_distribution(&stacks);
        
        assert_eq!(dist.total_gas, 10000);
        assert_eq!(dist.stack_count, 4);
        assert_eq!(dist.mean_gas_per_stack, 2500);
        assert!(dist.is_highly_concentrated()); // Top stack has 80%
    }

    #[test]
    fn test_gas_distribution_empty() {
        let stacks: Vec<CollapsedStack> = vec![];
        let dist = calculate_gas_distribution(&stacks);
        assert_eq!(dist.total_gas, 0);
        assert_eq!(dist.stack_count, 0);
    }

    #[test]
    fn test_create_hot_path() {
        let stack = CollapsedStack::new("test;path".to_string(), 2500);
        let hot_path = create_hot_path(&stack, 10000);
        
        assert_eq!(hot_path.stack, "test;path");
        assert_eq!(hot_path.gas, 2500);
        assert_eq!(hot_path.percentage, 25.0);
        assert!(hot_path.source_hint.is_none());
    }
}