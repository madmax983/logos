#![cfg(feature = "nova")]

//! Financial Health Score Calculator
//!
//! Evaluates a user's financial state against standard personal finance rules of thumb
//! to produce an overall "Health Score" (0-100) and specific actionable grades.

/// Represents a user's current financial snapshot for health evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinancialHealthProfile {
    /// Total readily available cash (checking, savings).
    pub liquid_assets_cents: i64,
    /// Typical monthly expenses (burn rate).
    pub monthly_expenses_cents: i64,
    /// Typical monthly gross income.
    pub monthly_income_cents: i64,
    /// Total monthly debt obligations (mortgage, student loans, car, etc).
    pub total_debt_payments_cents: i64,
    /// Total outstanding debt with high interest rates (e.g., >8% like credit cards).
    pub total_high_interest_debt_cents: i64,
}

/// The qualitative grade of a specific financial indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthGrade {
    /// Exceeds standard rules of thumb.
    Excellent,
    /// Meets basic standards.
    Good,
    /// Borderline or needs improvement.
    Warning,
    /// Dangerously out of bounds.
    Critical,
}

/// The result of evaluating a single financial indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndicatorResult {
    /// The name of the metric.
    pub name: String,
    /// A description of the finding.
    pub description: String,
    /// The qualitative grade.
    pub grade: HealthGrade,
    /// Points this indicator contributed.
    pub points_awarded: u8,
    /// Maximum possible points for this indicator.
    pub max_points: u8,
}

/// The complete health score report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthScoreReport {
    /// The total score (0-100).
    pub overall_score: u8,
    /// The individual indicator evaluations.
    pub indicators: Vec<IndicatorResult>,
}

/// Evaluates a `FinancialHealthProfile` to produce a `HealthScoreReport`.
#[derive(Debug, Clone)]
pub struct HealthScoreCalculator;

impl HealthScoreCalculator {
    /// Evaluates the given profile.
    #[must_use]
    pub fn evaluate(profile: &FinancialHealthProfile) -> HealthScoreReport {
        let mut indicators = Vec::new();
        let mut total_score = 0;

        // 1. Emergency Fund (Max 30 points)
        let runway_months = if profile.monthly_expenses_cents > 0 {
            profile.liquid_assets_cents / profile.monthly_expenses_cents
        } else {
            999
        };

        let (ef_grade, ef_points) = match runway_months {
            6..=i64::MAX => (HealthGrade::Excellent, 30),
            3..=5 => (HealthGrade::Good, 20),
            1..=2 => (HealthGrade::Warning, 10),
            _ => (HealthGrade::Critical, 0),
        };

        total_score += ef_points;
        indicators.push(IndicatorResult {
            name: "Emergency Fund".to_string(),
            description: format!("You have {} months of runway.", runway_months.min(999)),
            grade: ef_grade,
            points_awarded: ef_points,
            max_points: 30,
        });

        // 2. Savings Rate (Max 30 points)
        let savings_rate_pct = if profile.monthly_income_cents > 0 {
            let savings = profile
                .monthly_income_cents
                .saturating_sub(profile.monthly_expenses_cents);
            (savings * 100) / profile.monthly_income_cents
        } else {
            0
        };

        let (sr_grade, sr_points) = match savings_rate_pct {
            20..=i64::MAX => (HealthGrade::Excellent, 30),
            10..=19 => (HealthGrade::Good, 20),
            0..=9 => (HealthGrade::Warning, 10),
            _ => (HealthGrade::Critical, 0),
        };

        total_score += sr_points;
        indicators.push(IndicatorResult {
            name: "Savings Rate".to_string(),
            description: format!(
                "You are saving {}% of your income.",
                savings_rate_pct.max(0)
            ),
            grade: sr_grade,
            points_awarded: sr_points,
            max_points: 30,
        });

        // 3. Debt-to-Income (DTI) (Max 20 points)
        let dti_pct = if profile.monthly_income_cents > 0 {
            (profile.total_debt_payments_cents * 100) / profile.monthly_income_cents
        } else {
            100
        };

        let (dti_grade, dti_points) = match dti_pct {
            0..=15 => (HealthGrade::Excellent, 20),
            16..=36 => (HealthGrade::Good, 15),
            37..=49 => (HealthGrade::Warning, 5),
            _ => (HealthGrade::Critical, 0),
        };

        total_score += dti_points;
        indicators.push(IndicatorResult {
            name: "Debt-to-Income Ratio".to_string(),
            description: format!("Your DTI is {}%.", dti_pct.min(100)),
            grade: dti_grade,
            points_awarded: dti_points,
            max_points: 20,
        });

        // 4. High-Interest Debt (Max 20 points)
        let (hi_grade, hi_points) = if profile.total_high_interest_debt_cents == 0 {
            (HealthGrade::Excellent, 20)
        } else {
            (HealthGrade::Critical, 0)
        };

        total_score += hi_points;
        indicators.push(IndicatorResult {
            name: "High-Interest Debt".to_string(),
            description: if profile.total_high_interest_debt_cents == 0 {
                "No high-interest debt detected.".to_string()
            } else {
                "High-interest debt is dragging down your financial health.".to_string()
            },
            grade: hi_grade,
            points_awarded: hi_points,
            max_points: 20,
        });

        HealthScoreReport {
            overall_score: total_score,
            indicators,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn test_perfect_score() {
        let profile = FinancialHealthProfile {
            liquid_assets_cents: 60_000_00,
            monthly_expenses_cents: 10_000_00,
            monthly_income_cents: 15_000_00,
            total_debt_payments_cents: 1_500_00,
            total_high_interest_debt_cents: 0,
        };

        let report = HealthScoreCalculator::evaluate(&profile);
        assert_eq!(report.overall_score, 100);
        assert!(
            report
                .indicators
                .iter()
                .all(|i| i.grade == HealthGrade::Excellent)
        );
    }

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn test_critical_score() {
        let profile = FinancialHealthProfile {
            liquid_assets_cents: 500_00,
            monthly_expenses_cents: 10_000_00,
            monthly_income_cents: 8_000_00,
            total_debt_payments_cents: 5_000_00,
            total_high_interest_debt_cents: 20_000_00,
        };

        let report = HealthScoreCalculator::evaluate(&profile);
        assert_eq!(report.overall_score, 0);
        assert!(
            report
                .indicators
                .iter()
                .all(|i| i.grade == HealthGrade::Critical)
        );
    }
}
