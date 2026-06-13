# ternary-pareto

Pareto optimization for ternary agents — multi-objective strategies where you can't maximize everything at once. Dominance relations, Pareto front maintenance, hypervolume indicators, evolutionary search, and tradeoff analysis.

## Why It Matters

Real-world optimization rarely has a single objective. A ternary agent must balance speed vs. accuracy, exploration vs. exploitation, cooperation vs. competition. Pareto optimization handles these tradeoffs by finding the **Pareto front** — the set of strategies where no objective can be improved without sacrificing another.

This crate provides the full Pareto optimization toolkit:
- **Dominance checking**: determine if one strategy strictly dominates another
- **Pareto front maintenance**: incrementally insert strategies, automatically removing dominated ones
- **Hypervolume indicator**: quantify the quality of a Pareto front
- **Evolutionary search**: NSGA-II-style optimizer with tournament selection, crossover, and mutation
- **Tradeoff analysis**: marginal rates of substitution, knee points, conflict metrics

## How It Works

### Pareto Dominance

Strategy $a$ dominates strategy $b$ iff:

$$\forall i: f_i(a) \succeq_i f_i(b) \quad \text{AND} \quad \exists j: f_j(a) \succ_j f_j(b)$$

where $\succeq_i$ means "at least as good" for objective $i$. **Complexity:** O(m) per comparison, where $m$ = number of objectives.

### Pareto Front

The set of non-dominated strategies. Insertion is O($k \cdot m$) where $k$ is the current front size:
1. Remove all existing strategies dominated by the new one
2. If any remaining strategy dominates the new one, reject it
3. Otherwise, add to front

### Hypervolume Indicator

The volume of objective space dominated by the Pareto front, bounded by a reference point:

- **1D:** $\text{HV} = |f^* - r|$ — distance from best to reference
- **2D:** Sweep-line algorithm, O($k \log k$)
- **3D+:** Monte Carlo estimation with 10,000 samples

Higher hypervolume = better front quality.

### Crowding Distance

Diversity measure for NSGA-II selection. For each strategy, sum the normalized distances to its neighbors along each objective:

$$d_i = \sum_{m} \frac{f_m(i+1) - f_m(i-1)}{f_m^{\max} - f_m^{\min}}$$

Boundary strategies receive $d = \infty$. **Complexity:** O($k \log k$) per objective.

### Knee Point

The strategy with maximum distance from the ideal-to-nadir line — the "elbow" where improvements in one objective come at the greatest cost in others. Found by orthogonal projection in normalized objective space.

### Conflict Metric

Kendall-τ-inspired concordance measure between two objectives:

$$\text{conflict}(A, B) = \frac{|\text{discordant pairs}| - |\text{concordant pairs}|}{|\text{total pairs}|}$$

- **+1:** perfect conflict (Pareto front is a strict tradeoff curve)
- **-1:** perfect alignment (improving A always improves B)
- **0:** independence

## Quick Start

```rust
use ternary_pareto::*;
use std::collections::HashMap;

// Define objectives
let objectives = vec![
    Objective::maximize("speed"),
    Objective::minimize("energy"),
];

// Build a Pareto front
let mut front = ParetoFront::new(objectives.clone());
front.insert(ScoredStrategy::labeled(
    HashMap::from([("speed".into(), 10.0), ("energy".into(), 5.0)]),
    "fast-baseline"
));
front.insert(ScoredStrategy::labeled(
    HashMap::from([("speed".into(), 5.0), ("energy".into(), 2.0)]),
    "eco-mode"
));

// Hypervolume
let hv = HypervolumeIndicator::new(
    objectives.clone(),
    HashMap::from([("speed".into(), 0.0), ("energy".into(), 20.0)]),
);
let volume = hv.compute(&front);

// Tradeoff analysis
let analyzer = TradeoffAnalyzer::new(objectives.clone());
let knee = analyzer.knee_point(&front);
let conflict = analyzer.conflict(&front, "speed", "energy");

// Evolutionary search
let optimizer = ParetoOptimizer::new(objectives, vec![("x".into(), 0.0, 10.0)]);
let eval = |s: &HashMap<String, f64>| {
    let x = s["x"];
    HashMap::from([("speed".into(), x), ("energy".into(), x * x)])
};
let result = optimizer.optimize(&eval);
```

## API

| Type / Function | Description |
|---|---|
| `Objective::minimize/maximize(name)` | Define optimization objectives |
| `DominanceRelation::new(objectives)` | Dominance checker |
| `.dominates(a, b) / .compare(a, b)` | Pareto dominance queries |
| `ParetoFront::new(objectives)` | Non-dominated set |
| `.insert(strategy)` | Add strategy, remove dominated |
| `.ideal_point() / .nadir_point()` | Best/worst values per objective |
| `.closest_to_ideal()` | Strategy nearest to utopia |
| `.crowding_distance_ranking()` | NSGA-II diversity ranking |
| `HypervolumeIndicator::new(objs, ref_point)` | Quality indicator |
| `.compute(front)` | Hypervolume (exact for 1-2D, Monte Carlo for 3D+) |
| `ParetoOptimizer::new(objs, bounds)` | Evolutionary optimizer |
| `.optimize(evaluator)` | Run NSGA-II-style search |
| `TradeoffAnalyzer::new(objs)` | Tradeoff analysis toolkit |
| `.marginal_tradeoff / .knee_point / .conflict` | Tradeoff metrics |

## Architecture Notes

Multi-objective optimization in ternary systems directly reflects the **γ + η = C** conservation identity. Each objective represents a different "view" of the ternary population: maximizing one objective corresponds to growing the constructive mass γ, while minimizing another corresponds to shrinking the inhibitory mass η. The Pareto front traces the boundary where $\gamma + \eta = C$ is tight — any strategy on the front represents a different allocation of the conserved budget $C$ between competing objectives.

The knee point is particularly significant: it represents the allocation where the marginal rate of substitution between objectives is steepest — the "best compromise" point where the ternary system is most efficiently balanced.

## References

- Deb, K. (2001). *Multi-Objective Optimization Using Evolutionary Algorithms.* Wiley.
- Deb, K. et al. (2002). *A Fast and Elitist Multiobjective Genetic Algorithm: NSGA-II.* IEEE TEC, 6(2).
- Zitzler, E. & Thiele, L. (1999). *Multiobjective Evolutionary Algorithms: A Comparative Case Study.* PPSN.
- Branke, J. et al. (2008). *Finding the Knee in the Pareto Front.* GECCO.

## License

MIT
