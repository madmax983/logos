use logos_core::domain::account::AccountId;
use logos_core::domain::rsu::AllocationPolicy;
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};

fn main() {
    let policy = AllocationPolicy::new(40, 20, 30, 10).expect("valid 100% allocation");
    let config = RsuDistributorConfig {
        rsu_asset: AccountId::new("assets:rsu"),
        tax_reserve: AccountId::new("assets:tax"),
        smoothing_buffer: AccountId::new("assets:buffer"),
        goals: AccountId::new("assets:goals"),
        discretionary: AccountId::new("assets:checking"),
    };

    let distributor = RsuAutoDistributor::new(config);
    let tx = distributor
        .distribute_rsu_vest("RSU vest Mar 2026", 100_000, &policy)
        .expect("balanced transaction");
    assert_eq!(tx.postings().len(), 5);
}
