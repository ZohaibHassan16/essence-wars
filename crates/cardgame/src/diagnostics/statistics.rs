//! Statistical analysis utilities.
//!
//! Provides confidence intervals, significance testing, and correlation analysis
//! for diagnostic metrics.

use statrs::distribution::{ChiSquared, ContinuousCDF, Normal};

/// Wilson score confidence interval for a proportion.
///
/// This method is preferred over the normal approximation because it
/// performs well even with small sample sizes or extreme proportions.
///
/// # Arguments
/// * `successes` - Number of successes (e.g., P1 wins)
/// * `total` - Total number of trials (e.g., total games)
/// * `confidence` - Confidence level (e.g., 0.95 for 95% CI)
///
/// # Returns
/// Tuple of (lower_bound, upper_bound) for the confidence interval.
pub fn wilson_score_interval(successes: usize, total: usize, confidence: f64) -> (f64, f64) {
    if total == 0 {
        return (0.0, 1.0);
    }

    let n = total as f64;
    let p = successes as f64 / n;

    // Z-score for the confidence level
    let normal = Normal::new(0.0, 1.0).unwrap();
    let alpha = 1.0 - confidence;
    let z = normal.inverse_cdf(1.0 - alpha / 2.0);
    let z2 = z * z;

    let denominator = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator;
    let margin = (z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt()) / denominator;

    let lower = (center - margin).max(0.0);
    let upper = (center + margin).min(1.0);

    (lower, upper)
}

/// Chi-square goodness-of-fit test for comparing observed vs expected proportions.
///
/// Tests whether an observed proportion differs significantly from an expected value.
///
/// # Arguments
/// * `observed_successes` - Observed number of successes
/// * `total` - Total number of trials
/// * `expected_proportion` - Expected proportion under null hypothesis (e.g., 0.5)
///
/// # Returns
/// * `(chi_square, p_value)` - The chi-square statistic and p-value
pub fn chi_square_test(
    observed_successes: usize,
    total: usize,
    expected_proportion: f64,
) -> (f64, f64) {
    if total == 0 {
        return (0.0, 1.0);
    }

    let n = total as f64;
    let observed = observed_successes as f64;
    let expected = n * expected_proportion;

    // Chi-square = sum of (observed - expected)^2 / expected
    // For binary outcomes (success/failure):
    let obs_failure = n - observed;
    let exp_failure = n * (1.0 - expected_proportion);

    let chi_sq_success = if expected > 0.0 {
        (observed - expected).powi(2) / expected
    } else {
        0.0
    };
    let chi_sq_failure = if exp_failure > 0.0 {
        (obs_failure - exp_failure).powi(2) / exp_failure
    } else {
        0.0
    };

    let chi_square = chi_sq_success + chi_sq_failure;

    // Degrees of freedom = 1 for binary test
    let chi_dist = ChiSquared::new(1.0).unwrap();
    let p_value = 1.0 - chi_dist.cdf(chi_square);

    (chi_square, p_value)
}

/// Calculate percentiles from a sorted slice.
///
/// # Arguments
/// * `values` - Slice of values (must be sorted)
/// * `percentile` - Percentile to calculate (0.0 to 1.0)
///
/// # Returns
/// The value at the specified percentile.
pub fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }

    let n = sorted_values.len();
    if n == 1 {
        return Some(sorted_values[0]);
    }

    let p = percentile.clamp(0.0, 1.0);
    let idx = p * (n - 1) as f64;
    let lower_idx = idx.floor() as usize;
    let upper_idx = idx.ceil() as usize;

    if lower_idx == upper_idx {
        Some(sorted_values[lower_idx])
    } else {
        let fraction = idx - lower_idx as f64;
        Some(sorted_values[lower_idx] * (1.0 - fraction) + sorted_values[upper_idx] * fraction)
    }
}

/// Calculate mean and variance.
///
/// # Returns
/// * `(mean, variance, count)` tuple
pub fn mean_variance(values: &[f64]) -> (f64, f64, usize) {
    let n = values.len();
    if n == 0 {
        return (0.0, 0.0, 0);
    }

    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };

    (mean, variance, n)
}

/// Calculate Pearson correlation coefficient.
///
/// # Arguments
/// * `x` - First variable values
/// * `y` - Second variable values (must have same length as x)
///
/// # Returns
/// * `Some(r)` - Correlation coefficient (-1 to 1)
/// * `None` - If insufficient data or zero variance
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }

    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }

    let denominator = (sum_x2 * sum_y2).sqrt();
    if denominator == 0.0 {
        return None;
    }

    Some(sum_xy / denominator)
}

/// Calculate p-value for a Pearson correlation coefficient.
///
/// Uses t-distribution approximation.
///
/// # Arguments
/// * `r` - Correlation coefficient
/// * `n` - Sample size
///
/// # Returns
/// Two-tailed p-value for testing r != 0
pub fn correlation_p_value(r: f64, n: usize) -> f64 {
    if n < 3 {
        return 1.0;
    }

    let df = n as f64 - 2.0;
    let t = r * (df / (1.0 - r * r)).sqrt();

    // Two-tailed p-value using t-distribution
    // Approximation using normal for large n
    let normal = Normal::new(0.0, 1.0).unwrap();
    let p_one_tail = 1.0 - normal.cdf(t.abs());
    2.0 * p_one_tail
}

/// Significance level interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignificanceLevel {
    /// p >= 0.10 - Not significant
    NotSignificant,
    /// 0.05 <= p < 0.10 - Marginally significant
    Marginal,
    /// 0.01 <= p < 0.05 - Significant
    Significant,
    /// p < 0.01 - Highly significant
    HighlySignificant,
}

