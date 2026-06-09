//! The Financial Planning Engine
//!
//! # The Future is Plannable
//!
//! This module forms the predictive core of the system. While the rest of `logos-core` is focused on
//! immutable history (the ledger) and strict current-state rules (budgets), the `planning` module
//! exists to model *what happens next*.
//!
//! It brings together three critical components:
//! - [`fire`]: The destination. It calculates your ultimate target number and risk-adjusted safe net worth.
//! - [`net_worth_projector`]: The journey. It simulates how your steady savings and future RSU vests
//!   will grow your net worth month over month.
//! - [`rsu_distributor`]: The action. When vests actually occur, this automates routing the funds
//!   to tax reserves, smoothing buffers, and other goals.
//!
//! # Examples
//!
//! This overarching example shows how the `fire` target and `net_worth_projector` work together
//! to plot your path to financial independence.
//!
//! ```
//! use logos_core::{FireSimulator, UpcomingVest};
//! use logos_core::NetWorthProjector;
//!
//! // 1. Set the Destination: $5,000/month expenses = $1.5M FIRE number @ 4% SWR
//! let mut fire_sim = FireSimulator::new(500_000);
//! fire_sim.add_assets_liabilities(50_000_00, 0); // Starting with $50k
//! let target_fire_cents = fire_sim.fire_number_cents();
//!
//! // 2. Set the Journey: Projecting from our $50k base, saving $1k/month
//! let mut projector = NetWorthProjector::new(50_000_00, 1_000_00);
//!
//! // Track our FIRE number as a milestone
//! projector.add_milestone_cents(target_fire_cents);
//!
//! // 3. Add upcoming vests to both to see their impact
//! let vest = UpcomingVest {
//!     avg_close_price_cents: 10_000,
//!     units: 15_000,
//!     days_to_vest: 30, // Next month
//! };
//! fire_sim.add_upcoming_vest(vest);
//! projector.add_upcoming_vest(vest);
//!
//! // 4. Simulate the next 5 months
//! let (timeline, milestones) = projector.project_timeline(5);
//!
//! // We can see our safe net worth jump in month 1 when the RSU vests
//! assert_eq!(timeline.len(), 5);
//! ```

pub mod fire;
pub mod net_worth_projector;
pub mod rsu_distributor;
