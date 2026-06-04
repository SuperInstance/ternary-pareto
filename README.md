# ternary-pareto

Pareto optimization for ternary agents — multi-objective strategies where you can't maximize everything at once.

## Pareto Optimization Theory

In real-world optimization, you rarely have just one objective. You might want to **minimize latency** while **maximizing throughput** and **minimizing cost**. These objectives often conflict — improving one worsens another.

### Pareto Dominance

A strategy **A** *Pareto-dominates* strategy **B** if:
- A is at least as good as B in **every** objective
- A is strictly better than B in **at least one** objective

If neither dominates the other, they are **incomparable** — and both may be worth considering.

### Pareto Front

The **Pareto front** is the set of all non-dominated strategies. No strategy in the front dominates another. Every strategy on the front represents a different tradeoff.

### Hypervolume Indicator

The **hypervolume indicator** measures the quality of a Pareto front by computing the volume of objective space dominated by it (relative to a reference point). Higher hypervolume = better coverage of tradeoff space.

### Knee Point

The **knee point** is the strategy on the Pareto front where the marginal cost of improving any objective is highest. It's often the "best compromise" — the point where you get the most balanced outcome.

## Usage

```rust
use ternary_pareto::*;

// Define objectives
let objectives = vec![
    Objective::minimize("latency"),
    Objective::maximize("throughput"),
    Objective::minimize("cost"),
];

// Build a Pareto front
let mut front = ParetoFront::new(objectives.clone());
front.insert(ScoredStrategy::labeled(
    [("latency".into(), 10.0), ("throughput".into(), 500.0), ("cost".into(), 2.0)].into(),
    "fast-expensive",
));
front.insert(ScoredStrategy::labeled(
    [("latency".into(), 50.0), ("throughput".into(), 200.0), ("cost".into(), 0.5)].into(),
    "slow-cheap",
));

// Check dominance
let dom = DominanceRelation::new(objectives.clone());
assert!(dom.is_incomparable(
    &front.strategies()[0].values,
    &front.strategies()[1].values,
));

// Compute hypervolume
let reference = [("latency".into(), 100.0), ("throughput".into(), 0.0), ("cost".into(), 10.0)].into();
let hv = HypervolumeIndicator::new(objectives.clone(), reference);
println!("Hypervolume: {}", hv.compute(&front));

// Find the knee point
let analyzer = TradeoffAnalyzer::new(objectives);
if let Some(knee_idx) = analyzer.knee_point(&front) {
    println!("Knee point: {:?}", front.strategies()[knee_idx]);
}
```

## Evolutionary Search

```rust
use ternary_pareto::*;

let objectives = vec![Objective::minimize("x"), Objective::minimize("y")];
let optimizer = ParetoOptimizer::new(
    objectives.clone(),
    vec![("x".into(), 0.0, 10.0), ("y".into(), 0.0, 10.0)],
).with_config(SearchConfig {
    population_size: 100,
    generations: 50,
    seed: 42,
    ..Default::default()
});

let front = optimizer.optimize(&|params| params.clone());
println!("Found {} Pareto-optimal strategies", front.len());
```

## Features

- **`Objective`** — named objectives with direction (minimize/maximize) and weight
- **`ParetoFront`** — maintain non-dominated strategies with ideal/nadir points and crowding distance
- **`DominanceRelation`** — determine if one strategy dominates another
- **`ParetoOptimizer`** — evolutionary search and grid search for Pareto-optimal strategies
- **`HypervolumeIndicator`** — compute hypervolume (1D exact, 2D sweep, 3+D Monte Carlo)
- **`TradeoffAnalyzer`** — marginal tradeoffs, knee points, objective ranges, conflict analysis

## Design

- Pure Rust, no unsafe code, no external dependencies
- Generic over objective definitions
- Extensible evaluator functions for optimization

## License

MIT
