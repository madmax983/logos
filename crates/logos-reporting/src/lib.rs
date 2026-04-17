//! # The `logos-reporting` Library
//!
//! This crate contains the projection and aggregation engines for `logos`.
//! While `logos-core` handles the strict double-entry mechanics and isolated planning
//! primitives, `logos-reporting` is responsible for answering the high-level questions:
//! "How am I doing compared to my budget?", "What is my net worth?", and "What is my cashflow?".
//!
//! It provides pure functions that take raw primitive data (e.g., balances, amounts) and
//! project them into meaningful financial indicators.
//!
//! ## The Big Picture
//!
//! Financial reporting is ultimately about combining these indicators to build a complete
//! picture of health. The pure functions here can be composed together.
//!
//! ## Examples
//!
//! ```
//! use logos_reporting::{project_cashflow, project_net_worth, project_register_balance, RegisterEntry};
//!
//! // 1. Check liquidity: We start with $1,000 in the bank, and make some deposits and withdrawals.
//! let entries = [RegisterEntry::new(500_00), RegisterEntry::new(-200_00)];
//! let final_bank_balance = project_register_balance(1000_00, &entries);
//! assert_eq!(final_bank_balance, 1300_00);
//!
//! // 2. Check profitability: Our monthly income is $5,000 and expenses are $4,000.
//! let surplus = project_cashflow(5000_00, 4000_00);
//! assert_eq!(surplus, 1000_00);
//!
//! // 3. Check overall health: We have $15,000 in total assets and $5,000 in liabilities.
//! let total_net_worth = project_net_worth(15000_00, 5000_00);
//! assert_eq!(total_net_worth, 10000_00);
//! ```

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
