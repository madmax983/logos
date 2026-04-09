sed -i 's/pub(crate) mod schema/pub mod schema/g' crates/logos-store-pg/src/lib.rs

sed -i 's/logos_store::error::StoreError/logos_store::StoreError/g' crates/logos-store-pg/tests/store_contract.rs

sed -i 's/use logos_store::model::{/use logos_store::{/g' crates/logos-cli/src/commands/fetch.rs
sed -i 's/use logos_store::model::{/use logos_store::{/g' crates/logos-cli/src/commands/month.rs
