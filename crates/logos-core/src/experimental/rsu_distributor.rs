//! Automated distribution of Restricted Stock Unit (RSU) vests.
//!
//! This module provides an automated way to allocate the gross proceeds
//! of an RSU vest across multiple target accounts (like tax reserves, savings
//! buffers, and checking accounts) according to an [`AllocationPolicy`].
//!
//! It ensures that the generated distribution is recorded as a perfectly
//! balanced double-entry transaction, handling rounding remainders gracefully.

use crate::domain::rsu::AllocationPolicy;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;

/// Configuration for `RsuAutoDistributor`, preventing positional string arguments.
///
/// Defines the specific account names that will be credited/debited when
/// an RSU vest is distributed.
///
/// ## Examples
///
/// ```
/// use logos_core::experimental::rsu_distributor::RsuDistributorConfig;
///
/// let config = RsuDistributorConfig {
///     rsu_asset: "assets:brokerage:rsu".to_owned(),
///     tax_reserve: "liabilities:tax_reserve".to_owned(),
///     smoothing_buffer: "assets:checking:buffer".to_owned(),
///     goals: "assets:savings:goals".to_owned(),
///     discretionary: "assets:checking:spending".to_owned(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RsuDistributorConfig {
    pub rsu_asset: String,
    pub tax_reserve: String,
    pub smoothing_buffer: String,
    pub goals: String,
    pub discretionary: String,
}

/// Automatically distributes vested RSU funds across target accounts
/// according to an [`AllocationPolicy`].
///
/// This struct acts as a factory for creating distribution [`Transaction`]s.
/// It takes a configuration specifying the target accounts and uses the policy
/// to divide the gross vest amount into perfectly balanced postings.
///
/// ## Examples
///
/// ```
/// use logos_core::domain::rsu::AllocationPolicy;
/// use logos_core::experimental::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
///
/// let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();
/// let config = RsuDistributorConfig {
///     rsu_asset: "assets:rsu".to_owned(),
///     tax_reserve: "assets:tax".to_owned(),
///     smoothing_buffer: "assets:buffer".to_owned(),
///     goals: "assets:goals".to_owned(),
///     discretionary: "assets:checking".to_owned(),
/// };
///
/// let distributor = RsuAutoDistributor::new(config);
/// let transaction = distributor
///     .distribute_rsu_vest("Vest 2024-03", 100_000, &policy)
///     .expect("Creates a balanced transaction");
///
/// assert_eq!(transaction.postings().len(), 5);
/// ```
#[derive(Debug, Clone)]
pub struct RsuAutoDistributor {
    config: RsuDistributorConfig,
}

impl RsuAutoDistributor {
    /// Creates a new distributor with the specified account configuration.
    #[must_use]
    pub const fn new(config: RsuDistributorConfig) -> Self {
        Self { config }
    }

    /// Distributes a gross vest amount across the configured accounts.
    ///
    /// This method ensures the resulting transaction is perfectly balanced.
    /// Any remainder cents left over from percentage-based division are
    /// automatically swept into the tax reserve account to prevent the
    /// transaction from being rejected due to an imbalance.
    ///
    /// # Errors
    /// Returns an error if the transaction description is empty or if the resulting transaction is unbalanced.
    pub fn distribute_rsu_vest(
        &self,
        description: &str,
        gross_vest_cents: i64,
        policy: &AllocationPolicy,
    ) -> Result<Transaction, DomainError> {
        let smoothing_cents = (gross_vest_cents * i64::from(policy.smoothing_buffer_pct())) / 100;
        let goals_cents = (gross_vest_cents * i64::from(policy.goals_pct())) / 100;
        let discretionary_cents = (gross_vest_cents * i64::from(policy.discretionary_pct())) / 100;

        // The remaining amount goes to the tax reserve to ensure perfectly balanced transaction
        let tax_cents = gross_vest_cents - smoothing_cents - goals_cents - discretionary_cents;

        TransactionBuilder::new(description)
            .posting(Posting::credit(&self.config.rsu_asset, gross_vest_cents))
            .posting(Posting::debit(&self.config.tax_reserve, tax_cents))
            .posting(Posting::debit(
                &self.config.smoothing_buffer,
                smoothing_cents,
            ))
            .posting(Posting::debit(&self.config.goals, goals_cents))
            .posting(Posting::debit(
                &self.config.discretionary,
                discretionary_cents,
            ))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_distribution() {
        let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();
        let config = RsuDistributorConfig {
            rsu_asset: "assets:rsu".to_owned(),
            tax_reserve: "assets:tax".to_owned(),
            smoothing_buffer: "assets:buffer".to_owned(),
            goals: "assets:goals".to_owned(),
            discretionary: "assets:checking".to_owned(),
        };
        let distributor = RsuAutoDistributor::new(config);

        let tx = distributor
            .distribute_rsu_vest("Vest 1", 10000, &policy)
            .expect("should build perfectly");

        let postings = tx.postings();
        assert_eq!(postings.len(), 5);

        // the source account is credited the gross amount
        assert!(postings.contains(&Posting::credit("assets:rsu", 10000)));

        // the destinations are debited
        assert!(postings.contains(&Posting::debit("assets:tax", 4000)));
        assert!(postings.contains(&Posting::debit("assets:buffer", 2000)));
        assert!(postings.contains(&Posting::debit("assets:goals", 3000)));
        assert!(postings.contains(&Posting::debit("assets:checking", 1000)));
    }

    #[test]
    fn test_imperfect_distribution_sweeps_to_tax() {
        let policy = AllocationPolicy::new(33, 33, 33, 1).unwrap();
        let config = RsuDistributorConfig {
            rsu_asset: "assets:rsu".to_owned(),
            tax_reserve: "assets:tax".to_owned(),
            smoothing_buffer: "assets:buffer".to_owned(),
            goals: "assets:goals".to_owned(),
            discretionary: "assets:checking".to_owned(),
        };
        let distributor = RsuAutoDistributor::new(config);

        // 100 cents * 33% = 33 cents each, 1 cent for discretionary = 100 cents total.
        // Wait, let's pick a number that leaves remainders.
        // 10 cents * 33% = 3 cents. 3 * 3 + 0 = 9. Remainder = 1.
        let tx = distributor
            .distribute_rsu_vest("Vest 2", 10, &policy)
            .expect("should sweep remainder to tax reserve");

        let postings = tx.postings();

        // 3 cents buffer, 3 cents goals, 0 cents discretionary (10 * 1% = 0). Total so far = 6 cents.
        // Remainder = 10 - 6 = 4 cents.
        // So tax gets 4 cents.

        assert!(postings.contains(&Posting::credit("assets:rsu", 10)));
        assert!(postings.contains(&Posting::debit("assets:tax", 4)));
        assert!(postings.contains(&Posting::debit("assets:buffer", 3)));
        assert!(postings.contains(&Posting::debit("assets:goals", 3)));
        assert!(postings.contains(&Posting::debit("assets:checking", 0)));
    }
}
