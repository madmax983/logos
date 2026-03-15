with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    c = f.read()

# Add #[allow(clippy::needless_collect)] to the tests
c = c.replace("fn test_store_empty_accessors() {", "#[allow(clippy::needless_collect)]\nfn test_store_empty_accessors() {")
c = c.replace("fn test_store_populated_accessors() {", "#[allow(clippy::needless_collect)]\nfn test_store_populated_accessors() {")

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(c)
