#![allow(clippy::should_panic_without_expect)]
use logos_core::cashflow_projector::{CashflowProjector, RecurringTemplate};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_balances_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", amount);

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: amount,
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        let _ = projector.project_balances(2);
    }
}

#[cfg(feature = "nova")]
proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_subscription_fatigue_panics_on_overflow(
        amount1 in (i64::MAX / 2 + 1)..=i64::MAX,
        amount2 in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let analyzer = logos_core::subscription_fatigue::SubscriptionFatigueAnalyzer::new(1, 7.0, 10);
        let mut transactions = Vec::new();

        let tx1 = logos_core::TransactionBuilder::new("Sub1")
            .posting(logos_core::Posting::credit(logos_core::AccountId::new("assets:checking").unwrap(), amount1).unwrap())
            .posting(logos_core::Posting::debit(logos_core::AccountId::new("expenses:entertainment").unwrap(), amount1).unwrap())
            .build()
            .unwrap();
        transactions.push(tx1);

        let tx2 = logos_core::TransactionBuilder::new("Sub2")
            .posting(logos_core::Posting::credit(logos_core::AccountId::new("assets:checking").unwrap(), amount2).unwrap())
            .posting(logos_core::Posting::debit(logos_core::AccountId::new("expenses:health").unwrap(), amount2).unwrap())
            .build()
            .unwrap();
        transactions.push(tx2);

        let _ = analyzer.analyze(&transactions);
    }
}

#[cfg(feature = "nova")]
proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_category_trends_panics_on_overflow(
        amount1 in (i64::MAX / 2 + 1)..=i64::MAX,
        amount2 in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut analyzer = logos_core::category_trends::CategoryTrendAnalyzer::new();
        let rent_acc = logos_core::AccountId::new("expenses:rent").unwrap();
        let checking_acc = logos_core::AccountId::new("assets:checking").unwrap();
        let housing_group = logos_core::CategoryGroupId::from_name("Housing").unwrap();

        analyzer.map_account(rent_acc.clone(), housing_group);

        let tx1 = logos_core::TransactionBuilder::new("Rent 1")
            .posting(logos_core::Posting::debit(rent_acc.clone(), amount1).unwrap())
            .posting(logos_core::Posting::credit(checking_acc.clone(), amount1).unwrap())
            .build()
            .unwrap();

        let tx2 = logos_core::TransactionBuilder::new("Rent 2")
            .posting(logos_core::Posting::debit(rent_acc, amount2).unwrap())
            .posting(logos_core::Posting::credit(checking_acc, amount2).unwrap())
            .build()
            .unwrap();

        let _ = analyzer.compute_spending_by_category(&[tx1, tx2]);
    }
}
