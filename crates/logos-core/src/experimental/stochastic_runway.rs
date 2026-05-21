#![cfg(feature = "nova")]

//! Stochastic Runway Simulator
//!
//! A simulator that calculates how long liquid assets will last given a monthly burn rate,
//! inflation, and randomized market returns (Monte Carlo).
//!
//! 🌟 Nova Mashup: We mash this up with the `RunwaySimulator` and `MonteCarloProjector`.

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StochasticRunwayResult {
    /// 5th percentile outcome for runway months
    pub p5_months: u32,
    /// 50th percentile (median) outcome for runway months
    pub median_months: u32,
    /// 95th percentile outcome for runway months
    pub p95_months: u32,
}

#[derive(Debug, Clone)]
pub struct StochasticRunwaySimulator {
    liquid_assets_cents: i64,
    monthly_burn_cents: i64,
    annual_inflation_pct: f64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

impl StochasticRunwaySimulator {
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

    #[must_use]
    pub fn calculate_runway(&self, paths: u32) -> StochasticRunwayResult {
        if paths == 0 {
            return StochasticRunwayResult {
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
        let mut final_outcomes: Vec<u32> = Vec::with_capacity(paths as usize);

        for _ in 0..paths {
            let mut current_assets = self.liquid_assets_cents;
            #[allow(clippy::cast_precision_loss)]
            let mut current_burn = self.monthly_burn_cents as f64;
            let mut months = 0;

            while current_assets > 0 && months < 1200 {
                #[allow(clippy::cast_possible_truncation)]
                let current_burn_cents = current_burn.round() as i64;

                if current_burn_cents == 0 {
                    months = 1200;
                    break;
                }

                let actual_burn = current_assets.min(current_burn_cents);
                current_assets -= actual_burn;
                months += 1;

                if current_assets == 0 {
                    break;
                }

                let random_norm = lcg.next_normal();
                #[allow(clippy::suboptimal_flops)]
                let monthly_return = monthly_mean + monthly_volatility * random_norm;

                #[allow(clippy::cast_precision_loss)]
                let current_f64 = current_assets as f64;
                let gain = current_f64 * monthly_return;

                #[allow(clippy::cast_possible_truncation)]
                let gain_cents = gain.round() as i64;

                current_assets = current_assets.saturating_add(gain_cents);
                if current_assets < 0 {
                    current_assets = 0;
                }

                current_burn *= 1.0 + monthly_inflation_rate;
            }

            final_outcomes.push(months);
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

        StochasticRunwayResult {
            p5_months: final_outcomes[safe_lower_bound_idx],
            median_months: final_outcomes[safe_median_idx],
            p95_months: final_outcomes[safe_upper_bound_idx],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_runway() {
        let sim = StochasticRunwaySimulator::new(100_000_000, 100_000, 3.0, 0.07, 0.15, 42);
        let result = sim.calculate_runway(100);
        assert!(result.p5_months <= result.median_months);
        assert!(result.median_months <= result.p95_months);
        assert!(result.p5_months > 0);
    }

    #[test]
    fn test_stochastic_runway_zero_paths() {
        let sim = StochasticRunwaySimulator::new(100_000_000, 100_000, 3.0, 0.07, 0.15, 42);
        let result = sim.calculate_runway(0);
        assert_eq!(result.p5_months, 0);
        assert_eq!(result.median_months, 0);
        assert_eq!(result.p95_months, 0);
    }
}
