use crate::{Objective, ParetoFront, ScoredStrategy, Strategy};

/// Configuration for the evolutionary search.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Population size.
    pub population_size: usize,
    /// Number of generations.
    pub generations: usize,
    /// Mutation strength (standard deviation relative to objective range).
    pub mutation_rate: f64,
    /// Crossover probability.
    pub crossover_rate: f64,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            generations: 50,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            seed: 42,
        }
    }
}

impl SearchConfig {
    /// Create a new search config with custom parameters.
    pub fn new(population_size: usize, generations: usize) -> Self {
        Self {
            population_size,
            generations,
            ..Default::default()
        }
    }
}

/// A function that evaluates a strategy and returns objective values.
pub type EvaluatorFn = Box<dyn Fn(&Strategy) -> Strategy>;

/// Pareto optimizer that searches for Pareto-optimal strategies.
pub struct ParetoOptimizer {
    objectives: Vec<Objective>,
    config: SearchConfig,
    bounds: Vec<(String, f64, f64)>, // (name, min, max)
}

impl ParetoOptimizer {
    /// Create a new optimizer with objectives and search bounds.
    ///
    /// Bounds define the search space for each objective as (name, min_value, max_value).
    pub fn new(objectives: Vec<Objective>, bounds: Vec<(String, f64, f64)>) -> Self {
        Self {
            objectives,
            config: SearchConfig::default(),
            bounds,
        }
    }

    /// Set the search configuration.
    pub fn with_config(mut self, config: SearchConfig) -> Self {
        self.config = config;
        self
    }

    /// Run evolutionary search with an evaluator function.
    ///
    /// The evaluator takes a strategy (parameterization) and returns
    /// objective values.
    pub fn optimize(&self, evaluator: &dyn Fn(&Strategy) -> Strategy) -> ParetoFront {
        let mut rng = SimpleRng::new(self.config.seed);
        let mut front = ParetoFront::new(self.objectives.clone());

        // Generate initial population
        let mut population: Vec<Strategy> = (0..self.config.population_size)
            .map(|_| self.random_strategy(&mut rng))
            .collect();

        // Evaluate initial population
        for strategy in &population {
            let evaluated = evaluator(strategy);
            front.insert(ScoredStrategy::new(evaluated));
        }

        // Evolutionary loop
        for _gen in 0..self.config.generations {
            let mut offspring = Vec::new();

            // Generate offspring
            while offspring.len() < self.config.population_size {
                let parent_a = self.tournament_select(&population, evaluator, &mut rng);
                let parent_b = self.tournament_select(&population, evaluator, &mut rng);

                let (child_a, child_b) = if rng.next() < self.config.crossover_rate {
                    self.crossover(&parent_a, &parent_b, &mut rng)
                } else {
                    (parent_a.clone(), parent_b.clone())
                };

                offspring.push(self.mutate(&child_a, &mut rng));
                offspring.push(self.mutate(&child_b, &mut rng));
            }

            // Evaluate offspring and add to front
            for strategy in &offspring {
                let evaluated = evaluator(strategy);
                front.insert(ScoredStrategy::new(evaluated));
            }

            // Selection: keep best population_size strategies
            population = offspring;

            // Also keep current front members in population
            for s in front.strategies() {
                if population.len() < self.config.population_size * 2 {
                    population.push(s.values.clone());
                }
            }

            population.truncate(self.config.population_size * 2);
        }

        front
    }

    /// Run a simple grid search over the parameter space.
    /// Good for low-dimensional problems with cheap evaluators.
    pub fn grid_search(
        &self,
        evaluator: &dyn Fn(&Strategy) -> Strategy,
        resolution: usize,
    ) -> ParetoFront {
        let mut front = ParetoFront::new(self.objectives.clone());
        let total_points = resolution.pow(self.bounds.len() as u32);

        for idx in 0..total_points {
            let mut strategy = Strategy::new();
            let mut remaining = idx;

            for (name, min, max) in &self.bounds {
                let coord = remaining % resolution;
                remaining /= resolution;
                let t = if resolution > 1 {
                    coord as f64 / (resolution - 1) as f64
                } else {
                    0.5
                };
                strategy.insert(name.clone(), min + t * (max - min));
            }

            let evaluated = evaluator(&strategy);
            front.insert(ScoredStrategy::new(evaluated));
        }

        front
    }

    fn random_strategy(&self, rng: &mut SimpleRng) -> Strategy {
        let mut s = Strategy::new();
        for (name, min, max) in &self.bounds {
            s.insert(name.clone(), min + rng.next() * (max - min));
        }
        s
    }

    fn tournament_select(
        &self,
        population: &[Strategy],
        evaluator: &dyn Fn(&Strategy) -> Strategy,
        rng: &mut SimpleRng,
    ) -> Strategy {
        let tournament_size = 3;
        let dominance = crate::DominanceRelation::new(self.objectives.clone());

        let mut best = population[rng.next_usize(population.len())].clone();
        let best_eval = evaluator(&best);

        for _ in 1..tournament_size {
            let candidate = population[rng.next_usize(population.len())].clone();
            let candidate_eval = evaluator(&candidate);
            if dominance.dominates(&candidate_eval, &best_eval) {
                best = candidate;
            }
        }

        best
    }

    fn crossover(
        &self,
        a: &Strategy,
        b: &Strategy,
        rng: &mut SimpleRng,
    ) -> (Strategy, Strategy) {
        let mut child_a = Strategy::new();
        let mut child_b = Strategy::new();

        for (name, _min, _max) in &self.bounds {
            let va = a.get(name).copied().unwrap_or(0.0);
            let vb = b.get(name).copied().unwrap_or(0.0);
            let t = rng.next();
            child_a.insert(name.clone(), t * va + (1.0 - t) * vb);
            child_b.insert(name.clone(), (1.0 - t) * va + t * vb);
        }

        (child_a, child_b)
    }

    fn mutate(&self, strategy: &Strategy, rng: &mut SimpleRng) -> Strategy {
        let mut mutated = strategy.clone();

        for (name, min, max) in &self.bounds {
            if rng.next() < self.config.mutation_rate {
                let current = mutated.get(name).copied().unwrap_or((*min + *max) / 2.0);
                let range = max - min;
                // Gaussian-like mutation using Box-Muller
                let u1 = rng.next().max(1e-10);
                let u2 = rng.next();
                let normal = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                let new_val = current + normal * range * self.config.mutation_rate;
                mutated.insert(name.clone(), new_val.max(*min).min(*max));
            }
        }

        mutated
    }
}

/// Simple LCG-based PRNG (no external dependencies).
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        (self.state >> 33) as f64 / (1u64 << 31) as f64
    }

    fn next_usize(&mut self, max: usize) -> usize {
        (self.next() * max as f64) as usize
    }
}
