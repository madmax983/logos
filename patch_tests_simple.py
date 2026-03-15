with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    c = f.read()

c = c.replace("assert_eq!(store.transactions().count(), 0);", "assert_eq!(store.transactions().count(), 0);\n    assert_eq!(store.transactions().collect::<Vec<_>>().len(), 0);")
c = c.replace("assert_eq!(store.budget_targets().count(), 0);", "assert_eq!(store.budget_targets().count(), 0);\n    assert_eq!(store.budget_targets().collect::<Vec<_>>().len(), 0);")
c = c.replace("assert_eq!(store.analytics_artifacts().count(), 0);", "assert_eq!(store.analytics_artifacts().count(), 0);\n    assert_eq!(store.analytics_artifacts().collect::<Vec<_>>().len(), 0);")
c = c.replace("assert_eq!(store.import_records().count(), 0);", "assert_eq!(store.import_records().count(), 0);\n    assert_eq!(store.import_records().collect::<Vec<_>>().len(), 0);")
c = c.replace("assert_eq!(store.import_batches().count(), 0);", "assert_eq!(store.import_batches().count(), 0);\n    assert_eq!(store.import_batches().collect::<Vec<_>>().len(), 0);")
c = c.replace("assert_eq!(store.reconciliation_runs().count(), 0);", "assert_eq!(store.reconciliation_runs().count(), 0);\n    assert_eq!(store.reconciliation_runs().collect::<Vec<_>>().len(), 0);")
c = c.replace("assert_eq!(store.month_closes().count(), 0);", "assert_eq!(store.month_closes().count(), 0);\n    assert_eq!(store.month_closes().collect::<Vec<_>>().len(), 0);")

c = c.replace("assert_eq!(store.transactions().count(), 1);", "assert_eq!(store.transactions().count(), 1);\n    assert_eq!(store.transactions().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.budget_targets().count(), 1);", "assert_eq!(store.budget_targets().count(), 1);\n    assert_eq!(store.budget_targets().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.budget_targets().count(), 2);", "assert_eq!(store.budget_targets().count(), 2);\n    assert_eq!(store.budget_targets().collect::<Vec<_>>().len(), 2);")
c = c.replace("assert_eq!(store.analytics_artifacts().count(), 1);", "assert_eq!(store.analytics_artifacts().count(), 1);\n    assert_eq!(store.analytics_artifacts().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.analytics_artifacts().count(), 2);", "assert_eq!(store.analytics_artifacts().count(), 2);\n    assert_eq!(store.analytics_artifacts().collect::<Vec<_>>().len(), 2);")
c = c.replace("assert_eq!(store.import_records().count(), 1);", "assert_eq!(store.import_records().count(), 1);\n    assert_eq!(store.import_records().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.import_records().count(), 2);", "assert_eq!(store.import_records().count(), 2);\n    assert_eq!(store.import_records().collect::<Vec<_>>().len(), 2);")
c = c.replace("assert_eq!(store.import_batches().count(), 1);", "assert_eq!(store.import_batches().count(), 1);\n    assert_eq!(store.import_batches().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.import_batches().count(), 2);", "assert_eq!(store.import_batches().count(), 2);\n    assert_eq!(store.import_batches().collect::<Vec<_>>().len(), 2);")
c = c.replace("assert_eq!(store.reconciliation_runs().count(), 1);", "assert_eq!(store.reconciliation_runs().count(), 1);\n    assert_eq!(store.reconciliation_runs().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.reconciliation_runs().count(), 2);", "assert_eq!(store.reconciliation_runs().count(), 2);\n    assert_eq!(store.reconciliation_runs().collect::<Vec<_>>().len(), 2);")
c = c.replace("assert_eq!(store.month_closes().count(), 1);", "assert_eq!(store.month_closes().count(), 1);\n    assert_eq!(store.month_closes().collect::<Vec<_>>().len(), 1);")
c = c.replace("assert_eq!(store.month_closes().count(), 2);", "assert_eq!(store.month_closes().count(), 2);\n    assert_eq!(store.month_closes().collect::<Vec<_>>().len(), 2);")

c = c.replace("let mut proj = store.current_projection_without_superseded();", "let mut proj = store.current_projection_without_superseded();\n    assert!(!proj.iter().any(|t| t.id().as_str() == \"missing\"));")

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(c)
