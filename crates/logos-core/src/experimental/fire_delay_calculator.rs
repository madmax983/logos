//! FIRE Delay Calculator
//!
//! A simulator that translates financial decisions into "months of your life".
//! It calculates the time to reach Financial Independence (FI) and evaluates
//! how a proposed purchase or lifestyle upgrade delays that FI date due to
//! lost compound interest and increased burn rate.

/// Represents a proposed financial decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposedChange {
    /// A one-time purchase (e.g., buying a car in cash) in cents.
    OneTimeExpense(i64),
    /// A recurring monthly expense increase (e.g., upgrading an apartment) in cents.
    MonthlyExpenseIncrease(i64),
}

/// The result of calculating the delay to Financial Independence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireDelayResult {
    /// The target FIRE number in cents for the baseline scenario.
    pub baseline_fire_number_cents: i64,
    /// Months to FI in the baseline scenario. `None` if FI is never reached.
    pub baseline_months_to_fi: Option<u32>,
    /// The target FIRE number in cents for the proposed scenario.
    pub proposed_fire_number_cents: i64,
    /// Months to FI in the proposed scenario. `None` if FI is never reached.
    pub proposed_months_to_fi: Option<u32>,
    /// The exact delay in months. `None` if FI becomes unreachable.
    pub delay_months: Option<u32>,
}

/// A calculator to measure the impact of financial decisions on the FIRE timeline.
#[derive(Debug, Clone)]
pub struct FireDelayCalculator {
    safe_net_worth_cents: i64,
    monthly_expenses_cents: i64,
    monthly_contribution_cents: i64,
    annual_return_pct: f64,
    safe_withdrawal_rate_pct: u8,
}

impl FireDelayCalculator {
    /// Creates a new `FireDelayCalculator`.
    ///
    /// # Arguments
    /// * `safe_net_worth_cents` - The current base net worth in cents.
    /// * `monthly_expenses_cents` - The baseline monthly burn rate in cents.
    /// * `monthly_contribution_cents` - The amount saved and invested every month in cents.
    /// * `annual_return_pct` - Expected real annual return (e.g., 7.0 for 7%).
    /// * `safe_withdrawal_rate_pct` - The target safe withdrawal rate (e.g., 4 for 4%).
    #[must_use]
    pub const fn new(
        safe_net_worth_cents: i64,
        monthly_expenses_cents: i64,
        monthly_contribution_cents: i64,
        annual_return_pct: f64,
        safe_withdrawal_rate_pct: u8,
    ) -> Self {
        Self {
            safe_net_worth_cents,
            monthly_expenses_cents,
            monthly_contribution_cents,
            annual_return_pct,
            safe_withdrawal_rate_pct,
        }
    }

    /// Iteratively calculates the number of months required to reach the target FI number.
    /// Returns `None` if it requires an unreasonable amount of time (e.g., > 100 years).
    fn calculate_months_to_fi(
        current_nw_cents: i64,
        monthly_contribution_cents: i64,
        target_cents: i64,
        annual_return_pct: f64,
    ) -> Option<u32> {
        if current_nw_cents >= target_cents {
            return Some(0);
        }

        let monthly_return_rate = annual_return_pct / 100.0 / 12.0;
        let mut months = 0;
        #[allow(clippy::cast_precision_loss)]
        let mut current_f64 = current_nw_cents as f64;
        #[allow(clippy::cast_precision_loss)]
        let target_f64 = target_cents as f64;
        #[allow(clippy::cast_precision_loss)]
        let contrib_f64 = monthly_contribution_cents as f64;

        // Arbitrary cap at 1200 months (100 years) to prevent infinite loops
        while months < 1200 {
            let gain = current_f64 * monthly_return_rate;
            current_f64 += gain + contrib_f64;
            months += 1;

            if current_f64 >= target_f64 {
                return Some(months);
            }
        }

        None
    }

