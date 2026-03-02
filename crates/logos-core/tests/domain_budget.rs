use logos_core::domain::budget::BudgetMonth;

#[test]
fn test_budget_month_accessors() {
    let month = BudgetMonth::new("2024-05", 1000, 500, 200);

    assert_eq!(month.month_key(), "2024-05");
    assert_eq!(month.start_balance(), 1000);
    assert_eq!(month.assigned(), 500);
    assert_eq!(month.spent(), 200);
    assert_eq!(month.end_balance(), 1300);
}
