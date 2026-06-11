use logos_core::AllocationPolicy;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_allocation_policy(
        tax in any::<u8>(),
        smoothing in any::<u8>(),
        goals in any::<u8>(),
        discretionary in any::<u8>(),
    ) {
        let _ = AllocationPolicy::new(tax, smoothing, goals, discretionary);
    }
}
