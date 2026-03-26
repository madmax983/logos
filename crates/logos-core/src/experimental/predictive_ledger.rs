//! Predictive Ledger simulation.
//!
//! Generates future double-entry transactions from forecasted events (like RSU vests).

use crate::domain::rsu::{AllocationPolicy, HaircutTierTable, forecast_value_cents};
use crate::domain::transaction::Transaction;
use crate::error::DomainError;
use crate::planning::fire::UpcomingVest;
use crate::planning::rsu_distributor::RsuAutoDistributor;

/// A simulator that converts future unvested equity into a series of
/// balanced, strict double-entry transactions.
///
/// Instead of just projecting a single "net worth" number, this generates
/// the exact ledger postings that will occur when RSUs vest and are
/// automatically distributed according to your policy.
#[derive(Debug, Clone)]
pub struct PredictiveLedger {
    distributor: RsuAutoDistributor,
    policy: AllocationPolicy,
    haircut_tiers: HaircutTierTable,
    upcoming_vests: Vec<UpcomingVest>,
}

impl PredictiveLedger {
    /// Creates a new `PredictiveLedger` configured with a distributor and policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_core::planning::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
    /// use logos_core::domain::rsu::AllocationPolicy;
    /// use logos_core::experimental::predictive_ledger::PredictiveLedger;
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
    /// let ledger = PredictiveLedger::new(distributor, policy);
    /// ```
    #[must_use]
    pub fn new(distributor: RsuAutoDistributor, policy: AllocationPolicy) -> Self {
        Self {
            distributor,
            policy,
            haircut_tiers: HaircutTierTable::default(),
            upcoming_vests: Vec::new(),
        }
    }

    /// Updates the haircut tiers used to discount future vests.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_core::domain::rsu::HaircutTierTable;
    /// use logos_core::planning::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
    /// use logos_core::domain::rsu::AllocationPolicy;
    /// use logos_core::experimental::predictive_ledger::PredictiveLedger;
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
    /// let mut ledger = PredictiveLedger::new(distributor, policy);
    /// ledger.set_haircut_tiers(HaircutTierTable::new(10, 20, 30).unwrap());
    /// ```
    pub const fn set_haircut_tiers(&mut self, tiers: HaircutTierTable) {
        self.haircut_tiers = tiers;
    }

    /// Adds an upcoming RSU vest to the simulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_core::planning::fire::UpcomingVest;
    /// use logos_core::planning::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
    /// use logos_core::domain::rsu::AllocationPolicy;
    /// use logos_core::experimental::predictive_ledger::PredictiveLedger;
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
    /// let mut ledger = PredictiveLedger::new(distributor, policy);
    /// ledger.add_upcoming_vest(UpcomingVest {
    ///     avg_close_price_cents: 10_000,
    ///     units: 100,
    ///     days_to_vest: 60,
    /// });
    /// ```
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Projects all upcoming vests into a series of ledger transactions.
    ///
    /// Vests are risk-adjusted using the configured haircut tiers. If a vest's
    /// projected safe value is zero (e.g., due to extreme haircuts or low prices),
    /// it is skipped and no transaction is generated.
    ///
    /// # Errors
    /// Returns a `DomainError` if the transaction builder fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_core::planning::fire::UpcomingVest;
    /// use logos_core::planning::rsu_distributor::{RsuDistributorConfig, RsuAutoDistributor};
    /// use logos_core::domain::rsu::AllocationPolicy;
    /// use logos_core::experimental::predictive_ledger::PredictiveLedger;
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
    /// let mut ledger = PredictiveLedger::new(distributor, policy);
    /// ledger.add_upcoming_vest(UpcomingVest {
    ///     avg_close_price_cents: 10_000,
    ///     units: 1000,
    ///     days_to_vest: 15,
    /// });
    ///
    /// let txs = ledger.project_transactions().unwrap();
    /// assert_eq!(txs.len(), 1);
    /// ```
    pub fn project_transactions(&self) -> Result<Vec<Transaction>, DomainError> {
        let mut transactions = Vec::with_capacity(self.upcoming_vests.len());

        for (i, vest) in self.upcoming_vests.iter().enumerate() {
            let safe_value = forecast_value_cents(
                vest.avg_close_price_cents,
                vest.units,
                vest.days_to_vest,
                &self.haircut_tiers,
            );

            // Skip zero-value vests
            if safe_value <= 0 {
                continue;
            }

            let description = format!("Projected Vest {} ({} days)", i + 1, vest.days_to_vest);
            let tx =
                self.distributor
                    .distribute_rsu_vest(&description, safe_value, &self.policy)?;

            transactions.push(tx);
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::planning::rsu_distributor::RsuDistributorConfig;

    #[test]
    fn test_predictive_ledger_generation() {
        let config = RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").unwrap(),
            tax_reserve: AccountId::new("assets:tax").unwrap(),
            smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
            goals: AccountId::new("assets:goals").unwrap(),
            discretionary: AccountId::new("assets:checking").unwrap(),
        };
        let distributor = RsuAutoDistributor::new(config);
        let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();

        let mut ledger = PredictiveLedger::new(distributor, policy);

        // Add a vest: 1000 units @ $100 ($100k gross), vesting in 15 days (short tier, 25% haircut by default)
        // Safe value = 75% of $100k = $75k (7_500_000 cents)
        ledger.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 1000,
            days_to_vest: 15,
        });

        // Add a zero-value vest (e.g. price is 0)
        ledger.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 0,
            units: 500,
            days_to_vest: 60,
        });

        let txs = ledger
            .project_transactions()
            .expect("failed to project transactions");

        // The 0-value vest should be skipped
        assert_eq!(txs.len(), 1);

        let tx = &txs[0];
        assert_eq!(tx.description(), "Projected Vest 1 (15 days)");

        // Check if the total credits are exactly 7_500_000
        let mut total_credit = 0;
        for p in tx.postings() {
            if p.amount() < 0 {
                // In logos, credits are negative
                total_credit += p.amount().abs();
            }
        }
        assert_eq!(total_credit, 7_500_000);
    }
}
