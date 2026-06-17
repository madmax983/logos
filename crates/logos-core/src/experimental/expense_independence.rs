#![cfg(feature = "nova")]

//! Expense Independence Analyzer Module
//!
//! Re-frames FIRE (Financial Independence, Retire Early) by breaking down
//! large portfolio milestones into micro-independence goals. It calculates
//! the required invested capital to perpetually fund specific recurring expenses
//! based on a Safe Withdrawal Rate (SWR).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpenseTarget {
    pub name: String,
    pub monthly_cost_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndependenceMilestone {
    pub expense_name: String,
    pub required_capital_cents: i64,
    pub is_achieved: bool,
}

#[derive(Debug, Clone)]
pub struct ExpenseIndependenceAnalyzer {
    safe_withdrawal_rate_pct: u8,
    invested_capital_cents: i64,
    expenses: Vec<ExpenseTarget>,
}

impl ExpenseIndependenceAnalyzer {
    #[must_use]
    pub const fn new(safe_withdrawal_rate_pct: u8, invested_capital_cents: i64) -> Self {
        Self {
            safe_withdrawal_rate_pct,
            invested_capital_cents,
            expenses: Vec::new(),
        }
    }

    pub fn add_expense(&mut self, name: &str, monthly_cost_cents: i64) {
        self.expenses.push(ExpenseTarget {
            name: name.to_string(),
            monthly_cost_cents,
        });
    }

    #[must_use]
    pub fn calculate_milestones(&self) -> Vec<IndependenceMilestone> {
        let mut milestones = Vec::with_capacity(self.expenses.len());
        let mut capital_allocated: i64 = 0;

        for expense in &self.expenses {
            // (Monthly * 12) / SWR_pct * 100
            let yearly_cost = expense.monthly_cost_cents.saturating_mul(12);
            let required_capital = if self.safe_withdrawal_rate_pct > 0 {
                yearly_cost
                    .saturating_mul(100)
                    .saturating_div(i64::from(self.safe_withdrawal_rate_pct))
            } else {
                i64::MAX
            };

            capital_allocated = capital_allocated.saturating_add(required_capital);
            let is_achieved = self.invested_capital_cents >= capital_allocated;

            milestones.push(IndependenceMilestone {
                expense_name: expense.name.clone(),
                required_capital_cents: required_capital,
                is_achieved,
            });
        }

        milestones
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expense_independence() {
        let mut analyzer = ExpenseIndependenceAnalyzer::new(4, 5_000_000); // 4% SWR, $50,000 invested

        analyzer.add_expense("Netflix", 1_500);
        analyzer.add_expense("Groceries", 60_000);

        let milestones = analyzer.calculate_milestones();
        assert_eq!(milestones.len(), 2);

        // Netflix: $15/mo * 12 = $180/yr. $180 / 0.04 = $4500 required capital.
        assert_eq!(milestones[0].expense_name, "Netflix");
        assert_eq!(milestones[0].required_capital_cents, 450_000);
        assert!(milestones[0].is_achieved); // 50k > 4.5k

        // Groceries: $600/mo * 12 = $7200/yr. $7200 / 0.04 = $180,000 required.
        // Cumulative required = 184,500. 50k < 184.5k.
        assert_eq!(milestones[1].expense_name, "Groceries");
        assert_eq!(milestones[1].required_capital_cents, 18_000_000);
        assert!(!milestones[1].is_achieved);
    }
}
