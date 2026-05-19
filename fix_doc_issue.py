import re
import os

# There are only a few that are explicitly missing doc comments as reported by the script.
# "Missing doc comments" means no `///` at all. The script output was:
# crates/logos-store-pg/src/store.rs:320 missing doc: pub struct PostgresStore {
# crates/logos-fetch/src/model.rs:322 missing doc: pub fn is_valid_month_key(value: &str) -> bool {

with open('crates/logos-store-pg/src/store.rs', 'r') as f:
    content = f.read()

new_content = content.replace("pub struct PostgresStore {", "/// Concrete PostgreSQL storage implementation.\n///\n/// Handles connecting to the database and executing queries to implement the `LedgerStore` trait.\n///\n/// ## Examples\n///\n/// ```rust,ignore\n/// use logos_store_pg::PostgresStore;\n/// let store = PostgresStore::connect(\"postgres://logos:logos@127.0.0.1/logos\").unwrap();\n/// ```\npub struct PostgresStore {")

with open('crates/logos-store-pg/src/store.rs', 'w') as f:
    f.write(new_content)

with open('crates/logos-fetch/src/model.rs', 'r') as f:
    content = f.read()

new_content = content.replace("pub fn is_valid_month_key(value: &str) -> bool {", "/// Checks if a string is a valid month key in the format `YYYY-MM`.\n///\n/// ## Examples\n///\n/// ```\n/// use logos_fetch::model::is_valid_month_key;\n/// assert!(is_valid_month_key(\"2026-03\"));\n/// assert!(!is_valid_month_key(\"2026-13\"));\n/// ```\npub fn is_valid_month_key(value: &str) -> bool {")

with open('crates/logos-fetch/src/model.rs', 'w') as f:
    f.write(new_content)
