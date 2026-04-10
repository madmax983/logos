//! # The `logos-reporting` Library
//!
//! This crate contains the projection and aggregation engines for `logos`.
//! While `logos-core` handles the strict double-entry mechanics and isolated planning
//! primitives, `logos-reporting` is responsible for answering the high-level questions:
//! "How am I doing compared to my budget?", "What is my net worth?", and "What is my cashflow?".
//!
//! It provides pure functions that take raw primitive data (e.g., balances, amounts) and
//! project them into meaningful financial indicators.

pub(crate) mod budget_vs_actual;
pub(crate) mod cashflow;
pub(crate) mod net_worth;
pub(crate) mod register;
pub(crate) mod rsu_budget_plan;
pub(crate) mod rsu_forecast;

pub use budget_vs_actual::project_budget_variance;
pub use cashflow::project_cashflow;
pub use net_worth::project_net_worth;
pub use register::{RegisterEntry, project_register_balance, project_register_balance_iter};
pub use rsu_budget_plan::{
    RsuBudgetPlan, RsuBudgetPlanInput, ScenarioBudgetProjection, ScenarioKey, ScenarioPriceInputs,
    project_rsu_budget_plan,
};
pub use rsu_forecast::{RsuForecastSummary, project_rsu_forecast_summary};
