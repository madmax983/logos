use logos_core::AccountType;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_account_type_normal_balance_sign(
        val in 0..5usize,
    ) {
        let ty = match val {
            0 => AccountType::Asset,
            1 => AccountType::Liability,
            2 => AccountType::Equity,
            3 => AccountType::Income,
            4 => AccountType::Expense,
            _ => unreachable!(),
        };
        let _ = ty.normal_balance_sign();
    }
}
