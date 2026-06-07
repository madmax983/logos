//! Stochastic Runway Simulator
//!
//! 🌟 Nova Mashup: Combines `RunwaySimulator` with `MonteCarloProjector`.
//! Calculates how long your liquid assets will last when invested in a volatile market,
//! while accounting for a monthly burn rate and inflation.
//!
//! This is crucial because a standard runway calculator assumes flat returns. If your
//! runway is invested and the market drops 20% in the first month (Sequence of Returns Risk),
//! your runway might be drastically shorter than expected.

/// Represents the simulation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StochasticRunwayResult {
    /// The 5th percentile runway in months (pessimistic outcome).
    pub p5_months: u32,
    /// The median runway in months.
    pub median_months: u32,
    /// The 95th percentile runway in months (optimistic outcome).
    pub p95_months: u32,
}

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

/// A simulator for calculating financial runway with stochastic market returns.
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
    /// Creates a new `StochasticRunwaySimulator`.
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

    /// Simulates the runway until assets are depleted across multiple randomized paths.
    #[must_use]
    pub fn run(&self, paths: u32) -> StochasticRunwayResult {
        if paths == 0 {
            return StochasticRunwayResult {
                p5_months: 0,
                median_months: 0,
                p95_months: 0,
            };
        }

        let monthly_mean = self.annual_mean_return / 12.0;
        let monthly_volatility = self.annual_volatility / 12.0f64.sqrt();
        let mut lcg = Lcg::new(self.seed);

        let monthly_inflation_rate = if self.annual_inflation_pct > 0.0 {
            (1.0 + self.annual_inflation_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        let mut runway_outcomes: Vec<u32> = Vec::with_capacity(paths as usize);

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

            runway_outcomes.push(months);
        }

        runway_outcomes.sort_unstable();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::similar_names)]
        let p5_idx = ((f64::from(paths) * 0.05).floor() as usize).min(paths as usize - 1);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_sign_loss)]
        let median_idx = ((f64::from(paths) * 0.50).floor() as usize).min(paths as usize - 1);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::similar_names)]
        #[allow(clippy::similar_names)]
        let percentile_95_idx =
            ((f64::from(paths) * 0.95).floor() as usize).min(paths as usize - 1);

        StochasticRunwayResult {
            p5_months: runway_outcomes[p5_idx],
            median_months: runway_outcomes[median_idx],
            p95_months: runway_outcomes[percentile_95_idx],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_runway_no_volatility() {
        let sim = StochasticRunwaySimulator::new(1_000_0000, 1_000_000, 0.0, 0.0, 0.0, 42);
        let result = sim.run(100);
        assert_eq!(result.median_months, 10);
        assert_eq!(result.p5_months, 10);
        assert_eq!(result.p95_months, 10);
    }

    #[test]
    fn test_stochastic_runway_with_volatility() {
        let sim = StochasticRunwaySimulator::new(1_000_0000, 500_000, 0.0, 0.08, 0.15, 42);
        let result = sim.run(1000);

        assert!(result.p5_months <= result.median_months);
        assert!(result.median_months <= result.p95_months);
        assert!(result.p5_months < result.p95_months);
    }
}
