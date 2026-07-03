#![cfg(feature = "nova")]

//! FIRE Burn Rate Anomaly Impact
//!
//! Connects the `AnomalyDetector` with the `FireSimulator` to quantify how much
//! "damage" a statistically large purchase does to your long-term FIRE goals if
//! it represents permanent lifestyle inflation.

use crate::domain::transaction::Transaction;
use crate::experimental::anomaly_detector::{Anomaly, AnomalyDetector};
use crate::planning::fire::FireSimulator;

/// The impact of a single anomaly on the FIRE target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireAnomalyImpact {
    /// The anomaly that was detected.
    pub anomaly: Anomaly,
    /// The original FIRE number in cents before this anomaly.
    pub original_fire_number_cents: i64,
    /// The new FIRE number in cents if this anomaly is added to monthly expenses.
    pub new_fire_number_cents: i64,
    /// The absolute "damage" (increase) to the FIRE number in cents.
    pub fire_damage_cents: i64,
}

/// A report detailing the total impact of all anomalies on the FIRE target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireAnomalyReport {
    /// List of impacts for each detected anomaly.
    pub impacts: Vec<FireAnomalyImpact>,
    /// The sum of all individual FIRE damages.
    pub total_fire_damage_cents: i64,
}

/// Analyzer that calculates the long-term impact of unusually large transactions on FIRE goals.
#[derive(Debug, Clone)]
pub struct FireAnomalyImpactAnalyzer {
    anomaly_detector: AnomalyDetector,
}

impl FireAnomalyImpactAnalyzer {
    /// Creates a new `FireAnomalyImpactAnalyzer`.
    ///
    /// # Arguments
    /// * `iqr_multiplier` - Multiplier for the IQR anomaly detection (e.g., 1.5).
    #[must_use]
    pub const fn new(iqr_multiplier: f64) -> Self {
        Self {
            anomaly_detector: AnomalyDetector::new(iqr_multiplier),
        }
    }

    /// Analyzes transactions for anomalies and calculates their impact on the given FIRE simulator's goals.
    #[must_use]
    pub fn analyze(
        &self,
        transactions: &[Transaction],
        base_sim: &FireSimulator,
    ) -> FireAnomalyReport {
        let anomalies = self.anomaly_detector.detect(transactions);
        let original_fire_number_cents = base_sim.fire_number_cents();

        let mut impacts = Vec::new();
        let mut total_fire_damage_cents: i64 = 0;

        for anomaly in anomalies {
            // Create a copy of the sim and increase its monthly burn rate by the anomaly amount
            let new_expenses = base_sim
                .monthly_expenses_cents()
                .saturating_add(anomaly.amount_cents);
            let mut adjusted_sim = FireSimulator::new(new_expenses);
            adjusted_sim.set_config(base_sim.config());

            let new_fire_number_cents = adjusted_sim.fire_number_cents();

            let fire_damage_cents =
                if new_fire_number_cents != i64::MAX && original_fire_number_cents != i64::MAX {
                    new_fire_number_cents.saturating_sub(original_fire_number_cents)
                } else {
                    i64::MAX
                };

            total_fire_damage_cents = total_fire_damage_cents.saturating_add(fire_damage_cents);

            impacts.push(FireAnomalyImpact {
                anomaly,
                original_fire_number_cents,
                new_fire_number_cents,
                fire_damage_cents,
            });
        }

        FireAnomalyReport {
            impacts,
            total_fire_damage_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_fire_anomaly_impact_analysis() {
        let analyzer = FireAnomalyImpactAnalyzer::new(1.5);
        let base_sim = FireSimulator::new(500_000); // 5k/mo expenses

        let mut transactions = Vec::new();

        // 5 Normal amounts around $10
        let amounts = vec![1000, 1200, 1400, 1500, 2000];
        for amount in amounts {
            let tx = TransactionBuilder::new("Lunch")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap(),
                )
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), amount).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 1 Anomalous amount: $35 (anomaly threshold is ~27)
        let tx = TransactionBuilder::new("Fancy Dinner")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 3500).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 3500).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        let report = analyzer.analyze(&transactions, &base_sim);

        assert_eq!(report.impacts.len(), 1);
        let impact = &report.impacts[0];

        assert_eq!(impact.anomaly.description, "Fancy Dinner");
        assert_eq!(impact.anomaly.amount_cents, 3500);

        // Original FIRE: $5k/mo = $60k/yr @ 4% = $1.5M
        assert_eq!(impact.original_fire_number_cents, 150_000_000);

        // New FIRE: $5035/mo = $60,420/yr @ 4% = $1,510,500
        assert_eq!(impact.new_fire_number_cents, 151_050_000);

        // Damage is $10,500
        assert_eq!(impact.fire_damage_cents, 1_050_000);
        assert_eq!(report.total_fire_damage_cents, 1_050_000);
    }
}
