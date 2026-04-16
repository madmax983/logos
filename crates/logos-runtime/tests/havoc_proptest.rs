use logos_runtime::runtime::AppRuntime;
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

        let month_key = logos_runtime::AppRuntime::<logos_store::memory::MemoryStore>::current_month_key_local();

        let _ = runtime.month_report_for("assets:checking", &month_key);
    }
}
