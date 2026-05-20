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
            final_outcomes.push(self.simulate_path(
                months,
                monthly_mean,
                monthly_volatility,
                &mut lcg,
            ));
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

    fn simulate_path(
        &self,
        months: u16,
        monthly_mean: f64,
        monthly_volatility: f64,
        lcg: &mut Lcg,
    ) -> i64 {
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

        current_cents
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

    #[test]
    fn test_percentile_indices_and_math() {
        let projector = MonteCarloProjector::new(10_000, 1_000, 0.0, 0.0, 42);
        // Using 10 paths.
        // 5th percentile index = 10 * 0.05 = 0.5 -> floor = 0
        // Median index = 10 * 0.50 = 5.0 -> floor = 5
        // 95th percentile index = 10 * 0.95 = 9.5 -> floor = 9
        let result = projector.run(1, 10);
        // All paths start at 10000, gain is 0, plus 1000 contribution = 11000
        assert_eq!(result.p5_cents, 11_000);
        assert_eq!(result.median_cents, 11_000);
        assert_eq!(result.p95_cents, 11_000);
    }

    #[test]
    fn test_math_operators_and_bounds() {
        // By using a very distinct volatility and mean, we can ensure the resulting
        // math doesn't accidentally pass if + is swapped for - or *.
        let projector = MonteCarloProjector::new(100_000_000, 0, 0.12, 0.20, 42);
        let result = projector.run(12, 100);

        // If monthly_mean / 12.0 is mutated to % 12.0, it will be wildly different
        // If monthly_return = mean + vol * norm is mutated to -, it will invert direction
        // If it's mutated to *, it will be near 0
        assert!(result.p5_cents > 0);
        assert!(result.p95_cents > 0);
        assert!(result.median_cents > 0);

        // To kill the * 0.95 mutated to + 0.95, let's use enough paths that indices differ.
        // 100 paths: 95th is index 95. If mutated to `100 + 0.95` -> 100.
        // But clamp will pull it to 99, which has a different value.
        assert_ne!(result.p5_cents, result.p95_cents);
        assert_ne!(result.median_cents, result.p95_cents);

        // Exact percentiles validation for a known seed
        // The original outputs for seed 42 with 100 paths over 12 months:
        // By locking down the exact output, we catch any changes to + / - / %
        assert_eq!(result.p5_cents, 80_246_193);
        assert_eq!(result.median_cents, 110_558_507);
        assert_eq!(result.p95_cents, 154_640_458);
    }

    #[test]
    fn test_next_normal_range() {
        let mut lcg = Lcg::new(42);
        let _n1 = lcg.next_normal();
        // Sum of 12 uniform randoms in [0, 1) minus 6 should be near 0
        // If mutated to `/ 6.0`, it would be `sum / 6.0` which is roughly `6 / 6 = 1.0`
        // But specifically, the standard deviation is 1, so roughly [-3, 3]
        // Let's just check it isn't wildly off. The actual first value for seed 42
        // with this LCG is roughly -1.33. If mutated to division, it would be around +0.77.
        // We'll just generate many and calculate the mean to be sure it's around 0, not 1.
        let mut sum = 0.0;
        for _ in 0..100 {
            sum += lcg.next_normal();
        }
        let mean = sum / 100.0;
        assert!(
            mean < 0.5 && mean > -0.5,
            "Mean should be near 0, got {mean}"
        );
    }
}
