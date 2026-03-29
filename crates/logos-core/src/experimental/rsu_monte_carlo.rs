//! Monte Carlo RSU Vest Simulator
//!
//! Connecting `rsu.rs` and `monte_carlo.rs`.
//!
//! RSUs are highly volatile. Instead of using a static haircut tier table,
//! this simulator uses a Monte Carlo approach (Brownian Motion approximation)
//! to project the possible range of future RSU vest values based on annual
//! volatility and mean return.

use crate::planning::fire::UpcomingVest;

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

    #[allow(clippy::missing_const_for_fn)]
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        #[allow(clippy::cast_precision_loss)]
        let result = value as f64 * (1.0 / (1u64 << 53) as f64);
        result
    }

    fn next_normal(&mut self) -> f64 {
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += self.next_f64();
        }
        sum - 6.0
    }
}

/// The result of an RSU Monte Carlo simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsuMonteCarloResult {
    /// 5th percentile outcome of total vested value in cents.
    pub p5_cents: i64,
    /// 50th percentile (median) outcome of total vested value in cents.
    pub median_cents: i64,
    /// 95th percentile outcome of total vested value in cents.
    pub p95_cents: i64,
}

/// A projector to simulate many possible future paths for upcoming RSU vests.
#[derive(Debug, Clone)]
pub struct RsuMonteCarloProjector {
    upcoming_vests: Vec<UpcomingVest>,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

impl RsuMonteCarloProjector {
    /// Creates a new `RsuMonteCarloProjector`.
    #[must_use]
    pub const fn new(
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
    ) -> Self {
        Self {
            upcoming_vests: Vec::new(),
            annual_mean_return,
            annual_volatility,
            seed,
        }
    }

    /// Adds an upcoming vest to the projection.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Runs the Monte Carlo simulation for the configured vests.
    #[must_use]
    pub fn run(&self, paths: u32) -> RsuMonteCarloResult {
        if paths == 0 || self.upcoming_vests.is_empty() {
            return RsuMonteCarloResult {
                p5_cents: 0,
                median_cents: 0,
                p95_cents: 0,
            };
        }

        // Daily drift and volatility assuming 252 trading days
        let daily_mean = self.annual_mean_return / 252.0;
        let daily_volatility = self.annual_volatility / 252.0f64.sqrt();
        let mut lcg = Lcg::new(self.seed);

        let mut final_outcomes: Vec<i64> = Vec::with_capacity(paths as usize);

        for _ in 0..paths {
            let mut path_total_cents = 0_i64;

            for vest in &self.upcoming_vests {
                #[allow(clippy::cast_precision_loss)]
                let mut current_price = vest.avg_close_price_cents as f64;

                for _ in 0..vest.days_to_vest {
                    let random_norm = lcg.next_normal();
                    #[allow(clippy::suboptimal_flops)]
                    let daily_return = daily_mean + daily_volatility * random_norm;
                    current_price *= 1.0 + daily_return;
                }

                #[allow(clippy::cast_possible_truncation)]
                let vest_value = (current_price * f64::from(vest.units)).round() as i64;

                path_total_cents = path_total_cents.saturating_add(vest_value.max(0));
            }

            final_outcomes.push(path_total_cents);
        }

        final_outcomes.sort_unstable();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let lower_bound_idx = (f64::from(paths) * 0.05).floor() as usize;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let median_idx = (f64::from(paths) * 0.50).floor() as usize;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let upper_bound_idx = (f64::from(paths) * 0.95).floor() as usize;

        let safe_lower_bound_idx = lower_bound_idx.clamp(0, paths.saturating_sub(1) as usize);
        let safe_median_idx = median_idx.clamp(0, paths.saturating_sub(1) as usize);
        let safe_upper_bound_idx = upper_bound_idx.clamp(0, paths.saturating_sub(1) as usize);

        RsuMonteCarloResult {
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
    fn test_rsu_monte_carlo_projector_run() {
        let mut projector = RsuMonteCarloProjector::new(
            0.10, // 10% annual return
            0.50, // 50% volatility (high, typical for single tech stock)
            42,   // fixed seed
        );

        projector.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000, // $100
            units: 100, // 100 units -> $10k current value
            days_to_vest: 252, // 1 year out
        });

        let result = projector.run(1000); // 1000 paths

        // Median should be close to $10k + 10% = $11k
        assert!(result.median_cents > 900_000);
        assert!(result.median_cents < 1_300_000);

        // High volatility means P5 will be very low (e.g. < $5k) and P95 very high (e.g. > $20k)
        assert!(result.p5_cents < result.median_cents);
        assert!(result.median_cents < result.p95_cents);
    }

    #[test]
    fn test_zero_paths_or_vests() {
        let mut projector = RsuMonteCarloProjector::new(0.07, 0.15, 42);

        let result_no_vests = projector.run(100);
        assert_eq!(result_no_vests.p5_cents, 0);
        assert_eq!(result_no_vests.median_cents, 0);
        assert_eq!(result_no_vests.p95_cents, 0);

        projector.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 100,
            days_to_vest: 30,
        });

        let result_zero_paths = projector.run(0);
        assert_eq!(result_zero_paths.p5_cents, 0);
        assert_eq!(result_zero_paths.median_cents, 0);
        assert_eq!(result_zero_paths.p95_cents, 0);
    }
}
