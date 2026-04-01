use crate::error::RuntimeError;
use crate::models::MonthReconciliation;
use crate::runtime::{AppRuntime, transaction_in_month};
use logos_core::TransactionId;
use logos_store_aletheia::model::{StoredMonthClose, StoredReconciliationRun, StoredStatementLine};

impl AppRuntime {
    #[must_use]
    pub fn reconcile_month_for(
        &self,
        checking_account: &str,
        month_key: &str,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    ) -> MonthReconciliation {
        let mut ledger_delta_cents = 0_i64;
        let mut inflow_cents = 0_i64;
        let mut outflow_cents = 0_i64;
        let mut matched_postings = 0_usize;

        for posting in self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().as_str() == checking_account)
        {
            let amount = posting.amount();
            matched_postings = matched_postings.saturating_add(1);
            ledger_delta_cents = ledger_delta_cents.saturating_add(amount);
            if amount >= 0 {
                inflow_cents = inflow_cents.saturating_add(amount);
            } else {
                outflow_cents =
                    outflow_cents.saturating_add(amount.checked_abs().unwrap_or(i64::MAX));
            }
        }

        let expected_closing_balance_cents =
            opening_balance_cents.saturating_add(ledger_delta_cents);
        let variance_cents = closing_balance_cents.saturating_sub(expected_closing_balance_cents);
        MonthReconciliation::new(
            ledger_delta_cents,
            expected_closing_balance_cents,
            closing_balance_cents,
            variance_cents,
            variance_cents == 0,
            matched_postings,
            inflow_cents,
            outflow_cents,
        )
    }

    /// Computes and persists an immutable reconciliation run for the provided month and account.
    ///
    /// # Errors
    ///
    /// Returns an error when reconciliation persistence fails.
    pub fn reconcile_and_persist_month_for(
        &mut self,
        checking_account: &str,
        month_key: &str,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    ) -> Result<StoredReconciliationRun, RuntimeError> {
        let report = self.reconcile_month_for(
            checking_account,
            month_key,
            opening_balance_cents,
            closing_balance_cents,
        );
        let reconciled_txn_ids =
            self.reconciliation_transaction_ids_for(checking_account, month_key);
        let matched_postings = i64::try_from(report.matched_postings()).unwrap_or(i64::MAX);
        self.store
            .write_reconciliation_run(
                month_key,
                checking_account,
                opening_balance_cents,
                report.ledger_delta_cents(),
                report.expected_closing_balance_cents(),
                closing_balance_cents,
                report.variance_cents(),
                report.is_reconciled(),
                matched_postings,
                report.inflow_cents(),
                report.outflow_cents(),
                &reconciled_txn_ids,
            )
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn reconciliation_run_count(&self) -> usize {
        self.store.reconciliation_run_count()
    }

    #[must_use]
    pub fn reconciliation_run(&self, run_id: &str) -> Option<StoredReconciliationRun> {
        self.store.reconciliation_run(run_id).cloned()
    }

    #[must_use]
    pub fn list_reconciliation_runs(
        &self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) -> Vec<StoredReconciliationRun> {
        let mut runs: Vec<_> = self
            .store
            .reconciliation_runs()
            .filter(|run| month_key.is_none_or(|month| run.month_key() == month))
            .filter(|run| checking_account.is_none_or(|account| run.checking_account() == account))
            .cloned()
            .collect();
        runs.sort_by(|left, right| {
            right
                .created_at()
                .wallclock()
                .cmp(&left.created_at().wallclock())
                .then_with(|| left.run_id().cmp(right.run_id()))
        });
        runs
    }

    /// Closes a month scope using a previously persisted reconciliation run.
    ///
    /// # Errors
    ///
    /// Returns an error when scope metadata is invalid or write persistence fails.
    pub fn close_month(
        &mut self,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
    ) -> Result<StoredMonthClose, RuntimeError> {
        self.store
            .write_month_close(
                month_key,
                checking_account,
                reconciliation_run_id,
                analytics_artifact_id,
            )
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Option<StoredMonthClose> {
        self.store
            .month_close_for_scope(month_key, checking_account)
            .cloned()
    }

    #[must_use]
    pub fn statement_lines_for_reconciliation_run(&self, run_id: &str) -> Vec<StoredStatementLine> {
        self.store
            .statement_lines_for_reconciliation_run(run_id)
            .into_iter()
            .cloned()
            .collect()
    }

    pub(crate) fn reconciliation_transaction_ids_for(
        &self,
        checking_account: &str,
        month_key: &str,
    ) -> Vec<TransactionId> {
        let mut ids: Vec<_> = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .filter(|stored| {
                stored
                    .transaction()
                    .postings()
                    .iter()
                    .any(|posting| posting.account().as_str() == checking_account)
            })
            .map(|stored| stored.id().clone())
            .collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        ids.dedup_by(|left, right| left.as_str() == right.as_str());
        ids
    }

}
