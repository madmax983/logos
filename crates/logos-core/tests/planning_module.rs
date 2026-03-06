use logos_core::Posting;
use logos_core::domain::account::AccountId;
use logos_core::domain::rsu::AllocationPolicy;
use logos_core::planning::fire::{FireSimulator, UpcomingVest};
use logos_core::planning::net_worth_projector::NetWorthProjector;
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};

#[test]
fn planning_rsu_distributor_builds_balanced_transaction() {
    let policy = AllocationPolicy::new(40, 20, 30, 10).expect("policy");
    let config = RsuDistributorConfig {
        rsu_asset: AccountId::new("assets:rsu").expect("valid account id"),
        tax_reserve: AccountId::new("assets:tax").expect("valid account id"),
        smoothing_buffer: AccountId::new("assets:buffer").expect("valid account id"),
        goals: AccountId::new("assets:goals").expect("valid account id"),
        discretionary: AccountId::new("assets:checking").expect("valid account id"),
    };

    let distributor = RsuAutoDistributor::new(config);
    let tx = distributor
        .distribute_rsu_vest("Vest", 10_000, &policy)
        .expect("tx");

    assert_eq!(tx.postings().len(), 5);
    assert!(
        tx.postings()
            .contains(&Posting::credit("assets:rsu", 10_000).unwrap())
    );
    assert!(tx.postings().contains(&Posting::debit("assets:tax", 4_000)));
}

#[test]
fn planning_fire_simulator_reports_progress() {
    let mut sim = FireSimulator::new(400_000);
    sim.add_assets_liabilities(35_000_000, 5_000_000);
    sim.add_upcoming_vest(UpcomingVest {
        avg_close_price_cents: 100_000,
        units: 500,
        days_to_vest: 60,
    });

    assert_eq!(sim.fire_progress_pct(), 50);
}

#[test]
fn planning_net_worth_projection_tracks_milestone() {
    let mut projector = NetWorthProjector::new(50_000, 5_000);
    projector.add_upcoming_vest(UpcomingVest {
        avg_close_price_cents: 10_000,
        units: 10,
        days_to_vest: 45,
    });
    projector.add_milestone_cents(100_000);

    let (timeline, crossed) = projector.project_timeline(3);
    assert_eq!(timeline[1].net_worth_cents, 120_000);
    assert_eq!(crossed, vec![(100_000, 2)]);
}
