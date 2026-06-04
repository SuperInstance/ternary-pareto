use crate::{Objective, ParetoFront, ScoredStrategy, Strategy};

/// Analyzes tradeoffs between objectives along a Pareto front.
pub struct TradeoffAnalyzer {
    objectives: Vec<Objective>,
}

impl TradeoffAnalyzer {
    /// Create a new tradeoff analyzer.
    pub fn new(objectives: Vec<Objective>) -> Self {
        Self { objectives }
    }

    /// Compute the marginal rate of substitution between two objectives
    /// at each point in the front.
    ///
    /// Returns a list of (strategy_index, tradeoff_ratio) pairs.
    /// The ratio represents how much of objective_b you sacrifice per unit
    /// gain in objective_a (moving along the front).
    pub fn marginal_tradeoff(
        &self,
        front: &ParetoFront,
        objective_a: &str,
        objective_b: &str,
    ) -> Vec<(usize, f64)> {
        if front.len() < 2 {
            return Vec::new();
        }

        // Sort strategies by objective_a
        let obj_a = match self.objectives.iter().find(|o| o.name == objective_a) {
            Some(o) => o,
            None => return Vec::new(),
        };

        let mut indexed: Vec<(usize, f64, f64)> = front
            .strategies()
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let va = *s.values.get(objective_a)?;
                let vb = *s.values.get(objective_b)?;
                Some((i, va, vb))
            })
            .collect();

        // Sort by objective_a
        indexed.sort_by(|a, b| {
            if obj_a.direction == crate::Direction::Minimize {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let mut result = Vec::new();
        for k in 0..indexed.len().saturating_sub(1) {
            let (_, a1, b1) = indexed[k];
            let (idx2, a2, b2) = indexed[k + 1];
            let delta_a = a2 - a1;
            let delta_b = b2 - b1;
            if delta_a.abs() > f64::EPSILON {
                result.push((idx2, delta_b / delta_a));
            }
        }

        result
    }

    /// Find the strategy that best balances all objectives (knee point).
    /// The knee point is the strategy with the highest marginal tradeoff,
    /// meaning improvements in any objective come at the greatest cost in others.
    pub fn knee_point(&self, front: &ParetoFront) -> Option<usize> {
        if front.is_empty() {
            return None;
        }
        if front.len() == 1 {
            return Some(0);
        }

        // Use distance from the "utopia" line (line from ideal to nadir)
        let ideal = front.ideal_point();
        let nadir = front.nadir_point();

        let mut best_idx = 0;
        let mut best_distance = -1.0f64;

        for (i, s) in front.strategies().iter().enumerate() {
            let dist = self.distance_from_utopia_line(s, &ideal, &nadir);
            if dist > best_distance {
                best_distance = dist;
                best_idx = i;
            }
        }

        Some(best_idx)
    }

    /// Compute distance of a strategy from the utopia line (ideal→nadir).
    fn distance_from_utopia_line(
        &self,
        strategy: &ScoredStrategy,
        ideal: &Strategy,
        nadir: &Strategy,
    ) -> f64 {
        let n = self.objectives.len();
        if n == 0 {
            return 0.0;
        }

        // Normalize all points to [0, 1] range
        let mut s_norm = Vec::new();
        let mut i_norm = Vec::new();
        let mut n_norm = Vec::new();

        for obj in &self.objectives {
            let lo = ideal.get(&obj.name).copied().unwrap_or(0.0);
            let hi = nadir.get(&obj.name).copied().unwrap_or(0.0);
            let range = hi - lo;
            if range.abs() < f64::EPSILON {
                s_norm.push(0.5);
                i_norm.push(0.0);
                n_norm.push(1.0);
            } else {
                let sv = strategy.values.get(&obj.name).copied().unwrap_or(0.0);
                let norm_val = (sv - lo) / range;
                s_norm.push(norm_val);
                i_norm.push(0.0);
                n_norm.push(1.0);
            }
        }

        // Project s_norm onto line from i_norm to n_norm
        // Direction vector
        let dir: Vec<f64> = n_norm.iter().zip(i_norm.iter()).map(|(n, i)| n - i).collect();
        let point: Vec<f64> = s_norm.iter().zip(i_norm.iter()).map(|(s, i)| s - i).collect();

        let dir_sq: f64 = dir.iter().map(|d| d * d).sum();
        if dir_sq < f64::EPSILON {
            return 0.0;
        }

        let projection: f64 = point.iter().zip(dir.iter()).map(|(p, d)| p * d).sum();
        let t = projection / dir_sq;

        // Closest point on line
        let closest: Vec<f64> = i_norm.iter().zip(dir.iter()).map(|(i, d)| i + t * d).collect();

        // Distance from s_norm to closest
        s_norm
            .iter()
            .zip(closest.iter())
            .map(|(s, c)| (s - c) * (s - c))
            .sum::<f64>()
            .sqrt()
    }

    /// Analyze the range of each objective across the front.
    /// Returns (min, max, range) for each objective.
    pub fn objective_ranges(
        &self,
        front: &ParetoFront,
    ) -> Vec<(String, f64, f64, f64)> {
        let mut ranges = Vec::new();

        for obj in &self.objectives {
            let values: Vec<f64> = front
                .strategies()
                .iter()
                .filter_map(|s| s.values.get(&obj.name).copied())
                .collect();

            if values.is_empty() {
                ranges.push((obj.name.clone(), f64::NAN, f64::NAN, f64::NAN));
            } else {
                let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                ranges.push((obj.name.clone(), min, max, max - min));
            }
        }

        ranges
    }

    /// Compute conflict between two objectives.
    /// Returns a value in [-1, 1]:
    /// - 1: perfect conflict (improving one always worsens the other)
    /// - -1: perfect alignment (improving one always improves the other)
    /// - 0: independent
    pub fn conflict(&self, front: &ParetoFront, obj_a: &str, obj_b: &str) -> f64 {
        if front.len() < 2 {
            return 0.0;
        }

        let dir_a = self
            .objectives
            .iter()
            .find(|o| o.name == obj_a)
            .map(|o| o.direction);
        let dir_b = self
            .objectives
            .iter()
            .find(|o| o.name == obj_b)
            .map(|o| o.direction);

        let (Some(_), Some(_)) = (dir_a, dir_b) else {
            return 0.0;
        };

        // Normalize values (lower is always better after normalization)
        let pairs: Vec<(f64, f64)> = front
            .strategies()
            .iter()
            .filter_map(|s| {
                let a = s.values.get(obj_a).copied()?;
                let b = s.values.get(obj_b).copied()?;
                Some((a, b))
            })
            .collect();

        if pairs.len() < 2 {
            return 0.0;
        }

        // Count concordant vs discordant pairs (Kendall tau-like)
        let n = pairs.len();
        let mut concordant = 0i64;
        let mut discordant = 0i64;

        for i in 0..n {
            for j in i + 1..n {
                let diff_a = pairs[i].0 - pairs[j].0;
                let diff_b = pairs[i].1 - pairs[j].1;
                let product = diff_a * diff_b;
                if product > 0.0 {
                    concordant += 1;
                } else if product < 0.0 {
                    discordant += 1;
                }
            }
        }

        let total = concordant + discordant;
        if total == 0 {
            return 0.0;
        }

        (discordant as f64 - concordant as f64) / total as f64
    }
}
