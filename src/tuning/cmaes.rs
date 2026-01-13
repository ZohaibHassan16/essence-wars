//! CMA-ES (Covariance Matrix Adaptation Evolution Strategy) optimizer.
//!
//! A derivative-free optimization algorithm well-suited for:
//! - Noisy fitness functions (like game outcomes)
//! - Continuous parameter spaces
//! - Moderate dimensionality (10-100 parameters)

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/// Configuration for CMA-ES optimizer.
#[derive(Clone, Debug)]
pub struct CmaEsConfig {
    /// Population size (lambda). If None, uses default based on dimension.
    pub population_size: Option<usize>,
    /// Initial step size (sigma)
    pub initial_sigma: f64,
    /// Maximum generations
    pub max_generations: u32,
    /// Target fitness to stop early (higher is better)
    pub target_fitness: Option<f64>,
    /// Minimum sigma threshold for convergence (default: 0.001)
    pub min_sigma: f64,
    /// Random seed
    pub seed: u64,
}

impl Default for CmaEsConfig {
    fn default() -> Self {
        Self {
            population_size: None,
            initial_sigma: 0.5,
            max_generations: 100,
            target_fitness: None,
            min_sigma: 0.001,
            seed: 42,
        }
    }
}

/// CMA-ES optimizer state.
pub struct CmaEs {
    /// Problem dimension
    dim: usize,
    /// Population size (lambda)
    lambda: usize,
    /// Number of parents (mu)
    mu: usize,
    /// Recombination weights
    weights: Vec<f64>,
    /// Variance effective selection mass
    mu_eff: f64,

    /// Current mean (best estimate)
    mean: Vec<f64>,
    /// Step size
    sigma: f64,
    /// Covariance matrix (stored as symmetric)
    cov: Vec<Vec<f64>>,
    /// Evolution path for sigma
    ps: Vec<f64>,
    /// Evolution path for covariance
    pc: Vec<f64>,

    /// Learning rate for rank-one update
    c1: f64,
    /// Learning rate for rank-mu update
    cmu: f64,
    /// Learning rate for cumulation for sigma
    cs: f64,
    /// Learning rate for cumulation for covariance
    cc: f64,
    /// Damping for sigma
    damps: f64,

    /// Expected length of N(0,I) vector
    chi_n: f64,

    /// Current generation
    generation: u32,
    /// Configuration
    config: CmaEsConfig,
    /// RNG
    rng: SmallRng,

    /// Parameter bounds (min, max) per dimension
    bounds: Vec<(f64, f64)>,
}

impl CmaEs {
    /// Create a new CMA-ES optimizer.
    ///
    /// # Arguments
    /// * `initial_mean` - Starting point in parameter space
    /// * `bounds` - Min/max bounds per dimension
    /// * `config` - Optimizer configuration
    pub fn new(initial_mean: Vec<f64>, bounds: Vec<(f64, f64)>, config: CmaEsConfig) -> Self {
        let dim = initial_mean.len();
        assert_eq!(dim, bounds.len(), "Bounds must match dimension");
        assert!(dim > 0, "Dimension must be positive");

        // Default population size: 4 + floor(3 * ln(n))
        let lambda = config.population_size.unwrap_or_else(|| {
            4 + (3.0 * (dim as f64).ln()).floor() as usize
        });
        let mu = lambda / 2;

        // Recombination weights (logarithmic)
        let raw_weights: Vec<f64> = (0..mu)
            .map(|i| ((mu as f64 + 0.5).ln() - ((i + 1) as f64).ln()).max(0.0))
            .collect();
        let sum: f64 = raw_weights.iter().sum();
        let weights: Vec<f64> = raw_weights.iter().map(|w| w / sum).collect();

        // Variance effective selection mass
        let sum_sq: f64 = weights.iter().map(|w| w * w).sum();
        let mu_eff = 1.0 / sum_sq;

        // Adaptation parameters
        let cs = (mu_eff + 2.0) / (dim as f64 + mu_eff + 5.0);
        let cc = (4.0 + mu_eff / dim as f64) / (dim as f64 + 4.0 + 2.0 * mu_eff / dim as f64);
        let c1 = 2.0 / ((dim as f64 + 1.3).powi(2) + mu_eff);
        let cmu = (2.0 * (mu_eff - 2.0 + 1.0 / mu_eff)
            / ((dim as f64 + 2.0).powi(2) + 2.0 * mu_eff / 2.0))
            .min(1.0 - c1);
        let damps = 1.0 + 2.0 * (0.0_f64.max(((mu_eff - 1.0) / (dim as f64 + 1.0)).sqrt() - 1.0)) + cs;

        // Expected length of N(0,I) vector
        let chi_n = (dim as f64).sqrt()
            * (1.0 - 1.0 / (4.0 * dim as f64) + 1.0 / (21.0 * (dim as f64).powi(2)));

        // Initialize covariance matrix to identity
        let mut cov = vec![vec![0.0; dim]; dim];
        for (i, row) in cov.iter_mut().enumerate().take(dim) {
            row[i] = 1.0;
        }

        Self {
            dim,
            lambda,
            mu,
            weights,
            mu_eff,
            mean: initial_mean,
            sigma: config.initial_sigma,
            cov,
            ps: vec![0.0; dim],
            pc: vec![0.0; dim],
            c1,
            cmu,
            cs,
            cc,
            damps,
            chi_n,
            generation: 0,
            rng: SmallRng::seed_from_u64(config.seed),
            config,
            bounds,
        }
    }

