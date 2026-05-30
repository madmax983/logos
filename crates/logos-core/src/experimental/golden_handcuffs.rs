#![cfg(feature = "nova")]

//! Golden Handcuffs Analyzer
//!
//! For tech workers, unvested RSUs often act as "golden handcuffs", making it
//! financially difficult to walk away from a job. This module quantifies the
//! exact cost of quitting today versus staying, calculating the "Daily Retention Premium"
//! (how much you are effectively paid per day to endure the job until the next vest).
//!
//! 🌟 Nova Mashup: We combine upcoming RSU vests with base salary to determine
//! the true cost of quitting and the financial density of your upcoming time.

use crate::planning::fire::UpcomingVest;

/// A report detailing the financial implications of leaving unvested RSUs behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldenHandcuffsReport {
    /// Total value of all unvested RSUs left on the table if you quit today.
    pub total_unvested_value_cents: i64,
    /// The value of the very next vest.
    pub next_vest_value_cents: i64,
    /// Days until the next vest.
    pub days_to_next_vest: u16,
    /// How much you are effectively earning per day from RSUs alone until the next vest.
    /// (Next vest value / days to next vest)
    pub daily_retention_premium_cents: i64,
    /// Your total daily earning rate (Base salary daily + RSU retention premium).
    pub total_daily_compensation_cents: i64,
}

/// Analyzes the financial impact of unvested equity.
#[derive(Debug, Clone)]
pub struct GoldenHandcuffsAnalyzer {
    annual_base_salary_cents: i64,
    unvested_grants: Vec<UpcomingVest>,
}

impl GoldenHandcuffsAnalyzer {
    /// Creates a new `GoldenHandcuffsAnalyzer`.
    #[must_use]
    pub const fn new(annual_base_salary_cents: i64, unvested_grants: Vec<UpcomingVest>) -> Self {
        Self {
            annual_base_salary_cents,
            unvested_grants,
        }
    }

    /// Analyzes the golden handcuffs to produce a compensation report.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn analyze(&self) -> GoldenHandcuffsReport {
        let daily_base_cents = self.annual_base_salary_cents / 365;

        if self.unvested_grants.is_empty() {
            return GoldenHandcuffsReport {
                total_unvested_value_cents: 0,
                next_vest_value_cents: 0,
                days_to_next_vest: 0,
                daily_retention_premium_cents: 0,
                total_daily_compensation_cents: daily_base_cents,
            };
        }

        let mut total_unvested: i64 = 0;
        let mut next_vest: Option<&UpcomingVest> = None;

        for vest in &self.unvested_grants {
            let vest_value = i64::from(vest.units).saturating_mul(vest.avg_close_price_cents);
            total_unvested = total_unvested.saturating_add(vest_value);

            if let Some(nv) = next_vest {
                if vest.days_to_vest < nv.days_to_vest {
                    next_vest = Some(vest);
                }
            } else {
                next_vest = Some(vest);
            }
        }

        let next_vest = next_vest.expect("Should have at least one vest");
        let next_vest_value_cents =
            i64::from(next_vest.units).saturating_mul(next_vest.avg_close_price_cents);

        let daily_retention_premium_cents = if next_vest.days_to_vest > 0 {
            next_vest_value_cents / i64::from(next_vest.days_to_vest)
        } else {
            next_vest_value_cents // If it vests today, all of it is today's premium
        };

        GoldenHandcuffsReport {
            total_unvested_value_cents: total_unvested,
            next_vest_value_cents,
            days_to_next_vest: next_vest.days_to_vest,
            daily_retention_premium_cents,
            total_daily_compensation_cents: daily_base_cents
                .saturating_add(daily_retention_premium_cents),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_unvested_equity() {
        let analyzer = GoldenHandcuffsAnalyzer::new(10_000_000, vec![]);
        let report = analyzer.analyze();

        assert_eq!(report.total_unvested_value_cents, 0);
        assert_eq!(report.daily_retention_premium_cents, 0);
        assert_eq!(report.total_daily_compensation_cents, 10_000_000 / 365);
    }

    #[test]
    fn test_golden_handcuffs_analysis() {
        // Base salary: $150k / year
        // Vest 1: 100 units @ $150 = $15k in 30 days
        // Vest 2: 200 units @ $150 = $30k in 120 days
        let analyzer = GoldenHandcuffsAnalyzer::new(
            15_000_000, // $150,000
            vec![
                UpcomingVest {
                    avg_close_price_cents: 15_000,
                    units: 100,
                    days_to_vest: 30,
                },
                UpcomingVest {
                    avg_close_price_cents: 15_000,
                    units: 200,
                    days_to_vest: 120,
                },
            ],
        );

        let report = analyzer.analyze();

        assert_eq!(report.total_unvested_value_cents, 4_500_000); // $45k total
        assert_eq!(report.next_vest_value_cents, 1_500_000); // $15k next vest
        assert_eq!(report.days_to_next_vest, 30);

        // Daily retention premium = 1_500_000 / 30 = 50_000 ($500/day)
        assert_eq!(report.daily_retention_premium_cents, 50_000);

        // Daily base = 15_000_000 / 365 = 41_095 cents ($410.95/day)
        let expected_daily_base = 15_000_000 / 365;

        assert_eq!(
            report.total_daily_compensation_cents,
            expected_daily_base + 50_000
        );
    }

    #[test]
    fn test_vesting_today() {
        let analyzer = GoldenHandcuffsAnalyzer::new(
            10_000_000,
            vec![UpcomingVest {
                avg_close_price_cents: 10_000,
                units: 50,
                days_to_vest: 0, // Vests today
            }],
        );

        let report = analyzer.analyze();
        assert_eq!(report.daily_retention_premium_cents, 500_000); // Entire amount is today's premium
    }
}
