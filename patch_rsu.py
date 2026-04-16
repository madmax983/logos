import re

with open('crates/logos-reporting/src/rsu_budget_plan.rs', 'r') as f:
    content = f.read()

replacement = """
    let tiers = HaircutTierTable::default();
    let scenarios = [
        (ScenarioKey::Bear, input.scenario_prices.bear),
        (ScenarioKey::Base, input.scenario_prices.base),
        (ScenarioKey::Bull, input.scenario_prices.bull),
    ]
    .map(|(key, price)| {
        let monthly = monthly_income(price, input.quarterly_units, input.days_to_vest, tiers);
        (key, monthly)
    });

    let conservative_budget_cents = scenarios[0].1;
    let baseline_remaining_cents =
        conservative_budget_cents.saturating_sub(input.fixed_commitments_cents);

    let scenarios = scenarios.map(|(key, monthly)| {
        scenario_projection(
            key,
            monthly,
            conservative_budget_cents,
            input.reserve_sweep_pct,
            input.investing_sweep_pct,
        )
    });

    Ok(RsuBudgetPlan {
        month_key: month_key.to_owned(),
        conservative_budget_cents,
        fixed_commitments_cents: input.fixed_commitments_cents,
        baseline_remaining_cents,
        reserve_sweep_pct: input.reserve_sweep_pct,
        investing_sweep_pct: input.investing_sweep_pct,
        scenarios,
    })
}
"""

pattern = re.compile(
    r'    let tiers = HaircutTierTable::default\(\);\n'
    r'    let bear_monthly = monthly_income\(\n'
    r'        input\.scenario_prices\.bear,\n'
    r'        input\.quarterly_units,\n'
    r'        input\.days_to_vest,\n'
    r'        tiers,\n'
    r'    \);\n'
    r'    let base_monthly = monthly_income\(\n'
    r'        input\.scenario_prices\.base,\n'
    r'        input\.quarterly_units,\n'
    r'        input\.days_to_vest,\n'
    r'        tiers,\n'
    r'    \);\n'
    r'    let bull_monthly = monthly_income\(\n'
    r'        input\.scenario_prices\.bull,\n'
    r'        input\.quarterly_units,\n'
    r'        input\.days_to_vest,\n'
    r'        tiers,\n'
    r'    \);\n'
    r'\n'
    r'    let conservative_budget_cents = bear_monthly;\n'
    r'    let baseline_remaining_cents =\n'
    r'        conservative_budget_cents\.saturating_sub\(input\.fixed_commitments_cents\);\n'
    r'\n'
    r'    Ok\(RsuBudgetPlan \{\n'
    r'        month_key: month_key\.to_owned\(\),\n'
    r'        conservative_budget_cents,\n'
    r'        fixed_commitments_cents: input\.fixed_commitments_cents,\n'
    r'        baseline_remaining_cents,\n'
    r'        reserve_sweep_pct: input\.reserve_sweep_pct,\n'
    r'        investing_sweep_pct: input\.investing_sweep_pct,\n'
    r'        scenarios: \[\n'
    r'            scenario_projection\(\n'
    r'                ScenarioKey::Bear,\n'
    r'                bear_monthly,\n'
    r'                conservative_budget_cents,\n'
    r'                input\.reserve_sweep_pct,\n'
    r'                input\.investing_sweep_pct,\n'
    r'            \),\n'
    r'            scenario_projection\(\n'
    r'                ScenarioKey::Base,\n'
    r'                base_monthly,\n'
    r'                conservative_budget_cents,\n'
    r'                input\.reserve_sweep_pct,\n'
    r'                input\.investing_sweep_pct,\n'
    r'            \),\n'
    r'            scenario_projection\(\n'
    r'                ScenarioKey::Bull,\n'
    r'                bull_monthly,\n'
    r'                conservative_budget_cents,\n'
    r'                input\.reserve_sweep_pct,\n'
    r'                input\.investing_sweep_pct,\n'
    r'            \),\n'
    r'        \],\n'
    r'    \}\)\n'
    r'\}',
    re.MULTILINE
)

new_content, count = pattern.subn(replacement.strip(), content)

print(f"Replacements made: {count}")

with open('crates/logos-reporting/src/rsu_budget_plan.rs', 'w') as f:
    f.write(new_content)
