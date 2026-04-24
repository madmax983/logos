use logos_core::AccountType;

#[test]
fn test_account_type_normal_balance_sign() {
    assert_eq!(AccountType::Asset.normal_balance_sign(), 1);
    assert_eq!(AccountType::Liability.normal_balance_sign(), -1);
    assert_eq!(AccountType::Equity.normal_balance_sign(), -1);
    assert_eq!(AccountType::Income.normal_balance_sign(), -1);
    assert_eq!(AccountType::Expense.normal_balance_sign(), 1);
}
