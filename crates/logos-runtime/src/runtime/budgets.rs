use crate::error::RuntimeError;
use crate::models::MonthReport;
use crate::runtime::{AppRuntime, transaction_in_month};

use logos_core::Posting;
use logos_reporting::{project_budget_variance, project_cashflow, project_rsu_budget_plan, RsuBudgetPlan, RsuBudgetPlanInput, ScenarioPriceInputs};


impl AppRuntime {
    pub fn budget_variance_for(&self, budget_cents: i64, expense_account_prefix: &str) -> i64 {
        let actual_expense_cents: i64 = self
            .store
            .transactions()
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| {
                posting
                    .account()
                    .as_str()
                    .starts_with(expense_account_prefix)
            })
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .fold(0_i64, i64::saturating_add);
        project_budget_variance(budget_cents, actual_expense_cents)
    }

    /// Persists a budget target for a month and account-prefix scope.
    ///
    /// # Errors
    ///
    /// Returns an error when persistence fails.
    pub fn set_budget_target_for_month(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), RuntimeError> {
        self.store
            .write_budget_target(month_key, expense_account_prefix, budget_cents)?;
        Ok(())
    }

    pub fn budget_target_for_month(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<i64> {
        self.store
            .budget_target(month_key, expense_account_prefix)
            .map(logos_store_aletheia::model::StoredBudgetTarget::budget_cents)
    }

    #[must_use]
    pub fn budget_variance_for_month(
        &self,
        month_key: &str,
        budget_cents: i64,
        expense_account_prefix: &str,
    ) -> i64 {
        let actual_expense_cents = self.expense_total_for_month(month_key, expense_account_prefix);
        project_budget_variance(budget_cents, actual_expense_cents)
    }

    #[must_use]
    pub fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport {
        let mut checking_balance_cents = 0_i64;
        let mut income_cents = 0_i64;
        let mut expense_cents = 0_i64;

        for stored in self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
        {
            for posting in stored.transaction().postings() {
                let account = posting.account().as_str();
                let amount = posting.amount();

                if account == checking_account {
                    checking_balance_cents = checking_balance_cents.saturating_add(amount);
                }

                if account.starts_with("income:") && amount < 0 {
                    income_cents =
                        income_cents.saturating_add(amount.checked_abs().unwrap_or(i64::MAX));
                } else if account.starts_with("expenses:") && amount > 0 {
                    expense_cents = expense_cents.saturating_add(amount);
                }
            }
        }

        let cashflow_cents = project_cashflow(income_cents, expense_cents);
        MonthReport::new(
            checking_balance_cents,
            income_cents,
            expense_cents,
            cashflow_cents,
        )
    }


        #[allow(clippy::too_many_arguments, clippy::missing_errors_doc)]
    pub fn plan_rsu_budget_for_month(
        &self,
        month_key: &str,
        quarterly_units: u32,
        days_to_vest: u16,
        bear_price_cents: i64,
        base_price_cents: i64,
        bull_price_cents: i64,
        fixed_commitments_cents: i64,
        reserve_sweep_pct: u8,
        investing_sweep_pct: u8,
    ) -> Result<RsuBudgetPlan, RuntimeError> {
        let scenario_prices =
            ScenarioPriceInputs::new(bear_price_cents, base_price_cents, bull_price_cents)
                .map_err(|message| RuntimeError::Analytics { message })?;
        let input = RsuBudgetPlanInput::new(
            quarterly_units,
            days_to_vest,
            scenario_prices,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        )
        .map_err(|message| RuntimeError::Analytics { message })?;
        project_rsu_budget_plan(month_key, &input)
            .map_err(|message| RuntimeError::Analytics { message })
    }


    fn expense_total_for_month(&self, month_key: &str, expense_account_prefix: &str) -> i64 {
        self.store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| {
                posting
                    .account()
                    .as_str()
                    .starts_with(expense_account_prefix)
            })
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .fold(0_i64, i64::saturating_add)
    }

}
