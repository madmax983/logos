use std::collections::HashMap;

use crate::domain::category::CategoryGroupId;

/// Evaluates the subjective "Return on Investment" (ROI) in terms of Joy.
///
/// 🌟 Nova Mashup: We combine raw spending metrics with subjective human emotion
/// (Joy Scores 0-10) to determine if your money is actually buying happiness.
#[derive(Debug, Default, Clone)]
pub struct JoyRoiEvaluator {
    category_joy_scores: HashMap<CategoryGroupId, u8>,
}

/// A report detailing the emotional efficiency of your spending.
#[derive(Debug, Clone, PartialEq)]
pub struct JoyReport {
    /// The weighted average joy score across all categorized spending (0.0 - 10.0)
    pub overall_efficiency: f64,
    /// Total cents spent across categorized accounts
    pub total_categorized_spent_cents: i64,
    /// The category that had the most spending despite a low joy score (< 5)
    pub biggest_joy_drain: Option<CategoryGroupId>,
    /// The category that generated the most absolute joy points
    pub highest_joy_generator: Option<CategoryGroupId>,
}

impl JoyRoiEvaluator {
    /// Creates a new, empty Joy ROI Evaluator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a subjective Joy Score (0-10) to a specific category group.
    /// 0 = Miserable necessity, 10 = Pure bliss.
    pub fn assign_score(&mut self, group: CategoryGroupId, score: u8) {
        let clamped_score = score.clamp(0, 10);
        self.category_joy_scores.insert(group, clamped_score);
    }

    /// Evaluates a spending trend report to calculate Joy ROI.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn evaluate(&self, spending_by_category: &HashMap<CategoryGroupId, i64>) -> JoyReport {
        let mut total_spent = 0;
        let mut total_joy_points = 0.0;

        let mut max_drain_spent = -1;
        let mut biggest_joy_drain = None;

        let mut max_generated_joy = -1.0;
        let mut highest_joy_generator = None;

        for (group, &spent_cents) in spending_by_category {
            if spent_cents <= 0 {
                continue;
            }

            if let Some(&score) = self.category_joy_scores.get(group) {
                total_spent += spent_cents;

                // Points = (Dollars Spent) * JoyScore
                let dollars = spent_cents as f64 / 100.0;
                let joy_generated = dollars * f64::from(score);

                total_joy_points += joy_generated;

                // Check for biggest drain (score < 5, highest spend)
                if score < 5 && spent_cents > max_drain_spent {
                    max_drain_spent = spent_cents;
                    biggest_joy_drain = Some(group.clone());
                }

                // Check for highest joy generator
                if joy_generated > max_generated_joy {
                    max_generated_joy = joy_generated;
                    highest_joy_generator = Some(group.clone());
                }
            }
        }

        let overall_efficiency = if total_spent > 0 {
            // (Total Joy Points / Total Dollars Spent)
            total_joy_points / (total_spent as f64 / 100.0)
        } else {
            0.0
        };

        JoyReport {
            overall_efficiency,
            total_categorized_spent_cents: total_spent,
            biggest_joy_drain,
            highest_joy_generator,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joy_roi_evaluation() {
        let mut evaluator = JoyRoiEvaluator::new();

        let dining = CategoryGroupId::from_name("Dining").unwrap();
        let commute = CategoryGroupId::from_name("Commute").unwrap();
        let hobbies = CategoryGroupId::from_name("Hobbies").unwrap();

        // Dining brings some joy (7)
        evaluator.assign_score(dining.clone(), 7);
        // Commute is miserable (2)
        evaluator.assign_score(commute.clone(), 2);
        // Hobbies are amazing (10)
        evaluator.assign_score(hobbies.clone(), 10);

        let mut spending = HashMap::new();
        spending.insert(dining.clone(), 200_00); // $200
        spending.insert(commute.clone(), 300_00); // $300
        spending.insert(hobbies.clone(), 100_00); // $100

        // Total spent = $600
        // Joy points:
        // Dining: 200 * 7 = 1400
        // Commute: 300 * 2 = 600
        // Hobbies: 100 * 10 = 1000
        // Total joy points = 3000
        // Overall efficiency = 3000 / 600 = 5.0

        let report = evaluator.evaluate(&spending);

        assert_eq!(report.total_categorized_spent_cents, 600_00);
        assert!((report.overall_efficiency - 5.0).abs() < f64::EPSILON);

        // Commute is < 5 and has highest spend ($300)
        assert_eq!(report.biggest_joy_drain, Some(commute));
        // Dining has highest joy points (1400)
        assert_eq!(report.highest_joy_generator, Some(dining));
    }

    #[test]
    fn test_empty_evaluation() {
        let evaluator = JoyRoiEvaluator::new();
        let spending = HashMap::new();

        let report = evaluator.evaluate(&spending);
        assert_eq!(report.total_categorized_spent_cents, 0);
        assert_eq!(report.overall_efficiency, 0.0);
        assert_eq!(report.biggest_joy_drain, None);
        assert_eq!(report.highest_joy_generator, None);
    }

    #[test]
    fn test_clamp_score() {
        let mut evaluator = JoyRoiEvaluator::new();
        let group = CategoryGroupId::from_name("Test").unwrap();

        evaluator.assign_score(group.clone(), 255); // Should clamp to 10

        let mut spending = HashMap::new();
        spending.insert(group, 100_00); // $100

        let report = evaluator.evaluate(&spending);
        // 100 * 10 = 1000 points. 1000 / 100 = 10.0 efficiency.
        assert!((report.overall_efficiency - 10.0).abs() < f64::EPSILON);
    }
}
