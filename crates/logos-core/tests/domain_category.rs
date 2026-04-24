use logos_core::{Category, CategoryGroup};

#[test]
fn test_category_group_name() {
    let group = CategoryGroup::new("Expenses").unwrap();
    assert_eq!(group.name(), "Expenses");
}

#[test]
fn test_category_name() {
    let group = CategoryGroup::new("Expenses").unwrap();
    let category = Category::new(group.id().clone(), "Groceries").unwrap();
    assert_eq!(category.name(), "Groceries");
}
