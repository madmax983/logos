//! Tax Loss Harvesting Module
//!
//! Identifies capital losses that can be realized to offset capital gains.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxLot {
    pub units: u32,
    pub cost_basis_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarvestOpportunity {
    pub units_to_sell: u32,
    pub harvestable_loss_cents: i64,
}

pub struct TaxLossHarvester;

impl TaxLossHarvester {
    #[must_use]
    pub fn find_opportunities(
        lots: &[TaxLot],
        current_price_cents: i64,
    ) -> Vec<HarvestOpportunity> {
        let mut opportunities = Vec::new();
        for lot in lots {
            if lot.cost_basis_cents > current_price_cents {
                let loss_per_unit = lot.cost_basis_cents - current_price_cents;
                let total_loss = loss_per_unit * i64::from(lot.units);
                opportunities.push(HarvestOpportunity {
                    units_to_sell: lot.units,
                    harvestable_loss_cents: total_loss,
                });
            }
        }
        opportunities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_opportunities() {
        let lots = vec![
            TaxLot {
                units: 10,
                cost_basis_cents: 15_000,
            }, // $150
            TaxLot {
                units: 5,
                cost_basis_cents: 9_000,
            }, // $90
        ];

        // Price drops to $100
        let opps = TaxLossHarvester::find_opportunities(&lots, 10_000);

        assert_eq!(opps.len(), 1);
        assert_eq!(opps[0].units_to_sell, 10);
        // $50 loss per unit * 10 units = $500 loss
        assert_eq!(opps[0].harvestable_loss_cents, 50_000);
    }
}
