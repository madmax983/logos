use logos_core::{HaircutTierTable, forecast_value_cents};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioKey {
    Bear,
    Base,
    Bull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioPriceInputs {
    bear: i64,
    base: i64,
    bull: i64,
}

impl ScenarioPriceInputs {
    /// Creates a monotonic bear/base/bull scenario price set.
    ///
    /// # Errors
    ///
    /// Returns an error when any price is non-positive or when
    /// `bear_price_cents <= base_price_cents <= bull_price_cents` is violated.
    pub fn new(
        bear_price_cents: i64,
        base_price_cents: i64,
        bull_price_cents: i64,
    ) -> Result<Self, String> {
        if bear_price_cents <= 0 || base_price_cents <= 0 || bull_price_cents <= 0 {
            return Err("scenario prices must be positive".to_owned());
        }
        if !(bear_price_cents <= base_price_cents && base_price_cents <= bull_price_cents) {
            return Err("scenario prices must satisfy bear <= base <= bull".to_owned());
        }
        Ok(Self {
            bear: bear_price_cents,
            base: base_price_cents,
            bull: bull_price_cents,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioBudgetProjection {
    scenario: ScenarioKey,
    monthly_income_cents: i64,
    surplus_cents: i64,
    reserve_sweep_cents: i64,
    investing_sweep_cents: i64,
    available_after_sweeps_cents: i64,
}

impl ScenarioBudgetProjection {
    #[must_use]
    pub const fn scenario(&self) -> ScenarioKey {
        self.scenario
    }

    #[must_use]
    pub const fn monthly_income_cents(&self) -> i64 {
        self.monthly_income_cents
    }

    #[must_use]
    pub const fn surplus_cents(&self) -> i64 {
        self.surplus_cents
    }

    #[must_use]
    pub const fn reserve_sweep_cents(&self) -> i64 {
        self.reserve_sweep_cents
    }

    #[must_use]
    pub const fn investing_sweep_cents(&self) -> i64 {
        self.investing_sweep_cents
    }

    #[must_use]
    pub const fn available_after_sweeps_cents(&self) -> i64 {
        self.available_after_sweeps_cents
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsuBudgetPlanInput {
    quarterly_units: u32,
    days_to_vest: u16,
    scenario_prices: ScenarioPriceInputs,
    fixed_commitments_cents: i64,
    reserve_sweep_pct: u8,
    investing_sweep_pct: u8,
}

impl RsuBudgetPlanInput {
    /// Creates a validated RSU budget planning input payload.
    ///
    /// # Errors
    ///
    /// Returns an error when units are zero, fixed commitments are negative, or
    /// reserve/investing sweeps exceed 100% combined.
    pub fn new(
        quarterly_units: u32,
        days_to_vest: u16,
        scenario_prices: ScenarioPriceInputs,
        fixed_commitments_cents: i64,
        reserve_sweep_pct: u8,
        investing_sweep_pct: u8,
    ) -> Result<Self, String> {
        if quarterly_units == 0 {
            return Err("quarterly_units must be greater than zero".to_owned());
        }
        if fixed_commitments_cents < 0 {
            return Err("fixed_commitments_cents must be non-negative".to_owned());
        }
        let sweep_total = u16::from(reserve_sweep_pct) + u16::from(investing_sweep_pct);
        if sweep_total > 100 {
            return Err(format!(
                "reserve_sweep_pct + investing_sweep_pct must be <= 100, got {sweep_total}"
            ));
        }

        Ok(Self {
            quarterly_units,
            days_to_vest,
            scenario_prices,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsuBudgetPlan {
    month_key: String,
    conservative_budget_cents: i64,
    fixed_commitments_cents: i64,
    baseline_remaining_cents: i64,
    reserve_sweep_pct: u8,
    investing_sweep_pct: u8,
    scenarios: [ScenarioBudgetProjection; 3],
}

impl RsuBudgetPlan {
    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub const fn conservative_budget_cents(&self) -> i64 {
        self.conservative_budget_cents
    }

    #[must_use]
    pub const fn fixed_commitments_cents(&self) -> i64 {
        self.fixed_commitments_cents
    }

    #[must_use]
    pub const fn baseline_remaining_cents(&self) -> i64 {
        self.baseline_remaining_cents
    }

    #[must_use]
    pub const fn reserve_sweep_pct(&self) -> u8 {
        self.reserve_sweep_pct
    }

    #[must_use]
    pub const fn investing_sweep_pct(&self) -> u8 {
        self.investing_sweep_pct
    }

    #[must_use]
    pub fn scenario(&self, key: ScenarioKey) -> Option<&ScenarioBudgetProjection> {
        self.scenarios
            .iter()
            .find(|scenario| scenario.scenario() == key)
    }

    #[must_use]
    pub fn bear(&self) -> Option<&ScenarioBudgetProjection> {
        self.scenario(ScenarioKey::Bear)
    }

    #[must_use]
    pub fn base(&self) -> Option<&ScenarioBudgetProjection> {
        self.scenario(ScenarioKey::Base)
    }

    #[must_use]
    pub fn bull(&self) -> Option<&ScenarioBudgetProjection> {
        self.scenario(ScenarioKey::Bull)
    }

    #[must_use]
    pub const fn scenarios(&self) -> &[ScenarioBudgetProjection; 3] {
        &self.scenarios
    }
}

/// Projects a scenario-based RSU monthly budget model that uses bear-case income as baseline.
///
/// # Errors
///
/// Returns an error when the month key is empty or input validation fails.
pub fn project_rsu_budget_plan(
    month_key: &str,
    input: &RsuBudgetPlanInput,
) -> Result<RsuBudgetPlan, String> {
    if month_key.is_empty() {
        return Err("month_key must not be empty".to_owned());
    }

    let tiers = HaircutTierTable::default();
    let bear_monthly = monthly_income(
        input.scenario_prices.bear,
        input.quarterly_units,
        input.days_to_vest,
        tiers,
    );
    let base_monthly = monthly_income(
        input.scenario_prices.base,
        input.quarterly_units,
        input.days_to_vest,
        tiers,
    );
    let bull_monthly = monthly_income(
        input.scenario_prices.bull,
        input.quarterly_units,
        input.days_to_vest,
        tiers,
    );

    let conservative_budget_cents = bear_monthly;
    let baseline_remaining_cents = conservative_budget_cents - input.fixed_commitments_cents;

    Ok(RsuBudgetPlan {
        month_key: month_key.to_owned(),
        conservative_budget_cents,
        fixed_commitments_cents: input.fixed_commitments_cents,
        baseline_remaining_cents,
        reserve_sweep_pct: input.reserve_sweep_pct,
        investing_sweep_pct: input.investing_sweep_pct,
        scenarios: [
            scenario_projection(
                ScenarioKey::Bear,
                bear_monthly,
                conservative_budget_cents,
                input.reserve_sweep_pct,
                input.investing_sweep_pct,
            ),
            scenario_projection(
                ScenarioKey::Base,
                base_monthly,
                conservative_budget_cents,
                input.reserve_sweep_pct,
                input.investing_sweep_pct,
            ),
            scenario_projection(
                ScenarioKey::Bull,
                bull_monthly,
                conservative_budget_cents,
                input.reserve_sweep_pct,
                input.investing_sweep_pct,
            ),
        ],
    })
}

fn monthly_income(price_cents: i64, units: u32, days_to_vest: u16, tiers: HaircutTierTable) -> i64 {
    let quarterly = forecast_value_cents(price_cents, units, days_to_vest, &tiers);
    quarterly / 3
}

fn scenario_projection(
    scenario: ScenarioKey,
    monthly_income_cents: i64,
    conservative_budget_cents: i64,
    reserve_sweep_pct: u8,
    investing_sweep_pct: u8,
) -> ScenarioBudgetProjection {
    let surplus_cents = monthly_income_cents
        .saturating_sub(conservative_budget_cents)
        .max(0);
    let reserve_sweep_cents = surplus_cents.saturating_mul(i64::from(reserve_sweep_pct)) / 100;
    let investing_sweep_cents = surplus_cents.saturating_mul(i64::from(investing_sweep_pct)) / 100;
    let available_after_sweeps_cents = monthly_income_cents
        .saturating_sub(reserve_sweep_cents)
        .saturating_sub(investing_sweep_cents);

    ScenarioBudgetProjection {
        scenario,
        monthly_income_cents,
        surplus_cents,
        reserve_sweep_cents,
        investing_sweep_cents,
        available_after_sweeps_cents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_budget_projection_accessors() {
        let proj = ScenarioBudgetProjection {
            scenario: ScenarioKey::Base,
            monthly_income_cents: 100,
            surplus_cents: 200,
            reserve_sweep_cents: 300,
            investing_sweep_cents: 400,
            available_after_sweeps_cents: 500,
        };
        assert_eq!(proj.scenario(), ScenarioKey::Base);
        assert_eq!(proj.monthly_income_cents(), 100);
        assert_eq!(proj.surplus_cents(), 200);
        assert_eq!(proj.reserve_sweep_cents(), 300);
        assert_eq!(proj.investing_sweep_cents(), 400);
        assert_eq!(proj.available_after_sweeps_cents(), 500);
    }

    #[test]
    fn test_rsu_budget_plan_accessors() {
        let bear_proj = ScenarioBudgetProjection {
            scenario: ScenarioKey::Bear,
            monthly_income_cents: 0,
            surplus_cents: 0,
            reserve_sweep_cents: 0,
            investing_sweep_cents: 0,
            available_after_sweeps_cents: 0,
        };
        let base_proj = ScenarioBudgetProjection {
            scenario: ScenarioKey::Base,
            ..bear_proj
        };
        let bull_proj = ScenarioBudgetProjection {
            scenario: ScenarioKey::Bull,
            ..bear_proj
        };

        let plan = RsuBudgetPlan {
            month_key: "2024-01".to_owned(),
            conservative_budget_cents: 10,
            fixed_commitments_cents: 20,
            baseline_remaining_cents: 30,
            reserve_sweep_pct: 40,
            investing_sweep_pct: 50,
            scenarios: [bear_proj, base_proj, bull_proj],
        };

        assert_eq!(plan.month_key(), "2024-01");
        assert_eq!(plan.conservative_budget_cents(), 10);
        assert_eq!(plan.fixed_commitments_cents(), 20);
        assert_eq!(plan.baseline_remaining_cents(), 30);
        assert_eq!(plan.reserve_sweep_pct(), 40);
        assert_eq!(plan.investing_sweep_pct(), 50);

        assert_eq!(
            plan.scenario(ScenarioKey::Bear)
                .expect("should succeed")
                .scenario(),
            ScenarioKey::Bear
        );
        assert_eq!(
            plan.bear().expect("should succeed").scenario(),
            ScenarioKey::Bear
        );
        assert_eq!(
            plan.base().expect("should succeed").scenario(),
            ScenarioKey::Base
        );
        assert_eq!(
            plan.bull().expect("should succeed").scenario(),
            ScenarioKey::Bull
        );

        let sc = plan.scenarios();
        assert_eq!(sc.len(), 3);
        assert_eq!(sc[0].scenario(), ScenarioKey::Bear);
    }

    #[test]
    fn test_scenario_projection_sweeps() {
        let proj = scenario_projection(
            ScenarioKey::Base,
            1000,
            200, // surplus = 800
            10,  // 10% = 80
            20,  // 20% = 160
        );
        assert_eq!(proj.surplus_cents(), 800);
        assert_eq!(proj.reserve_sweep_cents(), 80);
        assert_eq!(proj.investing_sweep_cents(), 160);
        assert_eq!(proj.available_after_sweeps_cents(), 1000 - 80 - 160); // 760
    }

    #[test]
    fn test_monthly_income_math() {
        let tiers = HaircutTierTable::default();
        let val = monthly_income(1000, 300, 0, tiers);
        assert_eq!(val, 75000);
    }

    #[test]
    fn test_project_rsu_budget_plan() {
        let prices = ScenarioPriceInputs::new(100, 200, 300).expect("should succeed");
        let input = RsuBudgetPlanInput::new(300, 0, prices, 5000, 10, 20).expect("should succeed");

        let plan = project_rsu_budget_plan("2024-01", &input).expect("should succeed");

        assert_eq!(plan.conservative_budget_cents(), 7500);
        assert_eq!(plan.baseline_remaining_cents(), 7500 - 5000); // 2500
    }

    #[test]
    fn test_scenario_price_inputs_validation_errors() {
        assert_eq!(
            ScenarioPriceInputs::new(0, 200, 300).unwrap_err(),
            "scenario prices must be positive"
        );
        assert_eq!(
            ScenarioPriceInputs::new(100, -200, 300).unwrap_err(),
            "scenario prices must be positive"
        );
        assert_eq!(
            ScenarioPriceInputs::new(100, 200, 0).unwrap_err(),
            "scenario prices must be positive"
        );
        assert_eq!(
            ScenarioPriceInputs::new(300, 200, 100).unwrap_err(),
            "scenario prices must satisfy bear <= base <= bull"
        );
        assert_eq!(
            ScenarioPriceInputs::new(100, 300, 200).unwrap_err(),
            "scenario prices must satisfy bear <= base <= bull"
        );
    }

    #[test]
    fn test_rsu_budget_plan_input_validation_errors() {
        let valid_prices = ScenarioPriceInputs::new(100, 200, 300).expect("should succeed");

        assert_eq!(
            RsuBudgetPlanInput::new(0, 10, valid_prices, 5000, 10, 20).unwrap_err(),
            "quarterly_units must be greater than zero"
        );

        assert_eq!(
            RsuBudgetPlanInput::new(100, 10, valid_prices, -5000, 10, 20).unwrap_err(),
            "fixed_commitments_cents must be non-negative"
        );

        assert_eq!(
            RsuBudgetPlanInput::new(100, 10, valid_prices, 5000, 60, 50).unwrap_err(),
            "reserve_sweep_pct + investing_sweep_pct must be <= 100, got 110"
        );
    }
}
