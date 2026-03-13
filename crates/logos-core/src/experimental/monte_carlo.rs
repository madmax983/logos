//! Monte Carlo Investment Simulator.
//!
//! A simulator to project a range of possible future net worth outcomes
//! over a sequence of months using randomized market returns.
//!
//! This module avoids heavy external dependencies like `rand` by utilizing
//! an inline Linear Congruential Generator (LCG) and uniform sum approximation
//! (Irwin-Hall) for normally distributed random numbers.

/// A simple Linear Congruential Generator for deterministic randomness.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct Lcg {
    state: u64,
}

#[allow(dead_code)]
impl Lcg {
    const A: u64 = 6_364_136_223_846_793_005;
    const C: u64 = 1_442_695_040_888_963_407;

    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns a pseudo-random `u64`.
    const fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }

    /// Returns a pseudo-random `f64` in the range `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        #[allow(clippy::cast_precision_loss)]
        let result = value as f64 * (1.0 / (1u64 << 53) as f64);
        result
    }

    /// Approximates a standard normal distribution (mean 0, stddev 1)
    /// using the Irwin-Hall distribution (sum of 12 uniform randoms minus 6).
    fn next_normal(&mut self) -> f64 {
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += self.next_f64();
        }
        sum - 6.0
    }
}

/// The result of a Monte Carlo simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct MonteCarloResult {
    /// 5th percentile outcome in cents.
    pub(crate) p5: i64,
    /// 50th percentile (median) outcome in cents.
    pub(crate) median: i64,
    /// 95th percentile outcome in cents.
    pub(crate) p95: i64,
}

/// A projector to simulate many possible future paths for investments.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct MonteCarloProjector {
    initial_cents: i64,
    monthly_contribution_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

#[allow(dead_code)]
impl MonteCarloProjector {
    /// Creates a new `MonteCarloProjector`.
    #[must_use]
    pub(crate) const fn new(
        initial_cents: i64,
        monthly_contribution_cents: i64,
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
    ) -> Self {
        Self {
            initial_cents,
            monthly_contribution_cents,
            annual_mean_return,
            annual_volatility,
            seed,
        }
    }

    /// Runs the Monte Carlo simulation for a given number of months and paths.
    #[must_use]
    pub(crate) fn run(&self, months: u16, paths: u32) -> MonteCarloResult {
        if paths == 0 {
            return MonteCarloResult {
                p5: self.initial_cents,
                median: self.initial_cents,
                p95: self.initial_cents,
            };
        }

        let monthly_mean = self.annual_mean_return / 12.0;
        let monthly_volatility = self.annual_volatility / 12.0f64.sqrt();
        let mut lcg = Lcg::new(self.seed);

        let mut final_outcomes: Vec<i64> = Vec::with_capacity(paths as usize);

        for _ in 0..paths {
            let mut current_cents = self.initial_cents;

            for _ in 0..months {
                let random_norm = lcg.next_normal();
                let monthly_return = monthly_volatility.mul_add(random_norm, monthly_mean);

                #[allow(clippy::cast_precision_loss)]
                let current_f64 = current_cents as f64;

                let gain = current_f64 * monthly_return;

                #[allow(clippy::cast_possible_truncation)]
                let gain_cents = gain.round() as i64;

                current_cents = current_cents
                    .saturating_add(gain_cents)
                    .saturating_add(self.monthly_contribution_cents);
            }

            final_outcomes.push(current_cents);
        }

        final_outcomes.sort_unstable();

        // Calculate percentiles
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let low_index = (f64::from(paths) * 0.05).floor() as usize;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let median_index = (f64::from(paths) * 0.50).floor() as usize;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let high_index = (f64::from(paths) * 0.95).floor() as usize;

        // Ensure indices are within bounds (for very small path counts)
        let low_index = low_index.clamp(0, paths.saturating_sub(1) as usize);
        let median_index = median_index.clamp(0, paths.saturating_sub(1) as usize);
        let high_index = high_index.clamp(0, paths.saturating_sub(1) as usize);

        MonteCarloResult {
            p5: final_outcomes[low_index],
            median: final_outcomes[median_index],
            p95: final_outcomes[high_index],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_projector_run() {
        let projector = MonteCarloProjector::new(
            10_000_000, // $100,000 initial
            100_000,    // $1,000 monthly contribution
            0.07,       // 7% annual return
            0.15,       // 15% volatility
            42,         // fixed seed
        );

        let result = projector.run(120, 1000); // 10 years, 1000 paths

        // We expect the median to be roughly $100k + $120k + growth > $220k
        assert!(result.median > 22_000_000);

        // P5 should be less than Median, and Median should be less than P95
        assert!(result.p5 < result.median);
        assert!(result.median < result.p95);
    }

    #[test]
    fn test_zero_paths() {
        let projector = MonteCarloProjector::new(10_000, 10_000, 0.07, 0.15, 42);
        let result = projector.run(12, 0);
        assert_eq!(result.p5, 10_000);
        assert_eq!(result.median, 10_000);
        assert_eq!(result.p95, 10_000);
    }
}
