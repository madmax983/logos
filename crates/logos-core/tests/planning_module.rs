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
            .contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10_000).unwrap())
    );
    assert!(
        tx.postings().contains(
            &Posting::debit(AccountId::new("assets:tax").unwrap(), 4_000).expect("debit")
        )
    );

    // Test explicitly sweeping exactly 0 cents to tax reserve to kill boundary mutant
    let policy_no_tax = AllocationPolicy::new(0, 100, 0, 0).expect("policy");
    let tx_no_tax = distributor
        .distribute_rsu_vest("Vest No Tax", 10, &policy_no_tax)
        .expect("tx");

    assert_eq!(tx_no_tax.postings().len(), 2);
    assert!(
        tx_no_tax.postings()
            .contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10).unwrap())
    );
    assert!(
        tx_no_tax.postings().contains(
            &Posting::debit(AccountId::new("assets:buffer").unwrap(), 10).expect("debit")
        )
    );
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

#[test]
fn planning_rsu_distributor_skips_zero_cent_postings() {
    let policy = AllocationPolicy::new(100, 0, 0, 0).expect("policy");
    let config = RsuDistributorConfig {
        rsu_asset: AccountId::new("assets:rsu").expect("valid account id"),
        tax_reserve: AccountId::new("assets:tax").expect("valid account id"),
        smoothing_buffer: AccountId::new("assets:buffer").expect("valid account id"),
        goals: AccountId::new("assets:goals").expect("valid account id"),
        discretionary: AccountId::new("assets:checking").expect("valid account id"),
    };

    let distributor = RsuAutoDistributor::new(config);
    // Vest 10 cents, so 100% goes to tax, and 0 to smoothing, goals, discretionary.
    let tx = distributor
        .distribute_rsu_vest("Vest", 10, &policy)
        .expect("tx");

    assert_eq!(tx.postings().len(), 2);
    assert!(
        tx.postings()
            .contains(&Posting::credit(AccountId::new("assets:rsu").unwrap(), 10).unwrap())
    );
    assert!(
        tx.postings()
            .contains(&Posting::debit(AccountId::new("assets:tax").unwrap(), 10).expect("debit"))
    );
}

#[test]
fn planning_net_worth_projector_uses_custom_haircut_tiers() {
    use logos_core::domain::rsu::HaircutTierTable;
    let mut projector = NetWorthProjector::new(50_000, 0);
    let custom_tiers = HaircutTierTable::new(0, 0, 0).unwrap();
    projector.set_haircut_tiers(custom_tiers);

    projector.add_upcoming_vest(UpcomingVest {
        avg_close_price_cents: 10_000,
        units: 10,
        days_to_vest: 45, // Medium tier
    });

    let (timeline, _) = projector.project_timeline(2);
    // With 0% haircut, 100,000 cents vest is fully counted.
    assert_eq!(timeline[1].net_worth_cents, 150_000);
}

#[test]
fn planning_net_worth_projector_tracks_exact_milestone_hit() {
    let mut projector = NetWorthProjector::new(50_000, 5_000);
    // We start at 50,000, and add 5,000 per month.
    // In Month 2, we should be at exactly 60,000.
    projector.add_milestone_cents(60_000);

    let (timeline, crossed) = projector.project_timeline(2);
    assert_eq!(timeline[1].net_worth_cents, 60_000);
    assert_eq!(crossed, vec![(60_000, 2)]);

    // Explicit test for the boundary mutant `replace > with >= in project_timeline`
    // We create a case where the exact milestone is hit, but a mutant using `>=` when checking
    // if days_to_vest falls strictly after month_start_days might improperly include/exclude vests.
    let mut projector_vest_boundary = NetWorthProjector::new(0, 0);
    projector_vest_boundary.add_upcoming_vest(UpcomingVest {
        avg_close_price_cents: 10_000,
        units: 10,
        days_to_vest: 30, // exactly on the month 1 / month 2 boundary
    });

    let (timeline_boundary, _) = projector_vest_boundary.project_timeline(2);

    // Month 1 window is (0, 30]. Since 30 <= 30, it vests in month 1.
    assert_eq!(timeline_boundary[0].vested_value_cents, 60_000); // 100k gross, medium tier is 40% -> 60k safe

    // Month 2 window is (30, 60]. 30 is NOT > 30, so it shouldn't vest in month 2.
    assert_eq!(timeline_boundary[1].vested_value_cents, 0);
}
