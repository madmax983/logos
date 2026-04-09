sed -i 's/use logos_store::model::{/use logos_store::{/g' crates/logos-store/tests/memory_store.rs
sed -i 's/use logos_store::model::{/use logos_store::{/g' crates/logos-store/tests/model_smoke.rs
sed -i 's/use logos_store::error::StoreError/use logos_store::StoreError/g' crates/logos-store/tests/model_smoke.rs
sed -i 's/use logos_store::traits::LedgerStore/use logos_store::LedgerStore/g' crates/logos-store/tests/model_smoke.rs
