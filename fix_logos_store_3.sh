sed -i 's/logos_store::model::StoredMonthClose/logos_store::StoredMonthClose/g' crates/logos-cli/src/commands/close.rs
sed -i 's/logos_store::model::StoredFetchRun/logos_store::StoredFetchRun/g' crates/logos-cli/src/commands/fetch.rs
sed -i 's/logos_store::model::StoredFetchRunStatus/logos_store::StoredFetchRunStatus/g' crates/logos-cli/src/commands/month.rs
sed -i 's/logos_store::model::StoredReconciliationRun/logos_store::StoredReconciliationRun/g' crates/logos-cli/src/commands/reconcile.rs
sed -i 's/logos_store::model::StoredAnalyticsArtifactManifest/logos_store::StoredAnalyticsArtifactManifest/g' crates/logos-cli/src/commands/analytics.rs
