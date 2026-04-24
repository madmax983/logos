use logos_core::{BudgetMonth, rollover_end_balance};

#[test]
fn test_budget_month() {
    let b = BudgetMonth::new("2026-03", 100, 50, 20);
    assert_eq!(b.month_key(), "2026-03");
    assert_eq!(b.start_balance(), 100);
    assert_eq!(b.assigned(), 50);
    assert_eq!(b.spent(), 20);
    assert_eq!(b.end_balance(), 130);
}

#[test]
fn test_rollover_clamping() {
    assert_eq!(rollover_end_balance(i64::MAX, 10, 0), i64::MAX);
    assert_eq!(rollover_end_balance(i64::MIN, 0, 10), i64::MIN);
}