    /// Evaluates the delay caused by a proposed financial change.
    #[must_use]
    pub fn evaluate(&self, change: ProposedChange) -> FireDelayResult {
        let baseline_fire_num = self.calculate_fire_number(self.monthly_expenses_cents);
        let baseline_months = Self::calculate_months_to_fi(
            self.safe_net_worth_cents,
            self.monthly_contribution_cents,
            baseline_fire_num,
            self.annual_return_pct,
        );

        let (proposed_nw, proposed_expenses, proposed_contribution) = match change {
            ProposedChange::OneTimeExpense(cost) => {
                let nw = self.safe_net_worth_cents.saturating_sub(cost);
                (
                    nw,
                    self.monthly_expenses_cents,
                    self.monthly_contribution_cents,
                )
            }
            ProposedChange::MonthlyExpenseIncrease(increase) => {
                // If expenses go up, assuming fixed income, savings go down by the same amount.
                let exp = self.monthly_expenses_cents.saturating_add(increase);
                let contrib = self
                    .monthly_contribution_cents
                    .saturating_sub(increase)
                    .max(0);
                (self.safe_net_worth_cents, exp, contrib)
            }
        };

        let proposed_fire_num = self.calculate_fire_number(proposed_expenses);
        let proposed_months = Self::calculate_months_to_fi(
            proposed_nw,
            proposed_contribution,
            proposed_fire_num,
            self.annual_return_pct,
        );

        let delay_months = match (baseline_months, proposed_months) {
            (Some(b), Some(p)) => {
                if p >= b {
                    Some(p - b)
                } else {
                    Some(0) // Should not happen with valid inputs, but handled safely.
                }
            }
            _ => None,
        };

        FireDelayResult {
            baseline_fire_number_cents: baseline_fire_num,
            baseline_months_to_fi: baseline_months,
            proposed_fire_number_cents: proposed_fire_num,
            proposed_months_to_fi: proposed_months,
            delay_months,
        }
    }

    fn calculate_fire_number(&self, monthly_expenses_cents: i64) -> i64 {
        if self.safe_withdrawal_rate_pct == 0 {
            return i64::MAX;
        }
        let yearly_expenses = monthly_expenses_cents.saturating_mul(12);
        yearly_expenses.saturating_mul(100) / i64::from(self.safe_withdrawal_rate_pct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_delay_one_time_expense() {
        // Baseline:
        // NW: $100k
        // Expenses: $4k/mo -> target $1.2M
        // Contribution: $2k/mo
        // Return: 7%
        let calc = FireDelayCalculator::new(
            10_000_000, // 100k
            400_000,    // 4k
            200_000,    // 2k
            7.0, 4,
        );

        // Buy a $50k car in cash
        let result = calc.evaluate(ProposedChange::OneTimeExpense(5_000_000));

        assert_eq!(result.baseline_fire_number_cents, 120_000_000);
        assert_eq!(result.proposed_fire_number_cents, 120_000_000);

        let baseline = result.baseline_months_to_fi.unwrap();
        let proposed = result.proposed_months_to_fi.unwrap();

        // Should take more months to reach FI
        assert!(proposed > baseline);
        assert_eq!(result.delay_months.unwrap(), proposed - baseline);
    }

    #[test]
    fn test_fire_delay_monthly_expense_increase() {
        // Baseline:
        // NW: $500k
        // Expenses: $5k/mo -> target $1.5M
        // Contribution: $3k/mo
        // Return: 7%
        let calc = FireDelayCalculator::new(
            50_000_000, // 500k
            500_000,    // 5k
            300_000,    // 3k
            7.0, 4,
        );

        // Upgrade apartment, increasing rent by $1k/mo
        let result = calc.evaluate(ProposedChange::MonthlyExpenseIncrease(100_000));

        // Baseline FIRE target is 1.5M
        assert_eq!(result.baseline_fire_number_cents, 150_000_000);
        // Proposed FIRE target is 1.8M (6k * 12 * 25)
        assert_eq!(result.proposed_fire_number_cents, 180_000_000);

        let baseline = result.baseline_months_to_fi.unwrap();
        let proposed = result.proposed_months_to_fi.unwrap();

        // Significant delay due to higher target and lower contribution
        assert!(proposed > baseline);
        assert_eq!(result.delay_months.unwrap(), proposed - baseline);
    }

    #[test]
    fn test_already_fi() {
        let calc = FireDelayCalculator::new(
            200_000_000, // 2M
            400_000,     // 4k/mo -> target 1.2M
            0,
            7.0,
            4,
        );

        let result = calc.evaluate(ProposedChange::OneTimeExpense(1_000_000)); // Spend $10k

        assert_eq!(result.baseline_months_to_fi, Some(0));
        assert_eq!(result.proposed_months_to_fi, Some(0));
        assert_eq!(result.delay_months, Some(0));
    }

    #[test]
    fn test_unreachable_fi() {
        // 0 return, 0 contribution, target > NW
        let calc = FireDelayCalculator::new(10_000_000, 400_000, 0, 0.0, 4);

        let result = calc.evaluate(ProposedChange::OneTimeExpense(1_000_000));

        assert_eq!(result.baseline_months_to_fi, None);
        assert_eq!(result.proposed_months_to_fi, None);
        assert_eq!(result.delay_months, None);
    }
}
