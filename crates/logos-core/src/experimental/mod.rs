//! # The Experimental Laboratory
//!
//! Welcome to the bleeding edge of the `logos` engine. This module contains beta features,
//! proof-of-concepts, and highly speculative financial simulators.
//!
//! **Why does this exist?** Financial planning is often more art than science. While the `domain`
//! enforces absolute truth (double-entry rules), this module explores "what-if" scenarios,
//! such as reverse-engineering savings rates, running Monte Carlo simulations, and tracking
//! lifestyle creep. Treat these APIs as volatile and subject to change without notice.

#[cfg(feature = "nova")]
pub mod life_energy_calculator;

#[cfg(feature = "nova")]
pub mod tax_loss_harvester;

#[cfg(feature = "nova")]
pub mod portfolio_rebalancer;

#[cfg(feature = "nova")]
pub mod asset_depreciation;
#[cfg(feature = "nova")]
pub mod benford_law;
pub mod cashflow_projector;
#[cfg(feature = "nova")]
pub mod coast_fire;
#[cfg(feature = "nova")]
pub mod debt_optimizer;
/// Simulating the FIRE journey as a mountain ascent.
pub mod fire_ascent;
#[cfg(feature = "nova")]
pub mod fire_goal_seeker;
#[cfg(feature = "nova")]
pub mod goal_fund_projector;
#[cfg(feature = "nova")]
pub mod goal_seeker;
#[cfg(feature = "nova")]
pub mod inflation;
/// Visualizing cashflows using Mermaid Sankey diagrams.
pub mod mermaid_exporter;
#[cfg(feature = "nova")]
pub mod mermaid_xy_exporter;
pub mod monte_carlo;
#[cfg(feature = "nova")]
pub mod opportunity_cost;
#[cfg(feature = "nova")]
pub mod predictive_ledger;
/// Detecting recurring transactions to aid in automated classification and projection.
pub mod recurrence_detector;
#[cfg(feature = "nova")]
pub mod trinity_simulator;

#[cfg(feature = "nova")]
pub mod anomaly_detector;
#[cfg(feature = "nova")]
pub mod category_trends;
#[cfg(feature = "nova")]
pub mod income_router;
#[cfg(feature = "nova")]
pub mod lifestyle_creep;
#[cfg(feature = "nova")]
pub mod runway_simulator;
#[cfg(feature = "nova")]
pub mod subscription_fatigue;
