#![cfg(feature = "nova")]

//! Financial Health Score
//!
//! A module that aggregates data from FIRE, Runway, and Income models to compute a single
//! comprehensive "Health Grade" for personal finances.

use crate::experimental::runway_simulator::RunwaySimulator;
use crate::planning::fire::FireSimulator;

/// Represents the overall financial health grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthGrade {
    /// Excellent financial health (A)
    A,
    /// Good financial health (B)
    B,
    /// Average financial health (C)
    C,
    /// Poor financial health (D)
    D,
    /// Failing financial health (F)
    F,
}

/// The result of a financial health analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinancialHealthScore {
    /// The computed overall grade.
    pub grade: HealthGrade,
    /// Savings rate as a percentage (0-100).
    pub savings_rate_pct: u8,
    /// Months of runway available.
    pub runway_months: u32,
    /// Progress towards FIRE (0-100).
    pub fire_progress_pct: u8,
}

/// Analyzer that computes a comprehensive financial health score.
#[derive(Debug, Clone)]
pub struct FinancialHealthAnalyzer {
    fire_sim: FireSimulator,
    liquid_assets_cents: i64,
    monthly_income_cents: i64,
}

impl FinancialHealthAnalyzer {
    /// Creates a new `FinancialHealthAnalyzer`.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        liquid_assets_cents: i64,
        monthly_income_cents: i64,
    ) -> Self {
        Self {
            fire_sim,
            liquid_assets_cents,
            monthly_income_cents,
        }
    }

    /// Computes the overall financial health score.
    #[must_use]
    pub fn compute(&self) -> FinancialHealthScore {
        let expenses = self.fire_sim.monthly_expenses_cents();

        // 1. Calculate savings rate
        let savings_rate_pct = if self.monthly_income_cents > 0 {
            let savings = self.monthly_income_cents.saturating_sub(expenses);
            if savings <= 0 {
                0
            } else {
                let pct = (savings.saturating_mul(100)) / self.monthly_income_cents;
                std::cmp::min(100, pct).try_into().unwrap_or(100)
            }
        } else {
            0
        };

        // 2. Calculate runway
        let runway_sim = RunwaySimulator::new(self.liquid_assets_cents, expenses, 3.0); // Assume 3% inflation for conservative runway
        let runway_result = runway_sim.calculate_runway();
        let runway_months = runway_result.months;

        // 3. Calculate FIRE progress
        let fire_progress_pct = self.fire_sim.fire_progress_pct();

        // 4. Compute Grade (Simple heuristic for demonstration)
        // A: Savings > 20%, Runway > 6 months, FIRE > 50%
        // B: Savings > 10%, Runway > 3 months, FIRE > 20%
        // C: Savings > 0%, Runway > 1 month, FIRE > 5%
        // D: Zero savings, < 1 month runway
        // F: Negative savings, 0 runway
        let grade = if expenses > self.monthly_income_cents {
            HealthGrade::F
        } else if savings_rate_pct >= 20 && runway_months >= 6 && fire_progress_pct >= 50 {
            HealthGrade::A
        } else if savings_rate_pct >= 10 && runway_months >= 3 && fire_progress_pct >= 20 {
            HealthGrade::B
        } else if savings_rate_pct > 0 && runway_months >= 1 && fire_progress_pct >= 5 {
            HealthGrade::C
        } else {
            HealthGrade::D
        };

        FinancialHealthScore {
            grade,
            savings_rate_pct,
            runway_months,
            fire_progress_pct,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_grade_a() {
        let mut fire_sim = FireSimulator::new(400_000); // 4k expenses
        fire_sim.add_assets_liabilities(60_000_000, 0); // 600k NW. FIRE number = 1.2M -> 50%
        let analyzer = FinancialHealthAnalyzer::new(
            fire_sim, 5_000_000, // 50k liquid -> > 12 months runway
            600_000,   // 6k income -> 33% savings rate
        );

        let score = analyzer.compute();
        assert_eq!(score.grade, HealthGrade::A);
        assert_eq!(score.savings_rate_pct, 33);
        assert_eq!(score.fire_progress_pct, 50);
        assert!(score.runway_months > 6);
    }

    #[test]
    fn test_health_score_grade_f() {
        let fire_sim = FireSimulator::new(400_000); // 4k expenses
        let analyzer = FinancialHealthAnalyzer::new(
            fire_sim, 0,       // 0 liquid
            300_000, // 3k income -> negative savings
        );

        let score = analyzer.compute();
        assert_eq!(score.grade, HealthGrade::F);
        assert_eq!(score.savings_rate_pct, 0);
        assert_eq!(score.fire_progress_pct, 0);
        assert_eq!(score.runway_months, 0);
    }
}
