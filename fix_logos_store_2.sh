git restore crates/logos-runtime/src/runtime/mod.rs
sed -i 's/logos_store::error::StoreError/logos_store::StoreError/g' crates/logos-runtime/src/error/mod.rs
sed -i 's/logos_store::model::StoredFetchRun/logos_store::StoredFetchRun/g' crates/logos-runtime/src/models/mod.rs
sed -i 's/logos_store::model::{StoredMonthClose, StoredReconciliationRun}/logos_store::{StoredMonthClose, StoredReconciliationRun}/g' crates/logos-runtime/src/models/mod.rs

# In runtime/mod.rs:
# logos_store::{
#    MemoryStore,
#    error::StoreError,
#    model::{
#        NewImportRecord, StoredAnalyticsArtifactManifest, StoredFetchArtifactFormat,
#        StoredFetchRun, StoredFetchRunStatus, StoredMonthClose, StoredReconciliationRun,
#        StoredStatementLine, StoredTransaction,
#    },
#    traits::LedgerStore,
#};
sed -i 's/error::StoreError/StoreError/g' crates/logos-runtime/src/runtime/mod.rs
sed -i 's/traits::LedgerStore/LedgerStore/g' crates/logos-runtime/src/runtime/mod.rs
sed -i 's/model::{//g' crates/logos-runtime/src/runtime/mod.rs
# also need to remove the closing bracket of model::{
sed -i '/StoredStatementLine, StoredTransaction,/!b;n;s/    },//g' crates/logos-runtime/src/runtime/mod.rs
