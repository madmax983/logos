#![allow(
    clippy::redundant_pub_crate,
)]
#[cfg(feature = "nova")]
pub(crate) mod life_energy_calculator;

#[cfg(feature = "nova")]
pub(crate) mod tax_loss_harvester;

#[cfg(feature = "nova")]
pub(crate) mod portfolio_rebalancer;

#[cfg(feature = "nova")]
pub(crate) mod asset_depreciation;
#[cfg(feature = "nova")]
pub(crate) mod benford_law;
pub(crate) mod cashflow_projector;
#[cfg(feature = "nova")]
pub(crate) mod coast_fire;
#[cfg(feature = "nova")]
pub(crate) mod debt_optimizer;
/// Simulating the FIRE journey as a mountain ascent.
pub(crate) mod fire_ascent;
#[cfg(feature = "nova")]
pub(crate) mod fire_goal_seeker;
#[cfg(feature = "nova")]
pub(crate) mod goal_fund_projector;
#[cfg(feature = "nova")]
pub(crate) mod goal_seeker;
#[cfg(feature = "nova")]
pub(crate) mod inflation;
/// Visualizing cashflows using Mermaid Sankey diagrams.
pub(crate) mod mermaid_exporter;
#[cfg(feature = "nova")]
pub(crate) mod mermaid_xy_exporter;
pub(crate) mod monte_carlo;
#[cfg(feature = "nova")]
pub(crate) mod opportunity_cost;
#[cfg(feature = "nova")]
pub(crate) mod predictive_ledger;
/// Detecting recurring transactions to aid in automated classification and projection.
pub(crate) mod recurrence_detector;
#[cfg(feature = "nova")]
pub(crate) mod trinity_simulator;

#[cfg(feature = "nova")]
pub(crate) mod anomaly_detector;
#[cfg(feature = "nova")]
pub(crate) mod category_trends;
#[cfg(feature = "nova")]
pub(crate) mod income_router;
#[cfg(feature = "nova")]
pub(crate) mod lifestyle_creep;
#[cfg(feature = "nova")]
pub(crate) mod runway_simulator;
#[cfg(feature = "nova")]
pub(crate) mod subscription_fatigue;
