//! Debt Avalanche/Snowball Optimizer
//!
//! Provides a simulator to calculate the optimal debt payoff timeline using either
//! the Avalanche method (highest interest first) or the Snowball method (lowest balance first).

/// A single debt obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Debt {
    /// Name of the debt (e.g., "Student Loan", "Credit Card")
    pub name: String,
    /// The remaining principal balance in cents.
    pub principal_cents: i64,
    /// The annual interest rate in basis points (1 bps = 0.01%).
    /// Example: 5.5% = 550 bps.
    pub interest_rate_bps: u32,
    /// The required minimum monthly payment in cents.
    pub min_payment_cents: i64,
}

/// The strategy used to prioritize extra debt payments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayoffStrategy {
    /// Prioritize the debt with the highest interest rate (mathematically optimal).
    Avalanche,
    /// Prioritize the debt with the lowest remaining balance (psychological wins).
    Snowball,
}

/// Represents the final result of a debt payoff simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoffResult {
    /// The total number of months required to pay off all debts.
    pub months_to_payoff: u16,
    /// The total amount of interest paid across all debts during the payoff period, in cents.
    pub total_interest_paid_cents: i64,
}

/// A simulator to calculate debt payoff timelines.
#[derive(Debug, Clone)]
pub struct DebtPayoffOptimizer {
    debts: Vec<Debt>,
    strategy: PayoffStrategy,
    monthly_allocation_cents: i64,
}

impl DebtPayoffOptimizer {
    /// Creates a new `DebtPayoffOptimizer`.
    #[must_use]
    pub const fn new(debts: Vec<Debt>, strategy: PayoffStrategy, monthly_allocation_cents: i64) -> Self {
        Self {
            debts,
            strategy,
            monthly_allocation_cents,
        }
    }

    /// Simulates the debt payoff process.
    /// Returns the total months to payoff and the total interest paid in cents.
    /// If the monthly allocation is less than the sum of all minimum payments,
    /// or if it's impossible to pay off the debt (e.g. interest exceeds payments),
    /// it may run infinitely. We cap the simulation at 1200 months (100 years).
    #[must_use]
    pub fn simulate(&self) -> PayoffResult {
        let mut active_debts = self.debts.clone();
        let mut months = 0;
        let mut total_interest_paid = 0;

        while !active_debts.is_empty() && months < 1200 {
            months += 1;

            // 1. Sort debts according to the selected strategy.
            // This order is used to allocate any "extra" funds after minimums are met.
            active_debts.sort_by(|a, b| match self.strategy {
                PayoffStrategy::Avalanche => {
                    // Highest interest rate first, then lowest balance
                    b.interest_rate_bps
                        .cmp(&a.interest_rate_bps)
                        .then(a.principal_cents.cmp(&b.principal_cents))
                }
                PayoffStrategy::Snowball => {
                    // Lowest balance first, then highest interest rate
                    a.principal_cents
                        .cmp(&b.principal_cents)
                        .then(b.interest_rate_bps.cmp(&a.interest_rate_bps))
                }
            });

            // 2. Apply monthly interest to all active debts
            for debt in &mut active_debts {
                // Monthly interest rate is annual rate / 12.
                // bps = 1/10000. So monthly rate = bps / 120000.
                // interest = principal * bps / 120000.
                let interest_cents =
                    (debt.principal_cents * i64::from(debt.interest_rate_bps)) / 120_000;
                debt.principal_cents += interest_cents;
                total_interest_paid += interest_cents;
            }

            // 3. Make minimum payments
            let mut remaining_allocation = self.monthly_allocation_cents;
            for debt in &mut active_debts {
                let payment = std::cmp::min(debt.principal_cents, debt.min_payment_cents);
                let payment = std::cmp::min(payment, remaining_allocation);
                debt.principal_cents -= payment;
                remaining_allocation -= payment;
            }

            // 4. Allocate remaining funds to the highest priority debt
            for debt in &mut active_debts {
                if remaining_allocation <= 0 {
                    break;
                }
                if debt.principal_cents > 0 {
                    let payment = std::cmp::min(debt.principal_cents, remaining_allocation);
                    debt.principal_cents -= payment;
                    remaining_allocation -= payment;
                }
            }

            // 5. Remove paid off debts
            active_debts.retain(|d| d.principal_cents > 0);
        }

        PayoffResult {
            months_to_payoff: months,
            total_interest_paid_cents: total_interest_paid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_avalanche_vs_snowball() {
        let debts = vec![
            Debt {
                name: "Small Low Interest".to_string(),
                principal_cents: 100_000, // $1,000
                interest_rate_bps: 100,  // 1%
                min_payment_cents: 1_000, // $10
            },
            Debt {
                name: "Large High Interest".to_string(),
                principal_cents: 5_000_000, // $50,000
                interest_rate_bps: 2000,  // 20%
                min_payment_cents: 50_000, // $500
            },
        ];

        // Ensure that there's enough allocation to make a difference between strategies.
        // Avalanche mathematically guarantees the lowest or equal total interest paid.
        let avalanche = DebtPayoffOptimizer::new(debts.clone(), PayoffStrategy::Avalanche, 100_000); // $1000 monthly
        let avalanche_result = avalanche.simulate();

        let snowball = DebtPayoffOptimizer::new(debts, PayoffStrategy::Snowball, 100_000);
        let snowball_result = snowball.simulate();

        // Avalanche should result in strictly less total interest paid
        assert!(
            avalanche_result.total_interest_paid_cents < snowball_result.total_interest_paid_cents
        );

        assert!(avalanche_result.months_to_payoff < 200);
        assert!(snowball_result.months_to_payoff < 200);
    }

    #[test]
    fn test_simulate_insufficient_allocation() {
        let debts = vec![Debt {
            name: "Massive Debt".to_string(),
            principal_cents: 10_000_000, // $100,000
            interest_rate_bps: 2000,     // 20%
            min_payment_cents: 1_000,    // $10
        }];

        // $1 monthly allocation (can't even cover interest)
        let optimizer = DebtPayoffOptimizer::new(debts, PayoffStrategy::Avalanche, 100);
        let result = optimizer.simulate();

        // Should hit the 1200 month cap
        assert_eq!(result.months_to_payoff, 1200);
    }
}
