//! Debt Payoff Optimizer
//!
//! A simulator to calculate the fastest way to pay off multiple debts.
//! Supports different payoff strategies such as Snowball (lowest balance first)
//! and Avalanche (highest interest rate first).

/// Strategy for ordering which debt to pay extra towards first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayoffStrategy {
    /// Pay off the debt with the smallest balance first.
    Snowball,
    /// Pay off the debt with the highest interest rate first.
    Avalanche,
}

/// Represents a single debt obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Debt {
    /// The name of the debt (e.g. "Student Loan", "Credit Card").
    pub name: String,
    /// The current remaining balance in cents.
    pub balance_cents: i64,
    /// The annual interest rate as a percentage (e.g. 15 for 15%).
    pub interest_rate_pct: u32,
    /// The minimum required monthly payment in cents.
    pub min_payment_cents: i64,
}

/// The outcome of simulating a debt payoff strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoffResult {
    /// Total number of months it took to pay off all debts.
    pub total_months: u32,
    /// Total amount of interest paid across all debts, in cents.
    pub total_interest_paid_cents: i64,
}

/// An optimizer that simulates paying off a list of debts.
#[derive(Debug, Clone)]
pub struct DebtOptimizer {
    debts: Vec<Debt>,
    monthly_payment_cents: i64,
}

impl DebtOptimizer {
    /// Creates a new `DebtOptimizer` with an initial total monthly payment budget.
    #[must_use]
    pub const fn new(monthly_payment_cents: i64) -> Self {
        Self {
            debts: Vec::new(),
            monthly_payment_cents,
        }
    }

    /// Adds a debt to the optimizer.
    pub fn add_debt(&mut self, debt: Debt) {
        self.debts.push(debt);
    }

    /// Simulates the payoff using the chosen strategy.
    #[must_use]
    pub fn simulate(&self, strategy: PayoffStrategy) -> PayoffResult {
        let mut current_debts = self.debts.clone();
        let mut total_months = 0;
        let mut total_interest_paid_cents = 0;

        while current_debts.iter().any(|d| d.balance_cents > 0) {
            total_months += 1;

            // Safety to prevent infinite loops (e.g., if monthly payment cannot cover interest)
            if total_months > 1200 {
                break;
            }

            Self::apply_interest_to_debts(&mut current_debts, &mut total_interest_paid_cents);

            let mut remaining_cash = self.monthly_payment_cents;
            Self::pay_minimums(&mut current_debts, &mut remaining_cash);

            if remaining_cash <= 0 {
                continue;
            }

            Self::sort_debts_by_strategy(&mut current_debts, strategy);
            Self::pay_extra_with_remaining_cash(&mut current_debts, &mut remaining_cash);
        }

        PayoffResult {
            total_months,
            total_interest_paid_cents,
        }
    }

    fn apply_interest_to_debts(current_debts: &mut [Debt], total_interest_paid_cents: &mut i64) {
        for debt in current_debts {
            if debt.balance_cents > 0 {
                // Simple monthly interest: (balance * rate / 100) / 12
                let monthly_interest =
                    (debt.balance_cents * i64::from(debt.interest_rate_pct)) / 100 / 12;
                *total_interest_paid_cents += monthly_interest;
                debt.balance_cents += monthly_interest;
            }
        }
    }

    fn pay_minimums(current_debts: &mut [Debt], remaining_cash: &mut i64) {
        for debt in current_debts {
            if debt.balance_cents > 0 {
                let payment = std::cmp::min(debt.balance_cents, debt.min_payment_cents);
                let actual_payment = std::cmp::min(payment, *remaining_cash);

                debt.balance_cents -= actual_payment;
                *remaining_cash -= actual_payment;
            }
        }
    }

    fn sort_debts_by_strategy(current_debts: &mut [Debt], strategy: PayoffStrategy) {
        match strategy {
            PayoffStrategy::Snowball => {
                // Smallest balance first
                current_debts.sort_by_key(|d| d.balance_cents);
            }
            PayoffStrategy::Avalanche => {
                // Highest interest rate first (reverse sort)
                current_debts.sort_by(|a, b| b.interest_rate_pct.cmp(&a.interest_rate_pct));
            }
        }
    }

    fn pay_extra_with_remaining_cash(current_debts: &mut [Debt], remaining_cash: &mut i64) {
        for debt in current_debts {
            if debt.balance_cents > 0 && *remaining_cash > 0 {
                let extra_payment = std::cmp::min(debt.balance_cents, *remaining_cash);
                debt.balance_cents -= extra_payment;
                *remaining_cash -= extra_payment;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowball_vs_avalanche() {
        let mut optimizer = DebtOptimizer::new(100_000); // $1,000/month budget

        // Debt 1: Smallest balance, lower interest
        optimizer.add_debt(Debt {
            name: "Credit Card".to_string(),
            balance_cents: 200_000, // $2,000
            interest_rate_pct: 10,
            min_payment_cents: 5_000, // $50
        });

        // Debt 2: Larger balance, higher interest
        optimizer.add_debt(Debt {
            name: "Student Loan".to_string(),
            balance_cents: 500_000, // $5,000
            interest_rate_pct: 25,
            min_payment_cents: 10_000, // $100
        });

        let snowball_result = optimizer.simulate(PayoffStrategy::Snowball);
        let avalanche_result = optimizer.simulate(PayoffStrategy::Avalanche);

        // Both should finish paying off
        assert!(snowball_result.total_months > 0);
        assert!(avalanche_result.total_months > 0);

        // Avalanche should result in less total interest paid
        assert!(
            avalanche_result.total_interest_paid_cents < snowball_result.total_interest_paid_cents
        );
    }
}
