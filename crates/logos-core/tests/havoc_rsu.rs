use logos_core::HaircutTierTable;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_haircut_tier_table(
        short in any::<u8>(),
        medium in any::<u8>(),
        long in any::<u8>(),
    ) {
        let _ = HaircutTierTable::new(short, medium, long);
    }
}
