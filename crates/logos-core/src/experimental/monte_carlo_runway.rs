#![cfg(feature = "nova")]

//! Monte Carlo Runway Simulator.
//!
//! A simulator to project a range of possible future runway outcomes
//! over a sequence of months using randomized market returns.
//!
//! 🌟 Nova Mashup: We mash this up with the `RunwaySimulator` and `MonteCarloProjector`
//! so you can see the probability of survival over time.

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

/// The result of a Monte Carlo Runway simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonteCarloRunwayResult {
    /// 5th percentile outcome in months.
    pub p5_months: u32,
    /// 50th percentile (median) outcome in months.
    pub median_months: u32,
    /// 95th percentile outcome in months.
    pub p95_months: u32,
}

/// A simulator for calculating financial runway with randomized market returns.
#[derive(Debug, Clone)]
pub struct MonteCarloRunwaySimulator {
    liquid_assets_cents: i64,
    monthly_burn_cents: i64,
    annual_inflation_pct: f64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

impl MonteCarloRunwaySimulator {
    /// Creates a new `MonteCarloRunwaySimulator`.
    #[must_use]
    pub const fn new(
        liquid_assets_cents: i64,
        monthly_burn_cents: i64,
        annual_inflation_pct: f64,
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
    ) -> Self {
        Self {
            liquid_assets_cents,
            monthly_burn_cents,
            annual_inflation_pct,
            annual_mean_return,
            annual_volatility,
            seed,
        }
    }

    /// Calculates the Monte Carlo runway.
    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn calculate_runway(&self, paths: u32) -> MonteCarloRunwayResult {
        if paths == 0 {
            return MonteCarloRunwayResult {
                p5_months: 0,
                median_months: 0,
                p95_months: 0,
            };
        }

        let monthly_mean = self.annual_mean_return / 12.0;
        let monthly_volatility = self.annual_volatility / 12.0f64.sqrt();
        let monthly_inflation_rate = if self.annual_inflation_pct > 0.0 {
            (1.0 + self.annual_inflation_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        let mut lcg = Lcg::new(self.seed);
        let mut final_outcomes = Vec::with_capacity(paths as usize);

        for _ in 0..paths {
            let mut months = 0;
            let mut current_assets = self.liquid_assets_cents;
            #[allow(clippy::cast_precision_loss)]
            let mut current_burn = self.monthly_burn_cents as f64;

            while current_assets > 0 && months < 1200 {
                let random_norm = lcg.next_normal();
                #[allow(clippy::suboptimal_flops)]
                let monthly_return = monthly_mean + monthly_volatility * random_norm;

                #[allow(clippy::cast_precision_loss)]
                let current_f64 = current_assets as f64;
                let gain = current_f64 * monthly_return;

                #[allow(clippy::cast_possible_truncation)]
                let gain_cents = gain.round() as i64;

                current_assets = current_assets.saturating_add(gain_cents);

                #[allow(clippy::cast_possible_truncation)]
                let burn_cents = current_burn.round() as i64;

                current_assets = current_assets.saturating_sub(burn_cents);

                current_burn *= 1.0 + monthly_inflation_rate;
                months += 1;
            }

            final_outcomes.push(months);
        }

        final_outcomes.sort_unstable();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx_p5 = (f64::from(paths) * 0.05) as usize;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx_median = (f64::from(paths) * 0.50) as usize;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx_p95 = (f64::from(paths) * 0.95) as usize;

        let idx_p5 = idx_p5.min(paths as usize - 1);
        let idx_median = idx_median.min(paths as usize - 1);
        let idx_p95 = idx_p95.min(paths as usize - 1);

        MonteCarloRunwayResult {
            p5_months: final_outcomes[idx_p5],
            median_months: final_outcomes[idx_median],
            p95_months: final_outcomes[idx_p95],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_runway_zero_paths() {
        let sim = MonteCarloRunwaySimulator::new(100_000, 10_000, 0.0, 0.0, 0.0, 42);
        let result = sim.calculate_runway(0);
        assert_eq!(result.p5_months, 0);
        assert_eq!(result.median_months, 0);
        assert_eq!(result.p95_months, 0);
    }

    #[test]
    fn test_monte_carlo_runway_basic() {
        // $100k liquid, $2k monthly burn, 3% inflation, 7% return, 15% volatility
        let sim = MonteCarloRunwaySimulator::new(10_000_000, 200_000, 3.0, 0.07, 0.15, 12345);
        let result = sim.calculate_runway(1000);

        // P5 should be lower than Median, Median should be lower than P95
        assert!(result.p5_months <= result.median_months);
        assert!(result.median_months <= result.p95_months);

        // Without inflation and return, $100k / $2k = 50 months.
        assert!(result.median_months > 0);
    }
}
