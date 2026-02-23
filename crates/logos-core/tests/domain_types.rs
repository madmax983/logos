use logos_core::{AccountType, Category, CategoryGroup};

#[test]
fn account_types_and_category_hierarchy_are_constructible() {
    let group = CategoryGroup::new("needs").expect("group");
    let category = Category::new(group.id().clone(), "rent").expect("category");

    assert_eq!(category.group_id(), group.id());
    assert_eq!(AccountType::Asset.normal_balance_sign(), 1);
}
