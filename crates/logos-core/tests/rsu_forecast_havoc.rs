#![allow(clippy::should_panic_without_expect)]
use logos_core::AccountId;
use logos_core::AllocationPolicy;
use logos_core::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn havoc_rsu_distribute_overflow(
        vest in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let config = RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").unwrap(),
            tax_reserve: AccountId::new("assets:tax").unwrap(),
            smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
            goals: AccountId::new("assets:goals").unwrap(),
            discretionary: AccountId::new("assets:checking").unwrap(),
        };

        let distributor = RsuAutoDistributor::new(config);
        let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();

        let tx = distributor.distribute_rsu_vest("Vest 1", vest, &policy).unwrap();
        assert!(!tx.postings().is_empty());
    }
}
