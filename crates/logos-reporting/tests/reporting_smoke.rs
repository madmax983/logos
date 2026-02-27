use logos_reporting::{
    RegisterEntry, project_budget_variance, project_cashflow, project_net_worth,
    project_register_balance, project_register_balance_iter, project_rsu_forecast_summary,
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
