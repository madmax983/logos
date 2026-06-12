//! Historical Portfolio Stress Tester
//!
//! Applies the exact sequence of returns from major historical market crashes
//! to your current portfolio to calculate your maximum drawdown and runway survival.

#[derive(Debug, Clone, PartialEq)]
pub struct StressTestResult {
    /// Name of the scenario
    pub scenario_name: String,
    /// Starting portfolio value
    pub starting_cents: i64,
    /// Minimum portfolio value hit during the crash
    pub trough_cents: i64,
    /// Maximum percentage drawdown (0.0 to 100.0)
    pub max_drawdown_pct: f64,
    /// Number of months until the portfolio recovered to the starting value, or None if it didn't
    pub months_to_recovery: Option<u32>,
    /// Ending value after the scenario window
    pub ending_cents: i64,
    /// Did the portfolio run out of money (hit 0) due to burn rate?
    pub survived: bool,
}

#[derive(Debug, Clone)]
pub struct HistoricalScenario {
    pub name: String,
    /// Sequence of monthly returns (e.g., -0.05 for -5% drop that month)
    pub monthly_returns: Vec<f64>,
}

impl HistoricalScenario {
    #[must_use]
    pub fn dot_com_bubble() -> Self {
        Self {
            name: "Dot Com Bubble (2000-2002)".to_string(),
            monthly_returns: vec![
                -0.03, -0.05, 0.02, -0.04, -0.01, 0.01, -0.06, 0.02, -0.08, -0.02, -0.05, 0.01,
                0.01, -0.07, -0.04, 0.03, 0.01, -0.02, -0.03, -0.06, -0.09, 0.02, 0.04, 0.01,
                -0.01, -0.02, 0.03, -0.05, -0.01, -0.07, -0.08, 0.01, -0.10, 0.02, 0.04, -0.04,
                0.02, 0.01, 0.03, 0.05, 0.04, 0.01, 0.02, 0.03, -0.01, 0.04, 0.02, 0.03,
            ],
        }
    }

    #[must_use]
    pub fn global_financial_crisis() -> Self {
        Self {
            name: "Global Financial Crisis (2007-2009)".to_string(),
            monthly_returns: vec![
                0.01, -0.02, -0.04, 0.01, -0.01, -0.06, -0.03, -0.01, 0.04, 0.01, -0.08, -0.01,
                0.01, -0.09, -0.16, -0.07, 0.01, -0.08, -0.10, 0.08, 0.09, 0.05, 0.01, 0.07, 0.03,
                0.04, -0.02, 0.06, 0.02, -0.03, 0.03, 0.06, 0.01, -0.08, -0.05, 0.07, -0.04, 0.08,
                0.03, 0.01, 0.05,
            ],
        }
    }

    #[must_use]
    pub fn covid_crash() -> Self {
        Self {
            name: "COVID-19 Crash (2020)".to_string(),
            monthly_returns: vec![
                0.01, -0.08, -0.12, 0.12, 0.04, 0.02, 0.05, 0.07, -0.04, -0.02, 0.10, 0.03, 0.01,
                0.02, 0.04, 0.05, 0.01, 0.02, 0.02, 0.03, -0.04, 0.06, -0.01, 0.04,
            ],
        }
    }
}

pub struct StressTester {
    starting_cents: i64,
    monthly_burn_cents: i64,
}

impl StressTester {
    #[must_use]
    pub const fn new(starting_cents: i64, monthly_burn_cents: i64) -> Self {
        Self {
            starting_cents,
            monthly_burn_cents,
        }
    }

    #[must_use]
    pub fn run_scenario(&self, scenario: &HistoricalScenario) -> StressTestResult {
        let mut current_cents = self.starting_cents;
        let mut trough_cents = current_cents;
        let mut months_to_recovery = None;
        let mut survived = true;

        for (month, &return_rate) in scenario.monthly_returns.iter().enumerate() {
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let gain_loss = (current_cents as f64 * return_rate).round() as i64;

            current_cents = current_cents.saturating_add(gain_loss);
            current_cents = current_cents.saturating_sub(self.monthly_burn_cents);

            if current_cents <= 0 {
                current_cents = 0;
                survived = false;
                trough_cents = 0;
                break;
            }

            if current_cents < trough_cents {
                trough_cents = current_cents;
            }

            if months_to_recovery.is_none() && current_cents >= self.starting_cents {
                #[allow(clippy::cast_possible_truncation)]
                let month_u32 = (month + 1) as u32;
                months_to_recovery = Some(month_u32);
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let starting_f = self.starting_cents as f64;
        #[allow(clippy::cast_precision_loss)]
        let trough_f = trough_cents as f64;

        let max_drawdown_pct = if self.starting_cents > 0 {
            ((starting_f - trough_f) / starting_f) * 100.0
        } else {
            0.0
        };

        StressTestResult {
            scenario_name: scenario.name.clone(),
            starting_cents: self.starting_cents,
            trough_cents,
            max_drawdown_pct,
            months_to_recovery,
            ending_cents: current_cents,
            survived,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covid_crash_survival() {
        let tester = StressTester::new(100_000_000, 500_000); // $1M portfolio, $5k burn
        let scenario = HistoricalScenario::covid_crash();
        let result = tester.run_scenario(&scenario);

        assert!(result.survived);
        assert!(result.max_drawdown_pct > 10.0);
        assert!(result.ending_cents > 80_000_000);
    }

    #[test]
    fn test_gfc_wipeout() {
        let tester = StressTester::new(100_000_000, 5_000_000); // $1M portfolio, $50k burn (too high)
        let scenario = HistoricalScenario::global_financial_crisis();
        let result = tester.run_scenario(&scenario);

        assert!(!result.survived);
        assert_eq!(result.ending_cents, 0);
        assert_eq!(result.trough_cents, 0);
    }
}
