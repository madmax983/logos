#[cfg(feature = "nova")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtItem {
    pub name: String,
    pub balance_cents: i64,
    pub interest_rate_bps: u16, // Basis points, e.g., 500 = 5.00%
    pub min_payment_cents: i64,
}

#[cfg(feature = "nova")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayoffStrategy {
    /// Pay off lowest balances first
    Snowball,
    /// Pay off highest interest rates first
    Avalanche,
}

#[cfg(feature = "nova")]
#[derive(Debug, Clone)]
pub struct PayoffProjector {
    debts: Vec<DebtItem>,
    monthly_budget_cents: i64,
    strategy: PayoffStrategy,
}

#[cfg(feature = "nova")]
impl PayoffProjector {
    #[must_use]
    pub const fn new(monthly_budget_cents: i64, strategy: PayoffStrategy) -> Self {
        Self {
            debts: Vec::new(),
            monthly_budget_cents,
            strategy,
        }
    }

    pub fn add_debt(&mut self, debt: DebtItem) {
        self.debts.push(debt);
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn project_months_to_zero(&self) -> u16 {
        if self.debts.is_empty() {
            return 0;
        }

        let mut current_debts = self.debts.clone();
        let mut months = 0;

        while !current_debts.is_empty() && months < 1000 {
            months += 1;

            // Apply interest first
            for debt in &mut current_debts {
                let monthly_interest = (i128::from(debt.balance_cents) * i128::from(debt.interest_rate_bps)) / (12 * 10000);
                debt.balance_cents = debt.balance_cents.saturating_add(monthly_interest as i64);
            }

            let mut remaining_budget = self.monthly_budget_cents;

            // Sort according to strategy for extra payments
            current_debts.sort_by(|a, b| {
                match self.strategy {
                    PayoffStrategy::Snowball => a.balance_cents.cmp(&b.balance_cents),
                    PayoffStrategy::Avalanche => b.interest_rate_bps.cmp(&a.interest_rate_bps),
                }
            });

            // 1. Pay minimums
            for debt in &mut current_debts {
                let payment = debt.balance_cents.min(debt.min_payment_cents);
                let actual_payment = remaining_budget.min(payment);

                debt.balance_cents -= actual_payment;
                remaining_budget -= actual_payment;
            }

            // 2. Put remainder towards target
            for debt in &mut current_debts {
                if remaining_budget == 0 {
                    break;
                }

                let payment = debt.balance_cents.min(remaining_budget);
                debt.balance_cents -= payment;
                remaining_budget -= payment;
            }

            current_debts.retain(|d| d.balance_cents > 0);
        }

        months
    }
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;

    #[test]
    fn test_snowball_vs_avalanche() {
        let mut snowball = PayoffProjector::new(100_000, PayoffStrategy::Snowball); // $1000/mo
        let mut avalanche = PayoffProjector::new(100_000, PayoffStrategy::Avalanche);

        let d1 = DebtItem {
            name: "Credit Card".to_string(),
            balance_cents: 500_000, // $5000
            interest_rate_bps: 2000, // 20%
            min_payment_cents: 10_000, // $100
        };

        let d2 = DebtItem {
            name: "Car Loan".to_string(),
            balance_cents: 200_000, // $2000
            interest_rate_bps: 500, // 5%
            min_payment_cents: 5_000, // $50
        };

        snowball.add_debt(d1.clone());
        snowball.add_debt(d2.clone());

        avalanche.add_debt(d1);
        avalanche.add_debt(d2);

        // Avalanche should finish sooner/equal since it attacks high interest first
        let snow_months = snowball.project_months_to_zero();
        let ava_months = avalanche.project_months_to_zero();

        assert!(ava_months <= snow_months);
        assert!(ava_months > 0);
    }
}
