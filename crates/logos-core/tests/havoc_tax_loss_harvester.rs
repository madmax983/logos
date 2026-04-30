#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]

use logos_core::AccountId;
use logos_core::tax_loss_harvester::{TaxLossHarvester, TaxLot};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn test_tax_loss_harvester_panics_on_overflow(
        cost in (i64::MAX / 2 + 1)..=i64::MAX,
        price in i64::MIN..=(i64::MIN / 2),
    ) {
        let mut harvester = TaxLossHarvester::new();
        let asset = AccountId::new("assets:aapl").unwrap();
        harvester.add_lot(TaxLot {
            asset,
            units: 100,
            cost_basis_cents: cost,
        });

        let _ = harvester.find_opportunities(|_| Some(price));
    }
}
