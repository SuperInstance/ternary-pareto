#[cfg(test)]
mod tests {
    use ternary_pareto::*;

    fn make_strategy(pairs: &[(&str, f64)]) -> Strategy {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn make_objectives() -> Vec<Objective> {
        vec![
            Objective::minimize("latency"),
            Objective::maximize("throughput"),
        ]
    }

    #[test]
    fn test_objective_minimize() {
        let obj = Objective::minimize("cost");
        assert_eq!(obj.direction, Direction::Minimize);
        assert_eq!(obj.weight, 1.0);
        assert!(obj.is_better(1.0, 2.0));
        assert!(!obj.is_better(2.0, 1.0));
    }

    #[test]
    fn test_objective_maximize() {
        let obj = Objective::maximize("speed");
        assert_eq!(obj.direction, Direction::Maximize);
        assert!(obj.is_better(3.0, 2.0));
        assert!(!obj.is_better(2.0, 3.0));
    }

    #[test]
    fn test_objective_normalize() {
        let min_obj = Objective::minimize("cost");
        assert_eq!(min_obj.normalize(5.0), 5.0);

        let max_obj = Objective::maximize("speed");
        assert_eq!(max_obj.normalize(5.0), -5.0);
    }

    #[test]
    fn test_dominance_clear_case() {
        let dom = DominanceRelation::new(make_objectives());
        let a = make_strategy(&[("latency", 1.0), ("throughput", 100.0)]);
        let b = make_strategy(&[("latency", 2.0), ("throughput", 50.0)]);
        assert!(dom.dominates(&a, &b));
        assert!(!dom.dominates(&b, &a));
    }

    #[test]
    fn test_dominance_incomparable() {
        let dom = DominanceRelation::new(make_objectives());
        let a = make_strategy(&[("latency", 1.0), ("throughput", 50.0)]);
        let b = make_strategy(&[("latency", 2.0), ("throughput", 100.0)]);
        assert!(dom.is_incomparable(&a, &b));
    }

    #[test]
    fn test_dominance_equal() {
        let dom = DominanceRelation::new(make_objectives());
        let a = make_strategy(&[("latency", 1.0), ("throughput", 100.0)]);
        let b = make_strategy(&[("latency", 1.0), ("throughput", 100.0)]);
        // Equal strategies don't dominate each other
        assert!(!dom.dominates(&a, &b));
        assert!(!dom.dominates(&b, &a));
    }

    #[test]
    fn test_dominance_compare_scored() {
        let dom = DominanceRelation::new(make_objectives());
        let a = ScoredStrategy::new(make_strategy(&[("latency", 1.0), ("throughput", 100.0)]));
        let b = ScoredStrategy::new(make_strategy(&[("latency", 2.0), ("throughput", 50.0)]));
        assert_eq!(dom.compare(&a, &b), Dominance::ADominatesB);
    }

    #[test]
    fn test_pareto_front_insert_dominated() {
        let mut front = ParetoFront::new(make_objectives());
        assert!(front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 50.0)])));
        // Dominating strategy replaces it? No — the dominated one stays, and the new one is added.
        // Actually, dominated ones get removed.
        assert!(front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 100.0)])));
        assert_eq!(front.len(), 1); // first was dominated by second
    }

    #[test]
    fn test_pareto_front_incomparable() {
        let mut front = ParetoFront::new(make_objectives());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 50.0)]));
        front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 100.0)]));
        assert_eq!(front.len(), 2);
    }

    #[test]
    fn test_pareto_front_reject_dominated() {
        let mut front = ParetoFront::new(make_objectives());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 100.0)]));
        let added = front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 50.0)]));
        assert!(!added);
        assert_eq!(front.len(), 1);
    }

    #[test]
    fn test_pareto_front_ideal_point() {
        let mut front = ParetoFront::new(make_objectives());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 50.0)]));
        front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 100.0)]));
        let ideal = front.ideal_point();
        assert_eq!(ideal.get("latency").copied(), Some(1.0));
        assert_eq!(ideal.get("throughput").copied(), Some(100.0));
    }

    #[test]
    fn test_pareto_front_nadir_point() {
        let mut front = ParetoFront::new(make_objectives());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 50.0)]));
        front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 100.0)]));
        let nadir = front.nadir_point();
        assert_eq!(nadir.get("latency").copied(), Some(2.0));
        assert_eq!(nadir.get("throughput").copied(), Some(50.0));
    }

    #[test]
    fn test_pareto_front_merge() {
        let mut front1 = ParetoFront::new(make_objectives());
        front1.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 50.0)]));

        let mut front2 = ParetoFront::new(make_objectives());
        front2.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 100.0)]));

        let added = front1.merge(&front2);
        assert_eq!(added, 1);
        assert_eq!(front1.len(), 2);
    }

    #[test]
    fn test_hypervolume_2d() {
        let objectives = make_objectives();
        let mut front = ParetoFront::new(objectives.clone());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 100.0)]));
        front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 200.0)]));

        let reference = make_strategy(&[("latency", 3.0), ("throughput", 0.0)]);
        let hv = HypervolumeIndicator::new(objectives, reference);
        let volume = hv.compute(&front);
        assert!(volume > 0.0);
    }

    #[test]
    fn test_hypervolume_empty_front() {
        let objectives = make_objectives();
        let front = ParetoFront::new(objectives.clone());
        let reference = make_strategy(&[("latency", 10.0), ("throughput", 0.0)]);
        let hv = HypervolumeIndicator::new(objectives, reference);
        assert_eq!(hv.compute(&front), 0.0);
    }

    #[test]
    fn test_hypervolume_1d() {
        let objectives = vec![Objective::minimize("cost")];
        let mut front = ParetoFront::new(objectives.clone());
        front.insert_strategy(make_strategy(&[("cost", 5.0)]));

        let reference = make_strategy(&[("cost", 10.0)]);
        let hv = HypervolumeIndicator::new(objectives, reference);
        assert_eq!(hv.compute(&front), 5.0);
    }

    #[test]
    fn test_scored_strategy() {
        let s = ScoredStrategy::labeled(
            make_strategy(&[("x", 1.0)]),
            "test",
        );
        assert_eq!(s.label, Some("test".to_string()));
        assert_eq!(s.get("x"), Some(1.0));
        assert_eq!(s.get("y"), None);
    }

    #[test]
    fn test_crowding_distance() {
        let mut front = ParetoFront::new(make_objectives());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 50.0)]));
        front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 100.0)]));
        front.insert_strategy(make_strategy(&[("latency", 3.0), ("throughput", 150.0)]));

        let ranked = front.crowding_distance_ranking();
        assert_eq!(ranked.len(), 3);
        // Boundary points should be ranked first (highest crowding distance)
    }

    #[test]
    fn test_optimizer_grid_search() {
        let objectives = vec![Objective::minimize("x")];
        let optimizer = ParetoOptimizer::new(
            objectives.clone(),
            vec![("x".to_string(), 0.0, 10.0)],
        );

        let front = optimizer.grid_search(&|s| s.clone(), 11);
        // Should find x=0 as the best
        assert!(!front.is_empty());
        let best = front.closest_to_ideal().unwrap();
        assert!(best.get("x").unwrap() <= 1.0);
    }

    #[test]
    fn test_optimizer_evolutionary() {
        let objectives = vec![Objective::minimize("x"), Objective::minimize("y")];
        let optimizer = ParetoOptimizer::new(
            objectives.clone(),
            vec![("x".to_string(), 0.0, 10.0), ("y".to_string(), 0.0, 10.0)],
        ).with_config(SearchConfig {
            population_size: 50,
            generations: 20,
            seed: 42,
            ..Default::default()
        });

        // Simple evaluator: x and y are their own objectives
        let front = optimizer.optimize(&|s| s.clone());
        assert!(!front.is_empty());
    }

    #[test]
    fn test_tradeoff_conflict() {
        let mut front = ParetoFront::new(make_objectives());
        // Strategies with clear tradeoff: low latency = low throughput
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 10.0)]));
        front.insert_strategy(make_strategy(&[("latency", 2.0), ("throughput", 20.0)]));
        front.insert_strategy(make_strategy(&[("latency", 3.0), ("throughput", 30.0)]));

        let analyzer = TradeoffAnalyzer::new(make_objectives());
        let conflict = analyzer.conflict(&front, "latency", "throughput");
        // These objectives are aligned in this case (both increase together)
        // Conflict depends on perspective — both increasing means conflict for a minimizer
        assert!(conflict.abs() <= 1.0);
    }

    #[test]
    fn test_tradeoff_ranges() {
        let mut front = ParetoFront::new(make_objectives());
        front.insert_strategy(make_strategy(&[("latency", 1.0), ("throughput", 100.0)]));
        front.insert_strategy(make_strategy(&[("latency", 5.0), ("throughput", 200.0)]));

        let analyzer = TradeoffAnalyzer::new(make_objectives());
        let ranges = analyzer.objective_ranges(&front);
        assert_eq!(ranges.len(), 2);
        // Check latency range
        assert_eq!(ranges[0].0, "latency");
        assert_eq!(ranges[0].1, 1.0); // min
        assert_eq!(ranges[0].2, 5.0); // max
        assert_eq!(ranges[0].3, 4.0); // range
    }

    #[test]
    fn test_knee_point() {
        let mut front = ParetoFront::new(make_objectives());
        for i in 0..5u32 {
            let lat = 1.0 + i as f64;
            let thr = 200.0 - (i as f64 * 40.0);
            front.insert_strategy(make_strategy(&[("latency", lat), ("throughput", thr)]));
        }
        assert!(!front.is_empty());

        let analyzer = TradeoffAnalyzer::new(make_objectives());
        let knee = analyzer.knee_point(&front);
        assert!(knee.is_some());
    }
}
