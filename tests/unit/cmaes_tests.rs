//! Unit tests for CMA-ES optimizer.

use cardgame::tuning::{CmaEs, CmaEsConfig};

#[test]
fn test_cmaes_sphere() {
    // Simple sphere function: minimize sum of squares
    // Optimum is at origin with fitness 0
    let dim = 5;
    let initial = vec![1.0; dim];
    let bounds = vec![(-5.0, 5.0); dim];

    let config = CmaEsConfig {
        population_size: Some(10),
        initial_sigma: 1.0,
        max_generations: 50,
        target_fitness: Some(-0.01), // Fitness is negative of sphere value
        min_sigma: 0.001,
        seed: 42,
    };

    let mut cmaes = CmaEs::new(initial, bounds, config);

    let mut best_fitness = f64::NEG_INFINITY;

    while !cmaes.should_stop(best_fitness) {
        let population = cmaes.sample_population();

        let evaluated: Vec<(Vec<f64>, f64)> = population
            .into_iter()
            .map(|x| {
                // Sphere function (negated since we maximize)
                let fitness = -x.iter().map(|xi| xi * xi).sum::<f64>();
                (x, fitness)
            })
            .collect();

        best_fitness = evaluated.iter().map(|(_, f)| *f).fold(f64::NEG_INFINITY, f64::max);
        cmaes.update(evaluated);
    }

    // Should find near-optimal solution
    let mean = cmaes.mean();
    let mean_sq: f64 = mean.iter().map(|x| x * x).sum();
    assert!(mean_sq < 1.0, "CMA-ES should find near-optimal solution, got sum_sq={}", mean_sq);
}

#[test]
fn test_population_sampling() {
    let dim = 3;
    let initial = vec![0.0; dim];
    let bounds = vec![(-10.0, 10.0); dim];

    let config = CmaEsConfig::default();
    let mut cmaes = CmaEs::new(initial, bounds.clone(), config);

    let population = cmaes.sample_population();

    assert_eq!(population.len(), cmaes.population_size());
    for candidate in &population {
        assert_eq!(candidate.len(), dim);
        for (i, &val) in candidate.iter().enumerate() {
            assert!(val >= bounds[i].0 && val <= bounds[i].1);
        }
    }
}
