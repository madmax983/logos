use logos_core::CategoryGroupId;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_category_group_id_from_name(name in "\\PC*") {
        let _ = CategoryGroupId::from_name(&name);
    }
}
