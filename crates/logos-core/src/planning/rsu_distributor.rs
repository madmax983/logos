//! Automated distribution of restricted stock unit (RSU) vests.
//!
//! When an RSU vests, the gross value needs to be tracked and distributed across
//! several financial buckets (e.g., tax reserves, savings goals, discretionary spending).
//! This module automates the generation of perfectly balanced, multi-posting ledger
//! transactions according to a user's defined [`AllocationPolicy`].

use crate::domain::account::AccountId;
use crate::domain::rsu::AllocationPolicy;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;

/// Configuration mapping logical buckets to physical account IDs.
///
/// This struct prevents positional string argument mix-ups when initializing the
/// [`RsuAutoDistributor`].
///
/// ## Examples
///
/// ```
/// use logos_core::planning::rsu_distributor::RsuDistributorConfig;
/// use logos_core::AccountId;
///
/// let config = RsuDistributorConfig {
///     rsu_asset: AccountId::new("assets:rsu").unwrap(),
///     tax_reserve: AccountId::new("assets:tax").unwrap(),
///     smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
///     goals: AccountId::new("assets:goals").unwrap(),
///     discretionary: AccountId::new("assets:checking").unwrap(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RsuDistributorConfig {
    /// The account holding the gross RSU vest value.
    pub rsu_asset: AccountId,
    /// The account allocated for tax withholdings.
    pub tax_reserve: AccountId,
    /// The account allocated for income smoothing.
    pub smoothing_buffer: AccountId,
    /// The account allocated for specific financial goals.
    pub goals: AccountId,
    /// The account allocated for discretionary spending.
    pub discretionary: AccountId,
}

/// Automatically distributes vested RSU funds across target accounts
/// according to an [`AllocationPolicy`].
///
/// Creates a single, balanced [`Transaction`] representing the vest event.
#[derive(Debug, Clone)]
pub struct RsuAutoDistributor {
    config: RsuDistributorConfig,
}

impl RsuAutoDistributor {
    /// Creates a new distributor with the provided account routing configuration.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
    /// use logos_core::AccountId;
    ///
    /// let config = RsuDistributorConfig {
    ///     rsu_asset: AccountId::new("assets:rsu").unwrap(),
    ///     tax_reserve: AccountId::new("assets:tax").unwrap(),
    ///     smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
    ///     goals: AccountId::new("assets:goals").unwrap(),
    ///     discretionary: AccountId::new("assets:checking").unwrap(),
    /// };
    ///
    /// let distributor = RsuAutoDistributor::new(config);
    /// ```
    #[must_use]
    pub const fn new(config: RsuDistributorConfig) -> Self {
        Self { config }
    }

    /// Distributes a gross vest amount across the configured accounts.
    ///
    /// Because financial math often results in fractional cents when multiplying by
    /// percentages, this function guarantees that the resulting transaction is perfectly
    /// balanced by sweeping any remainder cents into the tax reserve account.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
    /// use logos_core::domain::rsu::AllocationPolicy;
    /// use logos_core::AccountId;
    ///
    /// let config = RsuDistributorConfig {
    ///     rsu_asset: AccountId::new("assets:rsu").unwrap(),
    ///     tax_reserve: AccountId::new("assets:tax").unwrap(),
    ///     smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
    ///     goals: AccountId::new("assets:goals").unwrap(),
    ///     discretionary: AccountId::new("assets:checking").unwrap(),
    /// };
    /// let distributor = RsuAutoDistributor::new(config);
    /// let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();
    ///
    /// // Distribute $100.00 (10,000 cents)
    /// let tx = distributor.distribute_rsu_vest("Vest 1", 10_000, &policy).unwrap();
    /// assert_eq!(tx.postings().len(), 5);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction `description` is empty, or if an invalid (non-positive)
    /// `gross_vest_cents` amount is provided.
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

        let mut builder = TransactionBuilder::new(description).posting(Posting::credit(
            self.config.rsu_asset.clone(),
            gross_vest_cents,
        )?);

        if tax_cents > 0 {
            builder = builder.posting(Posting::debit(self.config.tax_reserve.clone(), tax_cents)?);
        }
        if smoothing_cents > 0 {
            builder = builder.posting(Posting::debit(
                self.config.smoothing_buffer.clone(),
                smoothing_cents,
            )?);
        }
        if goals_cents > 0 {
            builder = builder.posting(Posting::debit(self.config.goals.clone(), goals_cents)?);
        }
        if discretionary_cents > 0 {
            builder = builder.posting(Posting::debit(
                self.config.discretionary.clone(),
                discretionary_cents,
            )?);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_distribution() {
        let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();
        let config = RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").expect("valid account id"),
            tax_reserve: AccountId::new("assets:tax").expect("valid account id"),
            smoothing_buffer: AccountId::new("assets:buffer").expect("valid account id"),
            goals: AccountId::new("assets:goals").expect("valid account id"),
            discretionary: AccountId::new("assets:checking").expect("valid account id"),
        };
        let distributor = RsuAutoDistributor::new(config);

        let tx = distributor
            .distribute_rsu_vest("Vest 1", 10000, &policy)
            .expect("should build perfectly");

        let postings = tx.postings();
        assert_eq!(postings.len(), 5);

        // the source account is credited the gross amount
        assert!(
            postings
                .contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10000).unwrap())
        );

        // the destinations are debited
        assert!(
            postings
                .contains(&Posting::debit(AccountId::new("assets:tax").unwrap(), 4000).unwrap())
        );
        assert!(
            postings
                .contains(&Posting::debit(AccountId::new("assets:buffer").unwrap(), 2000).unwrap())
        );
        assert!(
            postings
                .contains(&Posting::debit(AccountId::new("assets:goals").unwrap(), 3000).unwrap())
        );
        assert!(
            postings.contains(
                &Posting::debit(AccountId::new("assets:checking").unwrap(), 1000).unwrap()
            )
        );
    }

    #[test]
    fn test_imperfect_distribution_sweeps_to_tax() {
        let policy = AllocationPolicy::new(33, 33, 33, 1).unwrap();
        let config = RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").expect("valid account id"),
            tax_reserve: AccountId::new("assets:tax").expect("valid account id"),
            smoothing_buffer: AccountId::new("assets:buffer").expect("valid account id"),
            goals: AccountId::new("assets:goals").expect("valid account id"),
            discretionary: AccountId::new("assets:checking").expect("valid account id"),
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

        assert!(
            postings.contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10).unwrap())
        );
        assert!(
            postings.contains(&Posting::debit(AccountId::new("assets:tax").unwrap(), 4).unwrap())
        );
        assert!(
            postings
                .contains(&Posting::debit(AccountId::new("assets:buffer").unwrap(), 3).unwrap())
        );
        assert!(
            postings.contains(&Posting::debit(AccountId::new("assets:goals").unwrap(), 3).unwrap())
        );
        assert_eq!(postings.len(), 4);
    }
}
