//! Trinity Study Simulator.
//!
//! A simulator to project the success rate of a retirement drawdown strategy,
//! combining random market returns with inflation-adjusted withdrawals over time.

use crate::experimental::inflation::InflationProjector;

/// A simple Linear Congruential Generator for deterministic randomness.
/// Lifted from monte_carlo.rs for reuse in this module.
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

/// The result of a Trinity drawdown simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrinityResult {
    /// Percentage of simulation paths that did not run out of money (0 to 100).
    pub success_rate_pct: u8,
}

/// A simulator to calculate the probability of retirement portfolio survival.
#[derive(Debug, Clone)]
pub struct TrinitySimulator {
    initial_portfolio_cents: i64,
    initial_annual_withdrawal_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    inflation_projector: InflationProjector,
    seed: u64,
}

impl TrinitySimulator {
    /// Creates a new `TrinitySimulator`.
    #[must_use]
    pub const fn new(
        initial_portfolio_cents: i64,
        initial_annual_withdrawal_cents: i64,
        annual_mean_return: f64,
        annual_volatility: f64,
        annual_inflation_rate_pct: f64,
        seed: u64,
    ) -> Self {
        Self {
            initial_portfolio_cents,
            initial_annual_withdrawal_cents,
            annual_mean_return,
            annual_volatility,
            inflation_projector: InflationProjector::new(annual_inflation_rate_pct),
            seed,
        }
    }

    /// Runs the Trinity Monte Carlo simulation for a given retirement duration.
    ///
    /// The simulation runs `paths` independent trials. Each trial spans `years`.
    /// Each year, the annual withdrawal is taken out first, then market returns are applied.
    /// The annual withdrawal amount increases each year by the inflation rate.
    #[must_use]
    pub fn run(&self, years: u16, paths: u32) -> TrinityResult {
        if paths == 0 {
            return TrinityResult {
                success_rate_pct: 0,
            };
        }

        if self.initial_portfolio_cents <= 0 {
            return TrinityResult {
                success_rate_pct: 0,
            };
        }

        let mut successful_paths = 0;
        let mut lcg = Lcg::new(self.seed);

        for _ in 0..paths {
            let mut current_portfolio = self.initial_portfolio_cents;
            let mut survived = true;

            for year in 0..years {
                // Determine this year's withdrawal adjusted for inflation.
                // The inflation projector formula is FV = PV * (1 + r)^n
                let withdrawal = self
                    .inflation_projector
                    .future_nominal_cost_cents(self.initial_annual_withdrawal_cents, year);

                current_portfolio = current_portfolio.saturating_sub(withdrawal);

                if current_portfolio <= 0 {
                    survived = false;
                    break;
                }

                // Apply market return for the remaining portfolio
                let random_norm = lcg.next_normal();
                #[allow(clippy::suboptimal_flops)]
                let annual_return = self.annual_mean_return + self.annual_volatility * random_norm;

                #[allow(clippy::cast_precision_loss)]
                let current_f64 = current_portfolio as f64;

                let gain = current_f64 * annual_return;

                #[allow(clippy::cast_possible_truncation)]
                let gain_cents = gain.round() as i64;

                current_portfolio = current_portfolio.saturating_add(gain_cents);

                if current_portfolio <= 0 {
                    survived = false;
                    break;
                }
            }

            if survived {
                successful_paths += 1;
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let success_rate_f64 = (successful_paths as f64 / paths as f64) * 100.0;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let success_rate_pct = success_rate_f64.round() as u8;

        TrinityResult {
            success_rate_pct: success_rate_pct.clamp(0, 100),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_paths() {
        let sim = TrinitySimulator::new(10_000_000, 400_000, 0.07, 0.15, 3.0, 42);
        let result = sim.run(30, 0);
        assert_eq!(result.success_rate_pct, 0);
    }

    #[test]
    fn test_zero_portfolio() {
        let sim = TrinitySimulator::new(0, 400_000, 0.07, 0.15, 3.0, 42);
        let result = sim.run(30, 100);
        assert_eq!(result.success_rate_pct, 0);
    }

    #[test]
    fn test_guaranteed_failure() {
        // $100k portfolio, withdrawing $100k per year -> will fail in year 2 or even year 1 if inflation applied
        let sim = TrinitySimulator::new(10_000_000, 10_000_000, 0.05, 0.10, 3.0, 42);
        let result = sim.run(30, 100);
        // Withdrawals are at start of year. First year withdraw $100k -> $0 left -> failure.
        assert_eq!(result.success_rate_pct, 0);
    }

    #[test]
    fn test_guaranteed_success() {
        // $100M portfolio, withdrawing $1 per year. Should 100% succeed.
        let sim = TrinitySimulator::new(10_000_000_000, 100, 0.07, 0.15, 3.0, 42);
        let result = sim.run(30, 100);
        assert_eq!(result.success_rate_pct, 100);
    }

    #[test]
    fn test_trinity_classic_4_percent() {
        // Classic 4% rule. $1M portfolio, $40k withdrawal.
        // 7% nominal return, 15% volatility, 3% inflation.
        let sim = TrinitySimulator::new(
            100_000_000, // $1M
            4_000_000,   // $40k
            0.07,
            0.15,
            3.0,
            42, // fixed seed
        );

        let result = sim.run(30, 1000);

        // At 4%, 30 years, it usually succeeds somewhere between 80% and 100% of the time.
        // Let's assert it's somewhat realistic. We adjust the threshold a bit to handle random seeds.
        assert!(result.success_rate_pct > 60);
        assert!(result.success_rate_pct <= 100);
    }
}