    /// Get the current generation number.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Get the current best estimate (mean).
    pub fn mean(&self) -> &[f64] {
        &self.mean
    }

    /// Get the current step size.
    pub fn sigma(&self) -> f64 {
        self.sigma
    }

    /// Get population size.
    pub fn population_size(&self) -> usize {
        self.lambda
    }

    /// Check if optimization should stop.
    pub fn should_stop(&self, best_fitness: f64) -> bool {
        if self.generation >= self.config.max_generations {
            return true;
        }
        if let Some(target) = self.config.target_fitness {
            if best_fitness >= target {
                return true;
            }
        }
        // Stop if sigma drops below threshold (convergence)
        if self.sigma < self.config.min_sigma {
            return true;
        }
        false
    }

    /// Get the reason for stopping.
    pub fn stop_reason(&self, best_fitness: f64) -> Option<&'static str> {
        if self.generation >= self.config.max_generations {
            return Some("max generations reached");
        }
        if let Some(target) = self.config.target_fitness {
            if best_fitness >= target {
                return Some("target fitness reached");
            }
        }
        if self.sigma < self.config.min_sigma {
            return Some("sigma converged");
        }
        None
    }

    /// Sample a population of candidates.
    pub fn sample_population(&mut self) -> Vec<Vec<f64>> {
        // Compute Cholesky-like decomposition of covariance matrix
        // For simplicity, we use a basic approach: C = B * D^2 * B^T
        // where B is orthogonal and D is diagonal (eigendecomposition)
        let sqrt_cov = self.sqrt_covariance();

        let mut population = Vec::with_capacity(self.lambda);

        for _ in 0..self.lambda {
            // Sample from N(0, I)
            let z: Vec<f64> = (0..self.dim).map(|_| self.rng.gen::<f64>() * 2.0 - 1.0).collect();

            // Normalize to proper normal distribution using Box-Muller
            let z: Vec<f64> = z
                .chunks(2)
                .flat_map(|pair| {
                    if pair.len() == 2 {
                        let u1 = self.rng.gen::<f64>();
                        let u2 = self.rng.gen::<f64>();
                        let r = (-2.0 * u1.ln()).sqrt();
                        let theta = 2.0 * std::f64::consts::PI * u2;
                        vec![r * theta.cos(), r * theta.sin()]
                    } else {
                        let u1 = self.rng.gen::<f64>();
                        let u2 = self.rng.gen::<f64>();
                        let r = (-2.0 * u1.ln()).sqrt();
                        let theta = 2.0 * std::f64::consts::PI * u2;
                        vec![r * theta.cos()]
                    }
                })
                .take(self.dim)
                .collect();

            // Transform: x = mean + sigma * B * D * z
            let mut x = vec![0.0; self.dim];
            for i in 0..self.dim {
                x[i] = self.mean[i];
                for j in 0..self.dim {
                    x[i] += self.sigma * sqrt_cov[i][j] * z[j];
                }
            }

            // Clip to bounds
            for (i, val) in x.iter_mut().enumerate().take(self.dim) {
                *val = val.clamp(self.bounds[i].0, self.bounds[i].1);
            }

            population.push(x);
        }

        population
    }

    /// Update the optimizer with fitness values.
    ///
    /// `evaluated` should be pairs of (candidate, fitness) sorted by fitness (best first).
    pub fn update(&mut self, mut evaluated: Vec<(Vec<f64>, f64)>) {
        // Sort by fitness (descending - higher is better)
        evaluated.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select mu best
        let selected: Vec<&Vec<f64>> = evaluated.iter().take(self.mu).map(|(x, _)| x).collect();

        // Compute new mean
        let old_mean = self.mean.clone();
        self.mean = vec![0.0; self.dim];
        for (i, x) in selected.iter().enumerate() {
            for j in 0..self.dim {
                self.mean[j] += self.weights[i] * x[j];
            }
        }

        // Compute mean displacement
        let mean_diff: Vec<f64> = self.mean.iter()
            .zip(old_mean.iter())
            .map(|(new, old)| (new - old) / self.sigma)
            .collect();

        // Update evolution path for sigma
        let sqrt_cs = (self.cs * (2.0 - self.cs) * self.mu_eff).sqrt();
        for (ps_val, &diff) in self.ps.iter_mut().zip(mean_diff.iter()).take(self.dim) {
            *ps_val = (1.0 - self.cs) * *ps_val + sqrt_cs * diff;
        }

        // Compute |ps|
        let ps_norm: f64 = self.ps.iter().map(|x| x * x).sum::<f64>().sqrt();

        // Update sigma
        self.sigma *= ((self.cs / self.damps) * (ps_norm / self.chi_n - 1.0)).exp();
        self.sigma = self.sigma.clamp(1e-12, 1e6);

        // Heaviside function for stalling detection
        let hsig = if ps_norm / (1.0 - (1.0 - self.cs).powi(2 * (self.generation as i32 + 1))).sqrt()
            < (1.4 + 2.0 / (self.dim as f64 + 1.0)) * self.chi_n
        {
            1.0
        } else {
            0.0
        };

        // Update evolution path for covariance
        let sqrt_cc = (self.cc * (2.0 - self.cc) * self.mu_eff).sqrt();
        for (pc_val, &diff) in self.pc.iter_mut().zip(mean_diff.iter()).take(self.dim) {
            *pc_val = (1.0 - self.cc) * *pc_val + hsig * sqrt_cc * diff;
        }

        // Update covariance matrix
        // Rank-one update
        let c1_adj = self.c1 * (1.0 - (1.0 - hsig.powi(2)) * self.cc * (2.0 - self.cc));

        for i in 0..self.dim {
            for j in 0..self.dim {
                // Rank-one: c1 * pc * pc^T
                let rank_one = c1_adj * self.pc[i] * self.pc[j];

                // Rank-mu: cmu * sum(w_i * (x_i - old_mean) * (x_i - old_mean)^T) / sigma^2
                let mut rank_mu = 0.0;
                for (k, x) in selected.iter().enumerate() {
                    let yi = (x[i] - old_mean[i]) / self.sigma;
                    let yj = (x[j] - old_mean[j]) / self.sigma;
                    rank_mu += self.weights[k] * yi * yj;
                }
                rank_mu *= self.cmu;

                // Combined update
                self.cov[i][j] = (1.0 - c1_adj - self.cmu) * self.cov[i][j] + rank_one + rank_mu;
            }
        }

        // Ensure covariance matrix stays symmetric and positive definite
        self.repair_covariance();

        self.generation += 1;
    }

    /// Compute approximate square root of covariance matrix.
    fn sqrt_covariance(&self) -> Vec<Vec<f64>> {
        // For efficiency, we use a simplified approach:
        // Just return the covariance matrix itself when near identity,
        // otherwise use power iteration to get principal components.

        // Check if close to identity
        let mut max_off_diag: f64 = 0.0;
        for i in 0..self.dim {
            for j in 0..self.dim {
                if i != j {
                    max_off_diag = max_off_diag.max(self.cov[i][j].abs());
                }
            }
        }

        if max_off_diag < 0.1 {
            // Close to diagonal - just use square root of diagonal
            let mut result = vec![vec![0.0; self.dim]; self.dim];
            for (i, row) in result.iter_mut().enumerate().take(self.dim) {
                row[i] = self.cov[i][i].abs().sqrt().max(0.001);
            }
            return result;
        }

        // For more complex cases, use Cholesky decomposition
        self.cholesky()
    }

    /// Cholesky decomposition of covariance matrix.
    fn cholesky(&self) -> Vec<Vec<f64>> {
        let mut l = vec![vec![0.0; self.dim]; self.dim];

        for i in 0..self.dim {
            for j in 0..=i {
                let mut sum = self.cov[i][j];
                for (l_i, l_j) in l[i].iter().zip(l[j].iter()).take(j) {
                    sum -= l_i * l_j;
                }
                if i == j {
                    l[i][j] = sum.max(1e-10).sqrt();
                } else {
                    l[i][j] = sum / l[j][j].max(1e-10);
                }
            }
        }

        l
    }

    /// Repair covariance matrix to ensure positive definiteness.
    fn repair_covariance(&mut self) {
        // Enforce symmetry
        for i in 0..self.dim {
            for j in (i + 1)..self.dim {
                let avg = (self.cov[i][j] + self.cov[j][i]) / 2.0;
                self.cov[i][j] = avg;
                self.cov[j][i] = avg;
            }
        }

        // Ensure positive diagonal
        for i in 0..self.dim {
            self.cov[i][i] = self.cov[i][i].max(1e-10);
        }

        // Limit condition number by ensuring off-diagonal elements aren't too large
        for i in 0..self.dim {
            for j in 0..self.dim {
                if i != j {
                    let max_val = (self.cov[i][i] * self.cov[j][j]).sqrt() * 0.99;
                    self.cov[i][j] = self.cov[i][j].clamp(-max_val, max_val);
                }
            }
        }
    }
}
