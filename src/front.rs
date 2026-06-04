use crate::{DominanceRelation, Objective, ScoredStrategy, Strategy};

/// Maintains the Pareto front of non-dominated strategies.
#[derive(Debug, Clone)]
pub struct ParetoFront {
    objectives: Vec<Objective>,
    dominance: DominanceRelation,
    strategies: Vec<ScoredStrategy>,
}

impl ParetoFront {
    /// Create an empty Pareto front with the given objectives.
    pub fn new(objectives: Vec<Objective>) -> Self {
        let dominance = DominanceRelation::new(objectives.clone());
        Self {
            objectives,
            dominance,
            strategies: Vec::new(),
        }
    }

    /// Get the objectives used by this front.
    pub fn objectives(&self) -> &[Objective] {
        &self.objectives
    }

    /// Get the non-dominated strategies.
    pub fn strategies(&self) -> &[ScoredStrategy] {
        &self.strategies
    }

    /// Number of strategies in the Pareto front.
    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    /// Whether the front is empty.
    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }

    /// Insert a strategy, removing any strategies it dominates.
    /// Returns true if the strategy was added to the front.
    pub fn insert(&mut self, strategy: ScoredStrategy) -> bool {
        // Remove any strategies dominated by the new one
        self.strategies.retain(|existing| {
            !self.dominance.dominates(&strategy.values, &existing.values)
        });

        // Check if any remaining strategy dominates the new one
        for existing in &self.strategies {
            if self.dominance.dominates(&existing.values, &strategy.values) {
                return false;
            }
        }

        self.strategies.push(strategy);
        true
    }

    /// Insert a raw strategy (without label).
    pub fn insert_strategy(&mut self, strategy: Strategy) -> bool {
        self.insert(ScoredStrategy::new(strategy))
    }

    /// Merge another Pareto front into this one.
    /// Returns the number of strategies added.
    pub fn merge(&mut self, other: &ParetoFront) -> usize {
        let mut added = 0;
        for strategy in &other.strategies {
            if self.insert(strategy.clone()) {
                added += 1;
            }
        }
        added
    }

    /// Remove a strategy by index.
    pub fn remove(&mut self, index: usize) -> ScoredStrategy {
        self.strategies.remove(index)
    }

    /// Clear all strategies.
    pub fn clear(&mut self) {
        self.strategies.clear();
    }

    /// Find the strategy that is closest to the ideal point.
    /// The ideal point is the best value in each objective across all strategies.
    pub fn closest_to_ideal(&self) -> Option<&ScoredStrategy> {
        if self.strategies.is_empty() {
            return None;
        }

        // Compute ideal point
        let ideal = self.ideal_point();

        // Find closest by weighted Euclidean distance
        let mut best = &self.strategies[0];
        let mut best_dist = f64::MAX;

        for s in &self.strategies {
            let dist = self.normalized_distance(s, &ideal);
            if dist < best_dist {
                best_dist = dist;
                best = s;
            }
        }

        Some(best)
    }

    /// Compute the ideal point (best value in each objective).
    pub fn ideal_point(&self) -> Strategy {
        let mut ideal = Strategy::new();
        for obj in &self.objectives {
            let best = self
                .strategies
                .iter()
                .filter_map(|s| s.values.get(&obj.name).copied())
                .fold(f64::NAN, |acc, v| {
                    if acc.is_nan() {
                        v
                    } else if obj.is_better(v, acc) {
                        v
                    } else {
                        acc
                    }
                });
            ideal.insert(obj.name.clone(), best);
        }
        ideal
    }

    /// Compute the nadir point (worst value in each objective among front members).
    pub fn nadir_point(&self) -> Strategy {
        let mut nadir = Strategy::new();
        for obj in &self.objectives {
            let worst = self
                .strategies
                .iter()
                .filter_map(|s| s.values.get(&obj.name).copied())
                .fold(f64::NAN, |acc, v| {
                    if acc.is_nan() {
                        v
                    } else if !obj.is_better(v, acc) {
                        v
                    } else {
                        acc
                    }
                });
            nadir.insert(obj.name.clone(), worst);
        }
        nadir
    }

    /// Compute normalized Euclidean distance between a strategy and a reference point.
    fn normalized_distance(&self, strategy: &ScoredStrategy, reference: &Strategy) -> f64 {
        let mut sum = 0.0;
        for obj in &self.objectives {
            let v = strategy.values.get(&obj.name).copied().unwrap_or(0.0);
            let r = reference.get(&obj.name).copied().unwrap_or(0.0);
            // Normalize using objective weight
            let diff = obj.normalize(v) - obj.normalize(r);
            sum += diff * diff;
        }
        sum.sqrt()
    }

    /// Rank strategies by crowding distance (diversity measure).
    /// Returns indices sorted by crowding distance (most diverse first).
    pub fn crowding_distance_ranking(&self) -> Vec<usize> {
        if self.strategies.len() <= 2 {
            return (0..self.strategies.len()).collect();
        }

        let n = self.strategies.len();
        let mut distances = vec![0.0f64; n];

        for obj in &self.objectives {
            // Sort indices by this objective
            let mut indices: Vec<usize> = (0..n).collect();
            indices.sort_by(|&a, &b| {
                let va = self.strategies[a].values.get(&obj.name).copied().unwrap_or(0.0);
                let vb = self.strategies[b].values.get(&obj.name).copied().unwrap_or(0.0);
                va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Boundary points get infinite distance
            distances[indices[0]] = f64::INFINITY;
            distances[indices[n - 1]] = f64::INFINITY;

            // Compute range
            let f_min = self.strategies[indices[0]]
                .values
                .get(&obj.name)
                .copied()
                .unwrap_or(0.0);
            let f_max = self.strategies[indices[n - 1]]
                .values
                .get(&obj.name)
                .copied()
                .unwrap_or(0.0);
            let range = f_max - f_min;
            if range.abs() < f64::EPSILON {
                continue;
            }

            for k in 1..n - 1 {
                let prev = self.strategies[indices[k - 1]]
                    .values
                    .get(&obj.name)
                    .copied()
                    .unwrap_or(0.0);
                let next = self.strategies[indices[k + 1]]
                    .values
                    .get(&obj.name)
                    .copied()
                    .unwrap_or(0.0);
                distances[indices[k]] += (next - prev) / range;
            }
        }

        let mut ranked: Vec<usize> = (0..n).collect();
        ranked.sort_by(|&a, &b| {
            distances[b]
                .partial_cmp(&distances[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }
}
