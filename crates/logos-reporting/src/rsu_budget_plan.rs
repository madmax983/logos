use logos_core::{forecast_value_cents, HaircutTierTable};

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

    let tiers = HaircutTierTable::conservative_defaults();
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
