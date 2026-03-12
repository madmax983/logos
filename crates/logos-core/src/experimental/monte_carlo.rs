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
struct Lcg {
    state: u64,
}

impl Lcg {
    const A: u64 = 6_364_136_223_846_793_005;
    const C: u64 = 1_442_695_040_888_963_407;

    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns a pseudo-random `u64`.
    #[allow(clippy::missing_const_for_fn)]
    fn next_u64(&mut self) -> u64 {
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
pub struct MonteCarloResult {
    /// 5th percentile outcome in cents.
    pub p5_cents: i64,
    /// 50th percentile (median) outcome in cents.
    pub median_cents: i64,
    /// 95th percentile outcome in cents.
    pub p95_cents: i64,
}

/// A projector to simulate many possible future paths for investments.
#[derive(Debug, Clone)]
pub struct MonteCarloProjector {
    initial_cents: i64,
    monthly_contribution_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

impl MonteCarloProjector {
    /// Creates a new `MonteCarloProjector`.
    #[must_use]
    pub const fn new(
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
    pub fn run(&self, months: u16, paths: u32) -> MonteCarloResult {
        if paths == 0 {
            return MonteCarloResult {
                p5_cents: self.initial_cents,
                median_cents: self.initial_cents,
                p95_cents: self.initial_cents,
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
                #[allow(clippy::suboptimal_flops)]
                let monthly_return = monthly_mean + monthly_volatility * random_norm;

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
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let lower_bound_idx = (f64::from(paths) * 0.05).floor() as usize;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let median_idx = (f64::from(paths) * 0.50).floor() as usize;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let upper_bound_idx = (f64::from(paths) * 0.95).floor() as usize;

        // Ensure indices are within bounds (for very small path counts)
        let safe_lower_bound_idx = lower_bound_idx.clamp(0, paths.saturating_sub(1) as usize);
        let safe_median_idx = median_idx.clamp(0, paths.saturating_sub(1) as usize);
        let safe_upper_bound_idx = upper_bound_idx.clamp(0, paths.saturating_sub(1) as usize);

        MonteCarloResult {
            p5_cents: final_outcomes[safe_lower_bound_idx],
            median_cents: final_outcomes[safe_median_idx],
            p95_cents: final_outcomes[safe_upper_bound_idx],
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
        assert!(result.median_cents > 22_000_000);

        // P5 should be less than Median, and Median should be less than P95
        assert!(result.p5_cents < result.median_cents);
        assert!(result.median_cents < result.p95_cents);
    }

    #[test]
    fn test_zero_paths() {
        let projector = MonteCarloProjector::new(10_000, 10_000, 0.07, 0.15, 42);
        let result = projector.run(12, 0);
        assert_eq!(result.p5_cents, 10_000);
        assert_eq!(result.median_cents, 10_000);
        assert_eq!(result.p95_cents, 10_000);
    }
}
