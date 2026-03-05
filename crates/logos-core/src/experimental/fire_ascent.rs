use std::fmt::Write;

use crate::planning::fire::FireSimulator;
use crate::planning::net_worth_projector::NetWorthProjector;

/// A simulator that visualizes the journey to FIRE as a mountain ascent.
///
/// It combines the `FireSimulator` (to determine the goal) and the
/// `NetWorthProjector` (to calculate the timeline) to produce a textual
/// dashboard mapping financial progress to milestones on a mountain.
#[derive(Debug, Clone)]
pub struct FireAscentSimulator {
    fire_sim: FireSimulator,
    projector: NetWorthProjector,
    max_months: u16,
}

impl FireAscentSimulator {
    /// Creates a new `FireAscentSimulator` from the base simulators.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        projector: NetWorthProjector,
        max_months: u16,
    ) -> Self {
        Self {
            fire_sim,
            projector,
            max_months,
        }
    }

    /// Simulates the ascent and returns a textual log of the journey.
    #[must_use]
    pub fn ascend(&self) -> String {
        let fire_number = self.fire_sim.fire_number_cents();

        if fire_number == 0 {
            return String::from("Summit reached instantly: Expenses are zero!\n");
        }
        if fire_number == i64::MAX {
            return String::from("The Summit is infinite (Safe Withdrawal Rate is 0%). The ascent is impossible.\n");
        }

        // Add milestones for the ascent (25%, 50%, 75%, 100%)
        let mut ascent_projector = self.projector.clone();

        let camp1 = fire_number / 4;
        let camp2 = fire_number / 2;
        let camp3 = (fire_number * 3) / 4;
        let summit = fire_number;

        ascent_projector.add_milestone_cents(camp1);
        ascent_projector.add_milestone_cents(camp2);
        ascent_projector.add_milestone_cents(camp3);
        ascent_projector.add_milestone_cents(summit);

        let (timeline, crossed_milestones) = ascent_projector.project_timeline(self.max_months);

        let mut output = String::new();
        let _ = writeln!(
            output,
            "🏔️  FIRE Ascent Simulation  🏔️\nTarget Summit: ${:.2}",
            summit as f64 / 100.0
        );
        let _ = writeln!(output, "Maximum Duration: {} months\n", self.max_months);

        let mut reached_summit = false;
        let mut summit_month = 0;

        // Process milestones in order of achievement
        let mut sorted_milestones = crossed_milestones;
        sorted_milestones.sort_by_key(|&(_, month)| month);

        let milestone_names = [
            (camp1, "⛺ Camp 1 (25%)"),
            (camp2, "⛺ Camp 2 (50%)"),
            (camp3, "⛺ Camp 3 (75%)"),
            (summit, "🚩 SUMMIT (100%)"),
        ];

        // Let's refine the loop to just iterate through our predefined milestones and check if they were crossed
        for (target_cents, target_name) in milestone_names {
            if let Some(&(_, month)) = sorted_milestones.iter().find(|&&(c, _)| c == target_cents) {
                let _ = writeln!(
                    output,
                    "[{:^10}] Reached {} at ${:.2}",
                    format!("Month {}", month),
                    target_name,
                    target_cents as f64 / 100.0
                );

                if target_cents == summit {
                    reached_summit = true;
                    summit_month = month;
                }
            } else {
                let _ = writeln!(
                    output,
                    "[  PENDING ] {} at ${:.2} remains unreached.",
                    target_name,
                    target_cents as f64 / 100.0
                );
            }
        }

        let _ = writeln!(output);

        if reached_summit {
            let years = summit_month / 12;
            let months = summit_month % 12;
            let _ = writeln!(
                output,
                "🎉 Ascent Successful! Summit reached in {} years and {} months.",
                years, months
            );
        } else {
            let final_nw = timeline.last().map(|m| m.net_worth_cents).unwrap_or(0);
            #[allow(clippy::cast_precision_loss)]
            let progress_pct = (final_nw as f64 / summit as f64) * 100.0;
            let _ = writeln!(
                output,
                "⚠️  Expedition halted after {} months.",
                self.max_months
            );
            let _ = writeln!(
                output,
                "Final Net Worth: ${:.2} ({:.1}% of Summit)",
                final_nw as f64 / 100.0,
                progress_pct
            );

            if progress_pct < 0.0 {
                 let _ = writeln!(output, "The mountain is too steep. Consider increasing savings or reducing expenses.");
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::{FireConfig, FireSimulator};
    use crate::planning::net_worth_projector::NetWorthProjector;

    #[test]
    fn test_successful_ascent() {
        let mut fire_sim = FireSimulator::new(400_000); // 4k/mo expenses = 48k/yr
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4, // SWR 4% -> FIRE number 1.2M (120_000_000 cents)
        });

        // Current NW is 800k, saving 10k/mo. Need 400k more.
        // At 10k/mo, it takes 40 months.
        let projector = NetWorthProjector::new(80_000_000, 1_000_000);

        let ascent_sim = FireAscentSimulator::new(fire_sim, projector, 60);
        let log = ascent_sim.ascend();

        assert!(log.contains("Target Summit: $1200000.00"));
        assert!(log.contains("Reached ⛺ Camp 1 (25%) at $300000.00")); // Because we start at 800k, this is instantly crossed in simulation? Wait, the projector will only say it's crossed if it crosses it *during* the projection.
        // Actually, projector checks if `current_net_worth >= milestone`. If it starts >= milestone, it will cross it in month 1.
        assert!(log.contains("Reached 🚩 SUMMIT (100%) at $1200000.00"));
        assert!(log.contains("Ascent Successful!"));
    }

    #[test]
    fn test_failed_ascent() {
        let mut fire_sim = FireSimulator::new(500_000); // 5k/mo expenses = 60k/yr
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4, // SWR 4% -> FIRE number 1.5M
        });

        // Current NW is 100k, saving 1k/mo.
        let projector = NetWorthProjector::new(10_000_000, 100_000);

        // Simulate for only 12 months.
        let ascent_sim = FireAscentSimulator::new(fire_sim, projector, 12);
        let log = ascent_sim.ascend();

        assert!(log.contains("Expedition halted after 12 months."));
        assert!(log.contains("Final Net Worth: $112000.00"));
        assert!(log.contains("[  PENDING ] 🚩 SUMMIT (100%) at $1500000.00 remains unreached."));
    }

    #[test]
    fn test_zero_expenses_instant_summit() {
        let fire_sim = FireSimulator::new(0); // 0 expenses -> 0 FIRE number
        let projector = NetWorthProjector::new(0, 0);

        let ascent_sim = FireAscentSimulator::new(fire_sim, projector, 12);
        let log = ascent_sim.ascend();

        assert_eq!(log, "Summit reached instantly: Expenses are zero!\n");
    }
}
