/// Direction of optimization for an objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Lower values are better.
    Minimize,
    /// Higher values are better.
    Maximize,
}

/// A named objective with direction and weight.
#[derive(Debug, Clone)]
pub struct Objective {
    /// Name of the objective (e.g., "latency", "throughput").
    pub name: String,
    /// Whether to minimize or maximize this objective.
    pub direction: Direction,
    /// Relative weight of this objective (used for scalarization).
    pub weight: f64,
}

impl Objective {
    /// Create a new objective.
    pub fn new(name: impl Into<String>, direction: Direction, weight: f64) -> Self {
        Self {
            name: name.into(),
            direction,
            weight,
        }
    }

    /// Create a minimize objective with weight 1.0.
    pub fn minimize(name: impl Into<String>) -> Self {
        Self::new(name, Direction::Minimize, 1.0)
    }

    /// Create a maximize objective with weight 1.0.
    pub fn maximize(name: impl Into<String>) -> Self {
        Self::new(name, Direction::Maximize, 1.0)
    }

    /// Create a minimize objective with a custom weight.
    pub fn minimize_weighted(name: impl Into<String>, weight: f64) -> Self {
        Self::new(name, Direction::Minimize, weight)
    }

    /// Create a maximize objective with a custom weight.
    pub fn maximize_weighted(name: impl Into<String>, weight: f64) -> Self {
        Self::new(name, Direction::Maximize, weight)
    }

    /// Normalize a raw value to a "cost" where lower is always better.
    /// For Minimize objectives, returns the value directly.
    /// For Maximize objectives, returns the negation.
    pub fn normalize(&self, value: f64) -> f64 {
        match self.direction {
            Direction::Minimize => value * self.weight,
            Direction::Maximize => -value * self.weight,
        }
    }

    /// Check if a given value is better than another for this objective.
    pub fn is_better(&self, a: f64, b: f64) -> bool {
        match self.direction {
            Direction::Minimize => a < b,
            Direction::Maximize => a > b,
        }
    }
}
