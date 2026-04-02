//! # The `logos-reporting` Library: The Dashboard
//!
//! This crate acts as the dashboard for the `logos` financial engine.
//! While `logos-core` handles the meticulous, low-level mechanics of double-entry accounting,
//! `logos-reporting` exists to answer the human questions:
//! * "Did I stick to my budget?" (`budget_vs_actual`)
//! * "Are we bleeding cash or accumulating it?" (`cashflow`)
//! * "What is my actual net worth?" (`net_worth`)
//!
//! It consumes raw primitive data and weaves it into actionable financial intelligence.

pub mod budget_vs_actual;
pub mod cashflow;
pub mod net_worth;
pub mod register;
pub mod rsu_budget_plan;
pub mod rsu_forecast;

pub use budget_vs_actual::project_budget_variance;
pub use cashflow::project_cashflow;
pub use net_worth::project_net_worth;
pub use register::{RegisterEntry, project_register_balance, project_register_balance_iter};
pub use rsu_budget_plan::{
    RsuBudgetPlan, RsuBudgetPlanInput, ScenarioBudgetProjection, ScenarioKey, ScenarioPriceInputs,
    project_rsu_budget_plan,
};
pub use rsu_forecast::{RsuForecastSummary, project_rsu_forecast_summary};
