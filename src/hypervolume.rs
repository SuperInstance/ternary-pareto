use crate::{Objective, ParetoFront, Strategy};

/// Computes the hypervolume indicator of a Pareto front.
///
/// The hypervolume indicator measures the volume of objective space dominated
/// by the Pareto front, bounded by a reference point. Higher hypervolume means
/// a better Pareto front.
pub struct HypervolumeIndicator {
    objectives: Vec<Objective>,
    reference_point: Strategy,
}

impl HypervolumeIndicator {
    /// Create a new hypervolume indicator with a reference point.
    ///
    /// The reference point should be worse than all Pareto front members
    /// in every objective.
    pub fn new(objectives: Vec<Objective>, reference_point: Strategy) -> Self {
        Self {
            objectives,
            reference_point,
        }
    }

    /// Compute the hypervolume of the given Pareto front.
    ///
    /// For 1 objective: simple area calculation.
    /// For 2 objectives: uses the standard 2D hypervolume algorithm.
    /// For 3+ objectives: uses Monte Carlo estimation.
    pub fn compute(&self, front: &ParetoFront) -> f64 {
        if front.is_empty() {
            return 0.0;
        }

        match self.objectives.len() {
            0 => 0.0,
            1 => self.compute_1d(front),
            2 => self.compute_2d(front),
            _ => self.compute_monte_carlo(front, 10000),
        }
    }

    /// 1D hypervolume.
    fn compute_1d(&self, front: &ParetoFront) -> f64 {
        let obj = &self.objectives[0];
        let ref_val = self.reference_point.get(&obj.name).copied().unwrap_or(0.0);

        let mut best = f64::NAN;
        for s in front.strategies() {
            if let Some(&v) = s.values.get(&obj.name) {
                if best.is_nan() || obj.is_better(v, best) {
                    best = v;
                }
            }
        }

        if best.is_nan() {
            return 0.0;
        }

        let vol = match obj.direction {
            crate::Direction::Minimize => ref_val - best,
            crate::Direction::Maximize => best - ref_val,
        };

        vol.max(0.0)
    }

    /// 2D hypervolume using the standard sweeping algorithm.
    fn compute_2d(&self, front: &ParetoFront) -> f64 {
        let obj0 = &self.objectives[0];
        let obj1 = &self.objectives[1];
        let ref0 = self.reference_point.get(&obj0.name).copied().unwrap_or(0.0);
        let ref1 = self.reference_point.get(&obj1.name).copied().unwrap_or(0.0);

        // Collect and normalize points so lower is always better
        let mut points: Vec<(f64, f64)> = front
            .strategies()
            .iter()
            .filter_map(|s| {
                let v0 = s.values.get(&obj0.name).copied()?;
                let v1 = s.values.get(&obj1.name).copied()?;
                // Normalize to cost (lower = better)
                let n0 = obj0.normalize(v0);
                let n1 = obj1.normalize(v1);
                let nr0 = obj0.normalize(ref0);
                let nr1 = obj1.normalize(ref1);
                // Only include points that dominate the reference
                if n0 <= nr0 && n1 <= nr1 {
                    Some((n0, n1))
                } else {
                    None
                }
            })
            .collect();

        if points.is_empty() {
            return 0.0;
        }

        // Sort by first objective (ascending, since lower is better)
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Sweep and compute area
        let nr0 = obj0.normalize(ref0);
        let nr1 = obj1.normalize(ref1);

        let mut volume = 0.0;
        let mut prev_x = points[0].0;
        let mut min_y = points[0].1;

        for &(x, y) in &points {
            if y < min_y {
                // New point extends the front
                volume += (x - prev_x) * (nr1 - min_y).max(0.0);
                min_y = y;
                prev_x = x;
            }
        }
        // Last segment
        volume += (nr0 - prev_x) * (nr1 - min_y).max(0.0);

        volume.max(0.0)
    }

    /// Monte Carlo estimation for higher dimensions.
    fn compute_monte_carlo(&self, front: &ParetoFront, samples: usize) -> f64 {
        // Compute bounding box: ideal point to reference point
        let ideal = front.ideal_point();
        let mut lower = Vec::new();
        let mut upper = Vec::new();
        let mut ranges = Vec::new();

        for obj in &self.objectives {
            let lo = ideal.get(&obj.name).copied().unwrap_or(0.0);
            let hi = self.reference_point.get(&obj.name).copied().unwrap_or(0.0);
            // Ensure proper ordering based on direction
            let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
            lower.push(lo);
            upper.push(hi);
            ranges.push(hi - lo);
        }

        let total_volume: f64 = ranges.iter().product();
        if total_volume <= 0.0 {
            return 0.0;
        }

        // Simple LCG PRNG (no external deps)
        let mut state: u64 = 123456789;
        let mut dominated_count = 0usize;

        for _ in 0..samples {
            // Generate random point in bounding box
            let mut point = Vec::with_capacity(self.objectives.len());
            for i in 0..self.objectives.len() {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let t = (state >> 33) as f64 / (1u64 << 31) as f64;
                point.push(lower[i] + t * ranges[i]);
            }

            // Check if dominated by any strategy
            for s in front.strategies() {
                let mut is_dominated = true;
                for (i, obj) in self.objectives.iter().enumerate() {
                    let sv = s.values.get(&obj.name).copied().unwrap_or(f64::NAN);
                    if !obj.is_better(sv, point[i]) && (sv - point[i]).abs() > f64::EPSILON {
                        is_dominated = false;
                        break;
                    }
                }
                if is_dominated {
                    dominated_count += 1;
                    break;
                }
            }
        }

        total_volume * (dominated_count as f64) / (samples as f64)
    }

    /// Exclusive hypervolume contribution of a specific strategy.
    /// This is the volume that would be lost if this strategy were removed.
    pub fn exclusive_contribution(&self, front: &ParetoFront, index: usize) -> f64 {
        if index >= front.len() {
            return 0.0;
        }

        let full_hv = self.compute(front);

        // Create front without this strategy
        let mut reduced = ParetoFront::new(self.objectives.clone());
        for (i, s) in front.strategies().iter().enumerate() {
            if i != index {
                reduced.insert(s.clone());
            }
        }

        let reduced_hv = self.compute(&reduced);
        (full_hv - reduced_hv).max(0.0)
    }
}
