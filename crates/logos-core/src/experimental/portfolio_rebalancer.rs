use crate::domain::account::AccountId;
use crate::domain::transaction::Posting;
use crate::error::DomainError;
use std::collections::HashMap;

/// Represents the target allocation percentage for a specific account.
/// The percentage should be a value between `0.0` and `100.0`.
#[derive(Debug, Clone, Copy)]
pub struct TargetAllocation {
    pub percentage: f64,
}

/// Calculates the exact double-entry `Posting`s required to rebalance a portfolio
/// to a target asset allocation.
#[derive(Debug, Default)]
pub struct PortfolioRebalancer {}

impl PortfolioRebalancer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the required postings to rebalance current asset values to the target allocation.
    ///
    /// The returned list of `Posting`s will always sum to zero. Any penny rounding remainder
    /// caused by the distribution is applied to the account with the largest target amount.
    ///
    /// # Arguments
    /// * `current_balances` - A map of account IDs to their current balance in cents.
    /// * `target_allocations` - A map of account IDs to their target percentage (0.0 to 100.0).
    ///
    /// # Returns
    /// A list of `Posting`s representing the required buys (debits) and sells (credits).
    ///
    /// # Errors
    /// Returns a `DomainError` if creating any `Posting` fails (e.g. invalid account, negative amounts, or overflow limits).
    pub fn calculate_rebalance(
        &self,
        current_balances: &HashMap<AccountId, i64>,
        target_allocations: &HashMap<AccountId, TargetAllocation>,
    ) -> Result<Vec<Posting>, DomainError> {
        let total_value: i64 = current_balances.values().sum();
        if total_value <= 0 {
            return Ok(Vec::new());
        }

        let mut target_cents: HashMap<AccountId, i64> = HashMap::new();
        let mut allocated_total = 0;
        let mut largest_account: Option<(AccountId, i64)> = None;

        for (account, target) in target_allocations {
            #[allow(clippy::cast_precision_loss)]
            let raw_target = total_value as f64 * (target.percentage / 100.0);
            #[allow(clippy::cast_possible_truncation)]
            let account_target = raw_target.round() as i64;
            target_cents.insert(account.clone(), account_target);
            allocated_total += account_target;

            match &largest_account {
                Some((max_id, max_val)) => {
                    if account_target > *max_val
                        || (account_target == *max_val && account.as_str() > max_id.as_str())
                    {
                        largest_account = Some((account.clone(), account_target));
                    }
                }
                None => {
                    largest_account = Some((account.clone(), account_target));
                }
            }
        }

        // Adjust for penny rounding error on the largest account
        let remainder = total_value - allocated_total;
        if remainder != 0 {
            if let Some((largest_id, _)) = largest_account {
                if let Some(val) = target_cents.get_mut(&largest_id) {
                    *val += remainder;
                }
            }
        }

        let mut postings = Vec::new();
        for (account, target_amount) in target_cents {
            let current_amount = current_balances.get(&account).copied().unwrap_or(0);
            let diff = target_amount - current_amount;

            if diff > 0 {
                // We need more of this asset (buy/debit)
                postings.push(Posting::debit(account, diff)?);
            } else if diff < 0 {
                // We need less of this asset (sell/credit)
                postings.push(Posting::credit(account, diff.abs())?);
            }
        }

        Ok(postings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_rebalance() {
        let mut current_balances = HashMap::new();
        let eq = AccountId::new("assets:equities").unwrap();
        let bnd = AccountId::new("assets:bonds").unwrap();

        current_balances.insert(eq.clone(), 700_000); // 70%
        current_balances.insert(bnd.clone(), 300_000); // 30%

        let mut target_allocations = HashMap::new();
        target_allocations.insert(eq.clone(), TargetAllocation { percentage: 50.0 });
        target_allocations.insert(bnd.clone(), TargetAllocation { percentage: 50.0 });

        let rebalancer = PortfolioRebalancer::new();
        let mut postings = rebalancer
            .calculate_rebalance(&current_balances, &target_allocations)
            .unwrap();

        postings.sort_by(|a, b| a.account().as_str().cmp(b.account().as_str()));

        assert_eq!(postings.len(), 2);

        // assets:bonds needs to increase from 3000 to 5000 (+2000 debit)
        assert_eq!(postings[0].account(), &bnd);
        assert_eq!(postings[0].amount(), 200_000);

        // assets:equities needs to decrease from 7000 to 5000 (-2000 credit)
        assert_eq!(postings[1].account(), &eq);
        assert_eq!(postings[1].amount(), -200_000);
    }

    #[test]
    fn test_penny_rounding_distribution() {
        let mut current_balances = HashMap::new();
        let a = AccountId::new("assets:a").unwrap();
        let b = AccountId::new("assets:b").unwrap();
        let c = AccountId::new("assets:c").unwrap();

        current_balances.insert(a.clone(), 10_000); // 10000 cents total

        let mut target_allocations = HashMap::new();
        // 10000 / 3 = 3333.33 -> 3333 cents
        target_allocations.insert(
            a.clone(),
            TargetAllocation {
                percentage: 33.3333,
            },
        );
        target_allocations.insert(
            b.clone(),
            TargetAllocation {
                percentage: 33.3333,
            },
        );
        // Make C slightly larger so it takes the rounding remainder
        target_allocations.insert(
            c.clone(),
            TargetAllocation {
                percentage: 33.3334,
            },
        );

        let rebalancer = PortfolioRebalancer::new();
        let mut postings = rebalancer
            .calculate_rebalance(&current_balances, &target_allocations)
            .unwrap();

        postings.sort_by(|a, b| a.account().as_str().cmp(b.account().as_str()));

        assert_eq!(postings.len(), 3);

        // a needs to go from 10000 to 3333 (-6667 credit)
        assert_eq!(postings[0].account(), &a);
        assert_eq!(postings[0].amount(), -6667);

        // b needs to go from 0 to 3333 (+3333 debit)
        assert_eq!(postings[1].account(), &b);
        assert_eq!(postings[1].amount(), 3333);

        // c needs to go from 0 to 3334 (+3334 debit) (Takes the 1 cent remainder)
        assert_eq!(postings[2].account(), &c);
        assert_eq!(postings[2].amount(), 3334);

        let sum: i64 = postings
            .iter()
            .map(crate::domain::transaction::Posting::amount)
            .sum();
        assert_eq!(sum, 0); // Must be perfectly balanced
    }
}
