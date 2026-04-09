sed -i 's/pub mod/pub(crate) mod/g' crates/logos-store/src/lib.rs
sed -i 's/logos_store::error::StoreError/logos_store::StoreError/g' crates/logos-store-pg/src/store.rs
sed -i 's/logos_store::model::{/logos_store::{/g' crates/logos-store-pg/src/store.rs
sed -i 's/logos_store::traits::LedgerStore/logos_store::LedgerStore/g' crates/logos-store-pg/src/store.rs

sed -i 's/logos_store::error::StoreError/logos_store::StoreError/g' crates/logos-runtime/src/runtime/mod.rs
sed -i 's/logos_store::model::{/logos_store::{/g' crates/logos-runtime/src/runtime/mod.rs
sed -i 's/logos_store::traits::LedgerStore/logos_store::LedgerStore/g' crates/logos-runtime/src/runtime/mod.rs
