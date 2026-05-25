cat << 'INNER_EOF' > src/main.rs
use logos_core::fire::{FireSimulator, UpcomingVest};
use logos_core::net_worth_projector::NetWorthProjector;

fn main() {
    // 1. Set the Destination: $5,000/month expenses = $1.5M FIRE number @ 4% SWR
    let mut fire_sim = FireSimulator::new(500_000);
    fire_sim.add_assets_liabilities(50_000_00, 0); // Starting with $50k
    let target_fire_cents = fire_sim.fire_number_cents();

    // 2. Set the Journey: Projecting from our $50k base, saving $1k/month
    let mut projector = NetWorthProjector::new(50_000_00, 1_000_00);

    // Track our FIRE number as a milestone
    projector.add_milestone_cents(target_fire_cents);

    // 3. Add upcoming vests to both to see their impact
    let vest = UpcomingVest {
        avg_close_price_cents: 10_000,
        units: 15_000,
        days_to_vest: 30, // Next month
    };
    fire_sim.add_upcoming_vest(vest);
    projector.add_upcoming_vest(vest);

    // 4. Simulate the next 5 months
    let (timeline, milestones) = projector.project_timeline(5);

    // We can see our safe net worth jump in month 1 when the RSU vests
    assert_eq!(timeline.len(), 5);
}
INNER_EOF
cargo run
