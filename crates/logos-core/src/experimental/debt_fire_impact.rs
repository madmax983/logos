#![cfg(feature = "nova")]

//! Debt FIRE Impact Analyzer
//!
//! Evaluates the long-term impact of debt on your FIRE journey by combining
//! the debt payoff simulator with FIRE mechanics. It answers:
//! "How many months of my FIRE journey is this debt costing me?"
//!
//! 🌟 Nova Mashup: We mash up `DebtOptimizer` with `FireSimulator` and
//! `NetWorthProjector` concepts.

use crate::experimental::debt_optimizer::{Debt, DebtOptimizer, PayoffStrategy};
use crate::fire::FireSimulator;

/// A report detailing how a specific debt profile affects your FIRE trajectory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtFireImpactReport {
    /// The total months it will take to pay off the debt.
    pub months_to_payoff: u32,
    /// Total interest paid.
    pub total_interest_paid_cents: i64,
    /// The opportunity cost of the debt payments (principal + interest)
    /// if they had instead been invested and grown at a standard rate.
    pub opportunity_cost_cents: i64,
    /// How much higher your target FIRE number needs to be because
    /// of the monthly minimum payments required to carry this debt.
    pub fire_target_increase_cents: i64,
}

/// Evaluates debt against FIRE goals.
#[derive(Debug, Clone)]
pub struct DebtFireImpactAnalyzer {
    fire_sim: FireSimulator,
    market_growth_rate_pct: u32,
}

impl DebtFireImpactAnalyzer {
    #[must_use]
    pub const fn new(fire_sim: FireSimulator, market_growth_rate_pct: u32) -> Self {
        Self {
            fire_sim,
            market_growth_rate_pct,
        }
    }

    /// Evaluates the impact of a set of debts, given a monthly payment budget.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn evaluate(
        &self,
        debts: Vec<Debt>,
        monthly_payment_cents: i64,
        strategy: PayoffStrategy,
    ) -> DebtFireImpactReport {
        // Calculate the increase in the FIRE number required just to service the minimums indefinitely.
        // We temporarily increase the fire sim's monthly expenses.
        let mut debt_minimums: i64 = 0;
        for debt in &debts {
            debt_minimums = debt_minimums.saturating_add(debt.min_payment_cents);
        }



        let swr = self.fire_sim.config().safe_withdrawal_rate_pct;
        let fire_target_increase_cents = if swr == 0 {
            0
        } else {
            let yearly_debt_expenses = debt_minimums.saturating_mul(12);
            yearly_debt_expenses
                .saturating_mul(100)
                .saturating_div(i64::from(swr))
        };

        // Run the debt optimizer
        let mut optimizer = DebtOptimizer::new(monthly_payment_cents);
        for debt in debts {
            optimizer.add_debt(debt);
        }

        let payoff_result = optimizer.simulate(strategy);

        // Calculate simple opportunity cost: the total amount paid (principal + interest)
        // could have been invested over `months_to_payoff` at `market_growth_rate_pct`.
        // We'll do a simple future value of an annuity calculation for the monthly payments.

        // R = monthly payment
        // r = monthly interest rate
        // n = number of periods (months)
        // FV = R * [ ((1 + r)^n - 1) / r ]

        let monthly_rate = (f64::from(self.market_growth_rate_pct) / 100.0) / 12.0;
        let n = f64::from(payoff_result.total_months);
        let r_payment = monthly_payment_cents as f64;

        let opportunity_cost_fv = if monthly_rate > 0.0 {
            r_payment * (((1.0 + monthly_rate).powf(n) - 1.0) / monthly_rate)
        } else {
            r_payment * n
        };

        DebtFireImpactReport {
            months_to_payoff: payoff_result.total_months,
            total_interest_paid_cents: payoff_result.total_interest_paid_cents,
            opportunity_cost_cents: opportunity_cost_fv.round() as i64,
            fire_target_increase_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fire::FireConfig;

    #[test]
    fn test_debt_fire_impact() {
        let mut sim = FireSimulator::new(50_0000); // 5k base expenses
        sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });

        let analyzer = DebtFireImpactAnalyzer::new(sim, 7); // 7% market growth

        let debt = Debt {
            name: "Car Loan".to_string(),
            balance_cents: 3_000_000, // 30k
            interest_rate_pct: 5,
            min_payment_cents: 50_000, // $500
        };

        // We pay $1000/mo
        let report = analyzer.evaluate(vec![debt], 100_000, PayoffStrategy::Avalanche);

        // It should take ~33 months to pay off 30k at 5% with $1000/mo
        assert!(report.months_to_payoff > 30 && report.months_to_payoff < 35);

        // The fire target increase should be: $500 * 12 * 25 = $150,000
        assert_eq!(report.fire_target_increase_cents, 15_000_000);

        // Total invested opportunity cost > (33 * 1000)
        assert!(report.opportunity_cost_cents > 3_300_000);
    }
}
