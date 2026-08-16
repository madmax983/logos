#![cfg(feature = "nova")]

//! Tax-Loss Harvesting Engine
//!
//! Analyzes a portfolio's tax lots and identifies opportunities to harvest
//! capital losses to offset gains and reduce tax liability.

use crate::domain::account::AccountId;

/// Represents an individual tax lot (a specific purchase of an asset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxLot {
    /// The asset being held.
    pub asset: AccountId,
    /// The number of units purchased.
    pub units: u32,
    /// The cost basis per unit in cents.
    pub cost_basis_cents: i64,
}

/// A proposed trade to harvest losses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarvestingOpportunity {
    /// The asset to sell.
    pub asset: AccountId,
    /// The number of units to sell.
    pub units_to_sell: u32,
    /// The expected capital loss to be harvested (in cents, positive value indicating loss).
    pub estimated_loss_cents: i64,
}

/// Analyzes tax lots to find tax-loss harvesting opportunities.
#[derive(Debug, Clone, Default)]
pub struct TaxLossHarvester {
    lots: Vec<TaxLot>,
}

impl TaxLossHarvester {
    /// Creates a new, empty harvester.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a tax lot to the harvester's portfolio.
    pub fn add_lot(&mut self, lot: TaxLot) {
        self.lots.push(lot);
    }

    /// Evaluates the portfolio against current market prices to find harvesting opportunities.
    ///
    /// Identifies lots where the current price is lower than the cost basis.
    ///
    /// # Arguments
    /// * `current_prices` - A function or closure that returns the current price in cents for a given asset.
    #[must_use]
    pub fn find_opportunities<F>(&self, current_prices: F) -> Vec<HarvestingOpportunity>
    where
        F: Fn(&AccountId) -> Option<i64>,
    {
        let mut opportunities = Vec::new();

        for lot in &self.lots {
            if let Some(current_price) = current_prices(&lot.asset) {
                if current_price < lot.cost_basis_cents {
                    let loss_per_unit = lot.cost_basis_cents - current_price;
                    // We safely multiply knowing units is u32 and loss_per_unit is positive i64.
                    let total_loss = loss_per_unit.saturating_mul(i64::from(lot.units));

                    if total_loss > 0 {
                        opportunities.push(HarvestingOpportunity {
                            asset: lot.asset.clone(),
                            units_to_sell: lot.units,
                            estimated_loss_cents: total_loss,
                        });
                    }
                }
            }
        }

        opportunities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harvesting_opportunities() {
        let mut harvester = TaxLossHarvester::new();
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();

        // Lot 1: AAPL bought at $150
        harvester.add_lot(TaxLot {
            asset: aapl.clone(),
            units: 10,
            cost_basis_cents: 15_000,
        });

        // Lot 2: AAPL bought at $170
        harvester.add_lot(TaxLot {
            asset: aapl.clone(),
            units: 5,
            cost_basis_cents: 17_000,
        });

        // Lot 3: TSLA bought at $200
        harvester.add_lot(TaxLot {
            asset: tsla,
            units: 20,
            cost_basis_cents: 20_000,
        });

        // Current prices: AAPL is $160, TSLA is $250.
        // Lot 1 (AAPL @ 150) -> Gain of $10 per unit (No opportunity)
        // Lot 2 (AAPL @ 170) -> Loss of $10 per unit (Opportunity!)
        // Lot 3 (TSLA @ 200) -> Gain of $50 per unit (No opportunity)

        let opportunities = harvester.find_opportunities(|acc| {
            if acc.as_str() == "assets:aapl" {
                Some(16_000)
            } else if acc.as_str() == "assets:tsla" {
                Some(25_000)
            } else {
                None
            }
        });

        assert_eq!(opportunities.len(), 1);
        assert_eq!(opportunities[0].asset, aapl);
        assert_eq!(opportunities[0].units_to_sell, 5);
        assert_eq!(opportunities[0].estimated_loss_cents, 5_000); // 5 units * $10 loss = $50 (5000 cents)
    }

    #[test]
    fn test_no_opportunities_when_all_gains() {
        let mut harvester = TaxLossHarvester::new();
        let aapl = AccountId::new("assets:aapl").unwrap();

        harvester.add_lot(TaxLot {
            asset: aapl,
            units: 10,
            cost_basis_cents: 10_000,
        });

        let opportunities = harvester.find_opportunities(|_| Some(15_000));
        assert!(opportunities.is_empty());
    }

    #[test]
    fn test_handles_missing_prices_gracefully() {
        let mut harvester = TaxLossHarvester::new();
        let aapl = AccountId::new("assets:aapl").unwrap();

        harvester.add_lot(TaxLot {
            asset: aapl,
            units: 10,
            cost_basis_cents: 15_000,
        });

        let opportunities = harvester.find_opportunities(|_| None);
        assert!(opportunities.is_empty());
    }
}

#[cfg(test)]
mod sentinel_tests {
    use super::*;

    #[test]
    fn test_no_opportunities_when_price_equals_cost_basis() {
        let mut harvester = TaxLossHarvester::new();
        let aapl = AccountId::new("assets:aapl").unwrap();

        harvester.add_lot(TaxLot {
            asset: aapl,
            units: 10,
            cost_basis_cents: 10_000,
        });

        let opportunities = harvester.find_opportunities(|_| Some(10_000));
        assert!(opportunities.is_empty());
    }

    #[test]
    fn test_no_opportunities_when_units_zero() {
        let mut harvester = TaxLossHarvester::new();
        let aapl = AccountId::new("assets:aapl").unwrap();

        harvester.add_lot(TaxLot {
            asset: aapl,
            units: 0,
            cost_basis_cents: 15_000,
        });

        let opportunities = harvester.find_opportunities(|_| Some(10_000));
        assert!(opportunities.is_empty());
    }
}
