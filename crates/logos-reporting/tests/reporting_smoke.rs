use logos_reporting::{
    project_budget_variance, project_cashflow, project_net_worth, project_register_balance,
    project_register_balance_iter, project_rsu_budget_plan, project_rsu_forecast_summary,
    RegisterEntry, RsuBudgetPlanInput, ScenarioKey, ScenarioPriceInputs,
};

#[test]
fn reporting_projections_are_numerically_correct() {
    assert_eq!(project_net_worth(500_000, 120_000), 380_000);
    assert_eq!(project_cashflow(300_000, 180_000), 120_000);
    assert_eq!(project_budget_variance(90_000, 70_000), 20_000);

    let entries = vec![RegisterEntry::new(50_000), RegisterEntry::new(-20_000)];
    assert_eq!(project_register_balance(100_000, &entries), 130_000);
    assert_eq!(
        project_register_balance_iter(100_000, entries.iter().copied()),
        130_000
    );

    let rsu = project_rsu_forecast_summary(&[75_000, 125_000]);
    assert_eq!(rsu.event_count(), 2);
    assert_eq!(rsu.projected_total_cents(), 200_000);
}

#[test]
fn rsu_budget_plan_uses_bear_case_as_budget_and_sweeps_surplus() {
    let plan = project_rsu_budget_plan(
        "2026-03",
        &RsuBudgetPlanInput::new(
            300,
            45,
            ScenarioPriceInputs::new(10_000, 12_000, 16_000).expect("price scenarios"),
            250_000,
            60,
            30,
        )
        .expect("valid input"),
    )
    .expect("plan");

    assert_eq!(plan.month_key(), "2026-03");
    assert_eq!(
        plan.conservative_budget_cents(),
        plan.scenario(ScenarioKey::Bear)
            .expect("bear scenario exists")
            .monthly_income_cents()
    );
    assert!(
        plan.scenario(ScenarioKey::Base)
            .expect("base scenario")
            .reserve_sweep_cents()
            > 0
    );
    assert!(
        plan.scenario(ScenarioKey::Bull)
            .expect("bull scenario")
            .investing_sweep_cents()
            > 0
    );
}
