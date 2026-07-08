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
        let mut total_months: u32 = 0;
        let mut total_interest_paid_cents: i64 = 0;

        while current_debts.iter().any(|d| d.balance_cents > 0) {
            total_months = total_months.saturating_add(1);

            // Safety to prevent infinite loops (e.g., if monthly payment cannot cover interest)
            if total_months > 1200 {
                break;
            }

            total_interest_paid_cents =
                total_interest_paid_cents.saturating_add(Self::apply_interest(&mut current_debts));

            let mut remaining_cash = self.monthly_payment_cents;
            remaining_cash = Self::pay_minimums(&mut current_debts, remaining_cash);

            if remaining_cash <= 0 {
                continue;
            }

            Self::allocate_extra_payments(&mut current_debts, remaining_cash, strategy);
        }

        PayoffResult {
            total_months,
            total_interest_paid_cents,
        }
    }

    fn apply_interest(debts: &mut [Debt]) -> i64 {
        let mut interest_this_month: i64 = 0;
        for debt in debts {
            if debt.balance_cents > 0 {
                // Simple monthly interest: (balance * rate / 100) / 12
                let rate = i64::from(debt.interest_rate_pct);
                let monthly_interest = debt
                    .balance_cents
                    .saturating_mul(rate)
                    .saturating_div(100)
                    .saturating_div(12);
                interest_this_month = interest_this_month.saturating_add(monthly_interest);
                debt.balance_cents = debt.balance_cents.saturating_add(monthly_interest);
            }
        }
        interest_this_month
    }

    fn pay_minimums(debts: &mut [Debt], mut remaining_cash: i64) -> i64 {
        for debt in debts {
            if debt.balance_cents > 0 {
                let payment = std::cmp::min(debt.balance_cents, debt.min_payment_cents);
                let actual_payment = std::cmp::min(payment, remaining_cash);

                debt.balance_cents = debt.balance_cents.saturating_sub(actual_payment);
                remaining_cash = remaining_cash.saturating_sub(actual_payment);
            }
        }
        remaining_cash
    }

    fn allocate_extra_payments(
        debts: &mut [Debt],
        mut remaining_cash: i64,
        strategy: PayoffStrategy,
    ) {
        match strategy {
            PayoffStrategy::Snowball => {
                // Smallest balance first
                debts.sort_by_key(|d| d.balance_cents);
            }
            PayoffStrategy::Avalanche => {
                // Highest interest rate first (reverse sort)
                debts.sort_by(|a, b| b.interest_rate_pct.cmp(&a.interest_rate_pct));
            }
        }

        for debt in debts {
            if debt.balance_cents > 0 && remaining_cash > 0 {
                let extra_payment = std::cmp::min(debt.balance_cents, remaining_cash);
                debt.balance_cents = debt.balance_cents.saturating_sub(extra_payment);
                remaining_cash = remaining_cash.saturating_sub(extra_payment);
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
