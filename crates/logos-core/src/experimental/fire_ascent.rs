use crate::planning::fire::FireSimulator;
use crate::planning::net_worth_projector::NetWorthProjector;

/// Represents a milestone in the journey to FIRE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AscentMilestone {
    pub name: &'static str,
    pub target_cents: i64,
    pub month_reached: Option<u16>,
}

/// The result of simulating a FIRE ascent.
#[derive(Debug, Clone, PartialEq)]
pub struct AscentResult {
    pub summit_cents: i64,
    pub max_months: u16,
    pub final_net_worth_cents: i64,
    pub milestones: Vec<AscentMilestone>,
    pub success: bool,
    pub instant_summit: bool,
    pub impossible: bool,
}

/// A simulator that visualizes the journey to FIRE as a mountain ascent.
///
/// It combines the `FireSimulator` (to determine the goal) and the
/// `NetWorthProjector` (to calculate the timeline) to produce milestones.
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

    /// Simulates the ascent and returns the milestones reached and final outcome.
    #[must_use]
    pub fn ascend(&self) -> AscentResult {
        let fire_number = self.fire_sim.fire_number_cents();

        if fire_number == 0 {
            return AscentResult {
                summit_cents: 0,
                max_months: self.max_months,
                final_net_worth_cents: 0,
                milestones: Vec::new(),
                success: true,
                instant_summit: true,
                impossible: false,
            };
        }
        if fire_number == i64::MAX {
            return AscentResult {
                summit_cents: i64::MAX,
                max_months: self.max_months,
                final_net_worth_cents: 0,
                milestones: Vec::new(),
                success: false,
                instant_summit: false,
                impossible: true,
            };
        }

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

        let mut sorted_milestones = crossed_milestones;
        sorted_milestones.sort_by_key(|&(_, month)| month);

        let milestone_names = [
            (camp1, "⛺ Camp 1 (25%)"),
            (camp2, "⛺ Camp 2 (50%)"),
            (camp3, "⛺ Camp 3 (75%)"),
            (summit, "🚩 SUMMIT (100%)"),
        ];

        let mut milestones = Vec::new();
        let mut success = false;

        for (target_cents, target_name) in milestone_names {
            let month_reached = sorted_milestones
                .iter()
                .find(|&&(c, _)| c == target_cents)
                .map(|&(_, m)| m);
            if let Some(_) = month_reached {
                if target_cents == summit {
                    success = true;
                }
            }
            milestones.push(AscentMilestone {
                name: target_name,
                target_cents,
                month_reached,
            });
        }

        let final_net_worth_cents = timeline.last().map_or(0, |m| m.net_worth_cents);

        AscentResult {
            summit_cents: summit,
            max_months: self.max_months,
            final_net_worth_cents,
            milestones,
            success,
            instant_summit: false,
            impossible: false,
        }
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
        let result = ascent_sim.ascend();

        assert_eq!(result.summit_cents, 120_000_000);
        assert_eq!(result.milestones.len(), 4);
        assert!(result.milestones[0].month_reached.is_some());
        assert!(result.milestones[3].month_reached.is_some());
        assert!(result.success);
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
        let result = ascent_sim.ascend();

        assert!(!result.success);
        assert_eq!(result.final_net_worth_cents, 11_200_000);
        assert!(result.milestones[3].month_reached.is_none());
    }

    #[test]
    fn test_zero_expenses_instant_summit() {
        let fire_sim = FireSimulator::new(0); // 0 expenses -> 0 FIRE number
        let projector = NetWorthProjector::new(0, 0);

        let ascent_sim = FireAscentSimulator::new(fire_sim, projector, 12);
        let result = ascent_sim.ascend();

        assert!(result.instant_summit);
        assert!(result.success);
    }
}
