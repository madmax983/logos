use logos_runtime::AppRuntime;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_month_report_does_not_panic_on_overflow(
        amount in (i64::MAX / 2 + 1)..i64::MAX
    ) {
        let mut runtime = AppRuntime::new_in_memory();

        runtime.post_double_entry(
            "Expense 1",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        runtime.post_double_entry(
            "Expense 2",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        let month_key = logos_runtime::AppRuntime::<logos_store::MemoryStore>::current_month_key_local();

        let _ = runtime.month_report_for("assets:checking", &month_key);
    }
}

proptest! {
    #[test]
    fn havoc_reconcile_month_does_not_panic_on_overflow(
        amount in (i64::MAX / 2 + 1)..i64::MAX
    ) {
        let mut runtime = AppRuntime::new_in_memory();
        let month_key = logos_runtime::AppRuntime::<logos_store::MemoryStore>::current_month_key_local();

        runtime.post_double_entry(
            "Expense 1",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        runtime.post_double_entry(
            "Expense 2",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        let _ = runtime.reconcile_month_for("assets:checking", &month_key, amount, amount);
    }
}

proptest! {
    #[test]
    fn havoc_register_balance_for_does_not_panic(
        amount in (i64::MAX / 2 + 1)..i64::MAX
    ) {
        let mut runtime = AppRuntime::new_in_memory();

        runtime.post_double_entry(
            "Expense 1",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        runtime.post_double_entry(
            "Expense 2",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        let _ = runtime.register_balance_for("assets:checking");
    }
}

proptest! {
    #[test]
    fn havoc_budget_variance_does_not_panic(
        amount in (i64::MAX / 2 + 1)..i64::MAX
    ) {
        let mut runtime = AppRuntime::new_in_memory();
        let month_key = logos_runtime::AppRuntime::<logos_store::MemoryStore>::current_month_key_local();

        runtime.set_budget_target_for_month(&month_key, "expenses:food", amount).unwrap();

        runtime.post_double_entry(
            "Expense 1",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        runtime.post_double_entry(
            "Expense 2",
            "expenses:food",
            "assets:checking",
            amount
        ).unwrap();

        let _ = runtime.budget_variance_for_month(&month_key, amount, "expenses:food");
    }
}
