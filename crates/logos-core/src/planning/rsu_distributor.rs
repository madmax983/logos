use crate::domain::account::AccountId;
use crate::domain::rsu::AllocationPolicy;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;

/// Configuration for `RsuAutoDistributor`, preventing positional string arguments.
#[derive(Debug, Clone)]
pub struct RsuDistributorConfig {
    pub rsu_asset: AccountId,
    pub tax_reserve: AccountId,
    pub smoothing_buffer: AccountId,
    pub goals: AccountId,
    pub discretionary: AccountId,
}

/// Automatically distributes vested RSU funds across target accounts
/// according to an `AllocationPolicy`.
#[derive(Debug, Clone)]
pub struct RsuAutoDistributor {
    config: RsuDistributorConfig,
}

impl RsuAutoDistributor {
    #[must_use]
    pub const fn new(config: RsuDistributorConfig) -> Self {
        Self { config }
    }

    /// Distributes a gross vest amount across the configured accounts, ensuring perfectly balanced transactions.
    /// Remainder cents from percentage division are swept into the tax reserve account.
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
            .posting(Posting::credit(
                self.config.rsu_asset.clone(),
                gross_vest_cents,
            )?)
            .posting(Posting::debit(self.config.tax_reserve.clone(), tax_cents))
            .posting(Posting::debit(
                self.config.smoothing_buffer.clone(),
                smoothing_cents,
            ))
            .posting(Posting::debit(self.config.goals.clone(), goals_cents))
            .posting(Posting::debit(
                self.config.discretionary.clone(),
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
        assert!(postings.contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10000).unwrap()));

        // the destinations are debited
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:tax").unwrap(), 4000)));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:buffer").unwrap(), 2000)));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:goals").unwrap(), 3000)));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:checking").unwrap(), 1000)));
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

        assert!(postings.contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10).unwrap()));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:tax").unwrap(), 4)));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:buffer").unwrap(), 3)));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:goals").unwrap(), 3)));
        assert!(postings.contains(&Posting::debit(AccountId::new("assets:checking").unwrap(), 0)));
    }
}
