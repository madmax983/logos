sed -i 's/use logos_store::traits::LedgerStore/use logos_store::LedgerStore/g' crates/logos-store-pg/tests/*.rs
sed -i 's/use logos_store::model::/use logos_store::/g' crates/logos-store-pg/tests/*.rs
