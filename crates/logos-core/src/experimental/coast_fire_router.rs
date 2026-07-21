#![cfg(feature = "nova")]
//! Coast FIRE Router Module
//!
//! Automatically shifts income routing behavior based on Coast FIRE status.
//!
//! 🌟 Nova Mashup: Mashing up `CoastFireSimulator` and `IncomeRouter`. When you're
//! grinding to Coast FIRE, your paycheck goes heavily into investments. Once you cross
//! the Coast FIRE milestone, the router automatically flips to a "Coast Mode" where
//! you might route more income to lifestyle or cash reserves, knowing your retirement
//! is already funded!

use crate::domain::transaction::Transaction;
use crate::error::DomainError;
use crate::experimental::coast_fire::CoastFireSimulator;
use crate::experimental::income_router::IncomeRouter;

/// Routes income dynamically depending on Coast FIRE milestone status.
#[derive(Debug, Clone)]
pub struct CoastFireRouter {
    coast_fire_sim: CoastFireSimulator,
    grind_router: IncomeRouter,
    coast_router: IncomeRouter,
}

impl CoastFireRouter {
    /// Creates a new `CoastFireRouter`.
    #[must_use]
    pub const fn new(
        coast_fire_sim: CoastFireSimulator,
        grind_router: IncomeRouter,
        coast_router: IncomeRouter,
    ) -> Self {
        Self {
            coast_fire_sim,
            grind_router,
            coast_router,
        }
    }

    /// Checks if we are currently coasting.
    #[must_use]
    pub fn is_coasting(&self) -> bool {
        self.coast_fire_sim.calculate().is_coasting
    }

    /// Routes the income using the appropriate router depending on Coast FIRE status.
    ///
    /// # Errors
    /// Returns a `DomainError` if the routing amount is invalid or routing fails.
    pub fn route_income(
        &self,
        description: &str,
        amount_cents: i64,
    ) -> Result<Transaction, DomainError> {
        if self.is_coasting() {
            self.coast_router.route_income(description, amount_cents)
        } else {
            self.grind_router.route_income(description, amount_cents)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::Posting;
    use crate::experimental::income_router::RouteRule;
    use crate::planning::fire::FireSimulator;

    #[test]
    fn test_grind_routing() {
        let mut fire_sim = FireSimulator::new(400_000); // $4k/mo expenses
        fire_sim.add_assets_liabilities(5_000_000, 0); // $50k current, well below Coast FIRE (needs ~300k)
        let coast_sim = CoastFireSimulator::new(fire_sim, 7.0, 20);

        let source = AccountId::new("income:salary").unwrap();
        let inv_dest = AccountId::new("assets:investments").unwrap();
        let life_dest = AccountId::new("expenses:lifestyle").unwrap();

        let grind_router = IncomeRouter::new(
            source.clone(),
            vec![
                RouteRule {
                    destination: inv_dest.clone(),
                    percentage: 90,
                },
                RouteRule {
                    destination: life_dest.clone(),
                    percentage: 10,
                },
            ],
        )
        .unwrap();

        let coast_router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: inv_dest.clone(),
                    percentage: 10,
                },
                RouteRule {
                    destination: life_dest,
                    percentage: 90,
                },
            ],
        )
        .unwrap();

        let router = CoastFireRouter::new(coast_sim, grind_router, coast_router);

        assert!(!router.is_coasting());

        let tx = router.route_income("Paycheck", 1000).unwrap();
        let postings = tx.postings();
        assert!(postings.contains(&Posting::debit(inv_dest, 900).unwrap()));
    }

    #[test]
    fn test_coast_routing() {
        let mut fire_sim = FireSimulator::new(400_000);
        fire_sim.add_assets_liabilities(40_000_000, 0); // $400k current, above Coast FIRE
        let coast_sim = CoastFireSimulator::new(fire_sim, 7.0, 20);

        let source = AccountId::new("income:salary").unwrap();
        let inv_dest = AccountId::new("assets:investments").unwrap();
        let life_dest = AccountId::new("expenses:lifestyle").unwrap();

        let grind_router = IncomeRouter::new(
            source.clone(),
            vec![
                RouteRule {
                    destination: inv_dest.clone(),
                    percentage: 90,
                },
                RouteRule {
                    destination: life_dest.clone(),
                    percentage: 10,
                },
            ],
        )
        .unwrap();

        let coast_router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: inv_dest,
                    percentage: 10,
                },
                RouteRule {
                    destination: life_dest.clone(),
                    percentage: 90,
                },
            ],
        )
        .unwrap();

        let router = CoastFireRouter::new(coast_sim, grind_router, coast_router);

        assert!(router.is_coasting());

        let tx = router.route_income("Paycheck", 1000).unwrap();
        let postings = tx.postings();
        assert!(postings.contains(&Posting::debit(life_dest, 900).unwrap()));
    }
}
