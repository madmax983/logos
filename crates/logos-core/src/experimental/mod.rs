/// Tools for projecting cash flow over time.
pub mod cashflow_projector;
#[cfg(feature = "nova")]
pub mod debt_optimizer;
/// Alternative modeling for FIRE paths.
pub mod fire_ascent;
#[cfg(feature = "nova")]
pub mod inflation;
/// Exports ledger graphs to Mermaid format.
pub mod mermaid_exporter;
pub mod monte_carlo;
/// Detects recurring transactions automatically.
pub mod recurrence_detector;
