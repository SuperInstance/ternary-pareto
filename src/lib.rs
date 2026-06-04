//! # ternary-pareto
//!
//! Pareto optimization for ternary agents — multi-objective strategies where you
//! can't maximize everything at once.
//!
//! This crate provides tools for multi-objective optimization:
//! - Define objectives (minimize or maximize)
//! - Maintain Pareto fronts of non-dominated strategies
//! - Search for Pareto-optimal strategies via evolutionary methods
//! - Compute hypervolume indicators
//! - Analyze tradeoffs between objectives

mod dominance;
mod front;
mod hypervolume;
mod objective;
mod optimizer;
mod tradeoff;

pub use dominance::{Dominance, DominanceRelation};
pub use front::ParetoFront;
pub use hypervolume::HypervolumeIndicator;
pub use objective::{Direction, Objective};
pub use optimizer::{ParetoOptimizer, SearchConfig};
pub use tradeoff::TradeoffAnalyzer;

/// A strategy maps objective names to their measured values.
pub type Strategy = std::collections::HashMap<String, f64>;

/// A scored strategy with its objective evaluations and metadata.
#[derive(Debug, Clone)]
pub struct ScoredStrategy {
    /// The strategy's objective evaluations (objective name → value).
    pub values: Strategy,
    /// Optional label for this strategy.
    pub label: Option<String>,
}

impl ScoredStrategy {
    /// Create a new scored strategy from objective values.
    pub fn new(values: Strategy) -> Self {
        Self {
            values,
            label: None,
        }
    }

    /// Create a scored strategy with a label.
    pub fn labeled(values: Strategy, label: impl Into<String>) -> Self {
        Self {
            values,
            label: Some(label.into()),
        }
    }

    /// Get the value for a specific objective.
    pub fn get(&self, objective_name: &str) -> Option<f64> {
        self.values.get(objective_name).copied()
    }
}