impl SignificanceLevel {
    /// Determine significance level from p-value.
    pub fn from_p_value(p: f64) -> Self {
        if p < 0.01 {
            Self::HighlySignificant
        } else if p < 0.05 {
            Self::Significant
        } else if p < 0.10 {
            Self::Marginal
        } else {
            Self::NotSignificant
        }
    }

    /// Get display symbol for significance.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::HighlySignificant => "***",
            Self::Significant => "**",
            Self::Marginal => "*",
            Self::NotSignificant => "",
        }
    }

    /// Get display description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::HighlySignificant => "highly significant (p < 0.01)",
            Self::Significant => "significant (p < 0.05)",
            Self::Marginal => "marginally significant (p < 0.10)",
            Self::NotSignificant => "not significant",
        }
    }
}

/// Statistical summary for a proportion (e.g., win rate).
#[derive(Debug, Clone)]
pub struct ProportionStats {
    /// Observed proportion.
    pub proportion: f64,
    /// Number of successes.
    pub successes: usize,
    /// Total number of trials.
    pub total: usize,
    /// Lower bound of 95% CI.
    pub ci_lower: f64,
    /// Upper bound of 95% CI.
    pub ci_upper: f64,
    /// Chi-square statistic vs expected (default 0.5).
    pub chi_square: f64,
    /// P-value for chi-square test.
    pub p_value: f64,
    /// Significance level.
    pub significance: SignificanceLevel,
}

impl ProportionStats {
    /// Calculate proportion statistics.
    ///
    /// # Arguments
    /// * `successes` - Number of successes
    /// * `total` - Total number of trials
    /// * `expected` - Expected proportion (default 0.5 for balanced)
    pub fn calculate(successes: usize, total: usize, expected: f64) -> Self {
        let proportion = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };

        let (ci_lower, ci_upper) = wilson_score_interval(successes, total, 0.95);
        let (chi_square, p_value) = chi_square_test(successes, total, expected);
        let significance = SignificanceLevel::from_p_value(p_value);

        Self {
            proportion,
            successes,
            total,
            ci_lower,
            ci_upper,
            chi_square,
            p_value,
            significance,
        }
    }

    /// Format as percentage with confidence interval.
    pub fn format_with_ci(&self) -> String {
        format!(
            "{:.1}% [{:.1}-{:.1}%]",
            self.proportion * 100.0,
            self.ci_lower * 100.0,
            self.ci_upper * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wilson_score_interval() {
        // 50 out of 100 should give interval around 0.5
        let (lower, upper) = wilson_score_interval(50, 100, 0.95);
        assert!(lower < 0.5);
        assert!(upper > 0.5);
        assert!((lower - 0.4).abs() < 0.05);
        assert!((upper - 0.6).abs() < 0.05);
    }

    #[test]
    fn test_wilson_score_edge_cases() {
        // All successes
        let (lower, upper) = wilson_score_interval(100, 100, 0.95);
        assert!(lower > 0.95);
        assert!((upper - 1.0).abs() < 0.01);

        // No successes
        let (lower, upper) = wilson_score_interval(0, 100, 0.95);
        assert!(lower < 0.01);
        assert!(upper < 0.05);

        // Empty
        let (lower, upper) = wilson_score_interval(0, 0, 0.95);
        assert_eq!(lower, 0.0);
        assert_eq!(upper, 1.0);
    }

    #[test]
    fn test_chi_square_test() {
        // 50/100 vs expected 0.5 should not be significant
        let (chi_sq, p) = chi_square_test(50, 100, 0.5);
        assert!(chi_sq < 0.1);
        assert!(p > 0.05);

        // 70/100 vs expected 0.5 should be significant
        let (chi_sq, p) = chi_square_test(70, 100, 0.5);
        assert!(chi_sq > 10.0);
        assert!(p < 0.01);
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert!((percentile(&values, 0.0).unwrap() - 1.0).abs() < 0.01);
        assert!((percentile(&values, 0.5).unwrap() - 3.0).abs() < 0.01);
        assert!((percentile(&values, 1.0).unwrap() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_mean_variance() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, variance, n) = mean_variance(&values);

        assert_eq!(n, 8);
        assert!((mean - 5.0).abs() < 0.01);
        assert!((variance - 4.57).abs() < 0.1);
    }

    #[test]
    fn test_pearson_correlation() {
        // Perfect positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - 1.0).abs() < 0.01);

        // Perfect negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r_neg = pearson_correlation(&x, &y_neg).unwrap();
        assert!((r_neg + 1.0).abs() < 0.01);

        // No correlation
        let y_zero = vec![5.0, 3.0, 7.0, 2.0, 8.0];
        let r_zero = pearson_correlation(&x, &y_zero).unwrap();
        assert!(r_zero.abs() < 0.5);
    }

    #[test]
    fn test_significance_level() {
        assert_eq!(
            SignificanceLevel::from_p_value(0.001),
            SignificanceLevel::HighlySignificant
        );
        assert_eq!(
            SignificanceLevel::from_p_value(0.02),
            SignificanceLevel::Significant
        );
        assert_eq!(
            SignificanceLevel::from_p_value(0.07),
            SignificanceLevel::Marginal
        );
        assert_eq!(
            SignificanceLevel::from_p_value(0.5),
            SignificanceLevel::NotSignificant
        );
    }

    #[test]
    fn test_proportion_stats() {
        let stats = ProportionStats::calculate(65, 100, 0.5);

        assert!((stats.proportion - 0.65).abs() < 0.01);
        assert!(stats.ci_lower > 0.5);
        assert!(stats.ci_upper < 0.8);
        assert!(stats.p_value < 0.05);
        assert!(stats.significance == SignificanceLevel::Significant
            || stats.significance == SignificanceLevel::HighlySignificant);
    }
}
