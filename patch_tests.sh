import os

filepath = "crates/logos-reporting/tests/havoc_proptest.rs"
with open(filepath, "r") as f:
    content = f.read()

content = content.replace("#[should_panic(expected = \"attempt to subtract with overflow\")]\n    fn project_cashflow_panics_on_overflow", "fn project_cashflow_saturates_on_overflow")
content = content.replace("#[should_panic(expected = \"attempt to subtract with overflow\")]\n    fn project_net_worth_panics_on_overflow", "fn project_net_worth_saturates_on_overflow")
content = content.replace("#[should_panic(expected = \"attempt to subtract with overflow\")]\n    fn project_budget_variance_panics_on_overflow", "fn project_budget_variance_saturates_on_overflow")

content = content.replace("let _ = project_cashflow(income, expense);", "assert_eq!(project_cashflow(i64::MIN, expense), i64::MIN);")
content = content.replace("let _ = project_net_worth(assets, liabilities);", "assert_eq!(project_net_worth(i64::MIN, liabilities), i64::MIN);")
content = content.replace("let _ = project_budget_variance(budget, actual);", "assert_eq!(project_budget_variance(i64::MIN, actual), i64::MIN);")


with open(filepath, "w") as f:
    f.write(content)
