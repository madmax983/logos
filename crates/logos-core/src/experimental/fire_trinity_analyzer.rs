//! FIRE Trinity Analyzer Module.
//!
//! Answers the question: "If I retired today based on my `FireSimulator` progress,
//! what is the real probability of my portfolio surviving a 30-year drawdown
//! according to the Trinity Study?"

use crate::experimental::trinity_simulator::{TrinitySimulator, TrinityResult};
use crate::planning::fire::FireSimulator;

/// A bridge that evaluates a FIRE plan's resilience using the Trinity Simulator.
#[derive(Debug, Clone)]
pub struct FireTrinityAnalyzer {
    annual_mean_return: f64,
    annual_volatility: f64,
    annual_inflation_rate_pct: f64,
    seed: u64,
}

impl FireTrinityAnalyzer {
    /// Creates a new `FireTrinityAnalyzer`.
    #[must_use]
    pub const fn new(
        annual_mean_return: f64,
        annual_volatility: f64,
        annual_inflation_rate_pct: f64,
        seed: u64,
    ) -> Self {
        Self {
            annual_mean_return,
            annual_volatility,
            annual_inflation_rate_pct,
            seed,
        }
    }

    /// Evaluates the survival probability if retiring today.
    #[must_use]
    pub fn evaluate_retiring_today(&self, fire_sim: &FireSimulator) -> TrinityResult {
        let initial_portfolio_cents = fire_sim.safe_net_worth_cents();
        let initial_annual_withdrawal_cents = fire_sim.monthly_expenses_cents().saturating_mul(12);

        let simulator = TrinitySimulator::new(
            initial_portfolio_cents,
            initial_annual_withdrawal_cents,
            self.annual_mean_return,
            self.annual_volatility,
            self.annual_inflation_rate_pct,
            self.seed,
        );

        simulator.run(30, 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::{FireConfig, FireSimulator};

    #[test]
    fn test_evaluate_retiring_today_success() {
        let mut fire_sim = FireSimulator::new(400_000); // $4,000 / mo
        fire_sim.set_config(FireConfig { safe_withdrawal_rate_pct: 4 });

        // Inject enough assets to be at FIRE ($1.2M)
        fire_sim.add_assets_liabilities(120_000_000, 0);

        let analyzer = FireTrinityAnalyzer::new(0.07, 0.15, 3.0, 42);

        let result = analyzer.evaluate_retiring_today(&fire_sim);
        assert!(result.success_rate_pct > 60);
    }
}
